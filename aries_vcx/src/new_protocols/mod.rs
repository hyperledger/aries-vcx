use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds,
    ledger::base_ledger::{AnoncredsLedgerRead, AnoncredsLedgerWrite, IndyLedgerRead, IndyLedgerWrite},
    wallet::base_wallet::BaseWallet,
};
use async_trait::async_trait;
use chrono::Utc;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::{
    decorators::{thread::Thread, timing::Timing},
    msg_fields::protocols::connection::{
        invitation::{
            InvitationContent, PairwiseDidInvitationContent, PairwiseInvitationContent, PublicInvitationContent,
        },
        request::RequestContent,
        response::ResponseContent,
        ConnectionData,
    },
};
use url::Url;

use crate::{
    common::{
        ledger::transactions::get_service,
        signing::{decode_signed_connection_response, sign_connection_response},
    },
    errors::error::VcxResult,
};

pub struct BootstrapInfo {
    service_endpoint: Url,
    recipient_keys: Vec<String>,
    routing_keys: Vec<String>,
    did: Option<String>,
    service_endpoint_did: Option<String>,
}

#[async_trait]
pub trait MessageHandler {
    type LedgerRead: IndyLedgerRead + AnoncredsLedgerRead;

    type LedgerWrite: IndyLedgerWrite + AnoncredsLedgerWrite;

    type Wallet: BaseWallet;

    type Anoncreds: BaseAnonCreds;

    fn wallet(&self) -> &Self::Wallet;

    fn ledger_read(&self) -> &Self::LedgerRead;

    fn ledger_write(&self) -> &Self::LedgerWrite;

    fn anoncreds(&self) -> &Self::Anoncreds;

    async fn bootstrap_info_from_public_invitation(
        &self,
        invitation: PublicInvitationContent,
    ) -> VcxResult<BootstrapInfo> {
        let service = match get_service(self.ledger_read(), &invitation.did).await {
            Ok(s) => s,
            Err(err) => {
                error!("Failed to obtain service definition from the ledger: {}", err);
                return Err(err);
            }
        };

        let info = BootstrapInfo {
            service_endpoint: service.service_endpoint,
            recipient_keys: service.recipient_keys,
            routing_keys: service.routing_keys,
            did: Some(invitation.did),
            service_endpoint_did: None,
        };

        Ok(info)
    }

    fn bootstrap_info_from_pw_invitation(&self, invitation: PairwiseInvitationContent) -> BootstrapInfo {
        BootstrapInfo {
            service_endpoint: invitation.service_endpoint,
            recipient_keys: invitation.recipient_keys,
            routing_keys: invitation.routing_keys,
            did: None,
            service_endpoint_did: None,
        }
    }

    async fn bootstrap_info_from_pw_did_invitation(
        &self,
        mut invitation: PairwiseDidInvitationContent,
    ) -> VcxResult<BootstrapInfo> {
        let service = match get_service(self.ledger_read(), &invitation.service_endpoint).await {
            Ok(s) => s,
            Err(err) => {
                error!("Failed to obtain service definition from the ledger: {}", err);
                return Err(err);
            }
        };

        // See https://github.com/hyperledger/aries-rfcs/blob/main/features/0160-connection-protocol/README.md#agency-endpoint
        invitation.routing_keys.extend(service.recipient_keys);

        let info = BootstrapInfo {
            service_endpoint: service.service_endpoint,
            recipient_keys: invitation.recipient_keys,
            routing_keys: invitation.routing_keys,
            did: None,
            service_endpoint_did: Some(invitation.service_endpoint),
        };

        Ok(info)
    }

    async fn bootstrap_info_from_invitation(&self, invitation: InvitationContent) -> VcxResult<BootstrapInfo> {
        match invitation {
            InvitationContent::Public(invitation) => self.bootstrap_info_from_public_invitation(invitation).await,
            InvitationContent::Pairwise(invitation) => Ok(self.bootstrap_info_from_pw_invitation(invitation)),
            InvitationContent::PairwiseDID(invitation) => self.bootstrap_info_from_pw_did_invitation(invitation).await,
        }
    }

    async fn build_response_content(
        &self,
        verkey: &str,
        did: String,
        recipient_keys: Vec<String>,
        new_service_endpoint: Url,
        new_routing_keys: Vec<String>,
    ) -> VcxResult<ResponseContent> {
        let mut did_doc = AriesDidDoc::default();

        did_doc.set_id(did.clone());
        did_doc.set_service_endpoint(new_service_endpoint);
        did_doc.set_routing_keys(new_routing_keys);
        did_doc.set_recipient_keys(recipient_keys);

        let con_data = ConnectionData::new(did, did_doc);
        let con_sig = sign_connection_response(self.wallet(), verkey, &con_data).await?;
        let content = ResponseContent::new(con_sig);

        Ok(content)
    }

    fn make_reply_thread(&self, thread: Thread) -> Thread {
        Thread::new(thread.thid)
    }

    fn make_subthread(&self, thread: Thread, thread_id: String) -> Thread {
        let mut subthread = Thread::new(thread_id);
        subthread.pthid = Some(thread.thid);
        subthread
    }

    fn make_connection_request_thread(
        &self,
        invitation_thread: Thread,
        request_id: String,
        public_invite: bool,
    ) -> Thread {
        if public_invite {
            self.make_subthread(invitation_thread, request_id)
        } else {
            self.make_reply_thread(invitation_thread)
        }
    }

    fn make_timing(&self) -> Timing {
        Timing {
            out_time: Some(Utc::now()),
            ..Default::default()
        }
    }

    async fn process_connection_invitation(&self, msg_content: InvitationContent) -> VcxResult<BootstrapInfo> {
        //! This could arguably be a method on the invitation
        self.bootstrap_info_from_invitation(msg_content).await
    }

    async fn process_connection_request(&self, msg_content: RequestContent) -> VcxResult<AriesDidDoc> {
        //! This could arguably be a method on the did doc

        // If the request's DidDoc validation fails, we generate and send a ProblemReport.
        // We then return early with the provided error.
        if let Err(err) = msg_content.connection.did_doc.validate() {
            error!("Request DidDoc validation failed! Sending ProblemReport...");
            // TODO: There is a problem report generated here
            Err(err)?;
        }

        Ok(msg_content.connection.did_doc)
    }

    async fn process_connection_response(&self, msg_content: ResponseContent, verkey: &str) -> VcxResult<AriesDidDoc> {
        //! Let's pretend this function is inlined

        match decode_signed_connection_response(self.wallet(), msg_content, verkey).await {
            Ok(con_data) => Ok(con_data.did_doc),
            Err(err) => {
                // TODO: Theres a ProblemReport being built here.
                // Might be nice to either have a different type for the Err()
                // variant or incorporate ProblemReports into AriesVcxError
                error!("Request DidDoc validation failed! Sending ProblemReport...");
                Err(err)
            }
        }
    }
}

#[cfg(test)]
#[cfg(feature = "vdrtools")]
#[allow(clippy::unwrap_used)]
mod tests {
    use aries_vcx_core::{
        anoncreds::indy_anoncreds::IndySdkAnonCreds,
        ledger::indy_ledger::{IndySdkLedgerRead, IndySdkLedgerWrite},
        wallet::indy::IndySdkWallet,
    };
    use messages::msg_fields::protocols::{
        connection::{
            invitation::Invitation,
            request::{Request, RequestDecorators},
            response::{Response, ResponseDecorators},
        },
        notification::ack::{Ack, AckContent, AckDecorators, AckStatus},
    };
    use uuid::Uuid;

    use crate::{global::settings, utils::devsetup::SetupPoolDirectory};

    use aries_vcx_core::{
        ledger::indy::pool::{create_pool_ledger_config, indy_close_pool, indy_delete_pool, indy_open_pool},
        PoolHandle,
    };

    use crate::utils::devsetup::setup_issuer_wallet;

    use super::*;

    struct MsgHandler {
        ledger_read: IndySdkLedgerRead,
        ledger_write: IndySdkLedgerWrite,
        wallet: IndySdkWallet,
        anoncreds: IndySdkAnonCreds,
    }

    #[async_trait]
    impl MessageHandler for MsgHandler {
        type LedgerRead = IndySdkLedgerRead;

        type LedgerWrite = IndySdkLedgerWrite;

        type Wallet = IndySdkWallet;

        type Anoncreds = IndySdkAnonCreds;

        fn wallet(&self) -> &Self::Wallet {
            &self.wallet
        }

        fn ledger_read(&self) -> &Self::LedgerRead {
            &self.ledger_read
        }

        fn ledger_write(&self) -> &Self::LedgerWrite {
            &self.ledger_write
        }

        fn anoncreds(&self) -> &Self::Anoncreds {
            &self.anoncreds
        }
    }

    async fn indy_teardown(pool_handle: PoolHandle, pool_name: String) {
        indy_close_pool(pool_handle).await.unwrap();
        indy_delete_pool(&pool_name).await.unwrap();
    }

    async fn build_msg_handler(pool_handle: i32) -> (String, MsgHandler) {
        let (did, wallet_handle) = setup_issuer_wallet().await;

        let wallet = IndySdkWallet::new(wallet_handle);
        let anoncreds = IndySdkAnonCreds::new(wallet_handle);
        let ledger_read = IndySdkLedgerRead::new(wallet_handle, pool_handle);
        let ledger_write = IndySdkLedgerWrite::new(wallet_handle, pool_handle);

        anoncreds
            .prover_create_link_secret(settings::DEFAULT_LINK_SECRET_ALIAS)
            .await
            .unwrap();

        let msg_handler = MsgHandler {
            ledger_read,
            ledger_write,
            wallet,
            anoncreds,
        };

        (did, msg_handler)
    }

    async fn _test_connection_handler(pool_handle: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (_, faber) = build_msg_handler(pool_handle).await;
        let (_, alice) = build_msg_handler(pool_handle).await;

        let dummy_endpoint = Url::parse("https://dummy.dummy/dummy")?;
        let (_, faber_invite_verkey) = faber.wallet().create_and_store_my_did(None, None).await?;

        let invitation_content = PairwiseInvitationContent::new(
            "faber".to_owned(),
            vec![faber_invite_verkey.clone()],
            vec![],
            dummy_endpoint.clone(),
        );
        let invitation = Invitation::with_decorators(
            Uuid::new_v4().to_string(),
            InvitationContent::Pairwise(invitation_content),
            Default::default(),
        );

        let request_id = Uuid::new_v4().to_string();

        let thread = match &invitation.content {
            InvitationContent::Public(_) => {
                let mut thread = Thread::new(request_id.clone());
                thread.pthid = Some(invitation.id.clone());
                thread
            }
            InvitationContent::Pairwise(_) => Thread::new(invitation.id.clone()),
            InvitationContent::PairwiseDID(_) => Thread::new(invitation.id.clone()),
        };

        // Extract info from invitation
        let bootstrap_info = alice.process_connection_invitation(invitation.content).await?;

        // Build request
        let (alice_did, alice_verkey) = alice.wallet().create_and_store_my_did(None, None).await?;
        let mut did_doc = AriesDidDoc {
            id: alice_did.clone(),
            ..Default::default()
        };
        did_doc.set_service_endpoint(bootstrap_info.service_endpoint);
        did_doc.set_routing_keys(bootstrap_info.routing_keys);
        did_doc.set_recipient_keys(bootstrap_info.recipient_keys);

        let con_data = ConnectionData::new(alice_did, did_doc);

        let request_content = RequestContent::new("my_request".to_owned(), con_data);
        let timing = Timing {
            out_time: Some(Utc::now()),
            ..Default::default()
        };

        let request_decorators = RequestDecorators {
            thread: Some(thread),
            timing: Some(timing),
        };

        let request = Request::with_decorators(request_id, request_content, request_decorators);

        // Process request
        let alice_did_doc = faber.process_connection_request(request.content).await?;
        let recipient_keys = alice_did_doc.recipient_keys()?;
        let their_verkey = recipient_keys.first().ok_or("no recipient keys")?.as_str();

        // Build response
        let (faber_did, faber_verkey) = faber.wallet().create_and_store_my_did(None, None).await?;
        let response_content = faber
            .build_response_content(their_verkey, faber_did, vec![faber_verkey], dummy_endpoint, vec![])
            .await?;

        let timing = Timing {
            out_time: Some(Utc::now()),
            ..Default::default()
        };

        let response_decorators = ResponseDecorators {
            thread: Thread::new(request.decorators.thread.map(|t| t.thid).unwrap_or(request.id)),
            please_ack: None,
            timing: Some(timing),
        };

        let response = Response::with_decorators(Uuid::new_v4().to_string(), response_content, response_decorators);

        // Process response
        let _faber_did_doc = alice
            .process_connection_response(response.content, &alice_verkey)
            .await?;

        // Build ack
        let ack_content = AckContent::new(AckStatus::Ok);

        let timing = Timing {
            out_time: Some(Utc::now()),
            ..Default::default()
        };

        let ack_decorators = AckDecorators {
            thread: Thread::new(response.decorators.thread.thid),
            timing: Some(timing),
        };

        let ack = Ack::with_decorators(Uuid::new_v4().to_string(), ack_content, ack_decorators);

        // Process ack
        // The inviter merely needs to see this message (after decryption) to assess that
        // the connection is now complete for them as well.
        let _ = ack;

        Ok(())
    }

    #[tokio::test]
    async fn test_connection_handler() {
        SetupPoolDirectory::run(|setup| async move {
            let pool_name = Uuid::new_v4().to_string();
            create_pool_ledger_config(&pool_name, &setup.genesis_file_path).unwrap();
            let pool_handle = indy_open_pool(&pool_name, None).await.unwrap();

            _test_connection_handler(pool_handle).await.ok();
            indy_teardown(pool_handle, pool_name).await;
        })
        .await
    }
}
