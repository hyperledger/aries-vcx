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
    msg_fields::protocols::{
        connection::{
            invitation::{
                Invitation, InvitationContent, PairwiseDidInvitationContent, PairwiseInvitationContent,
                PublicInvitationContent,
            },
            problem_report::ProblemReport,
            request::{Request, RequestDecorators},
            response::{Response, ResponseContent, ResponseDecorators},
            ConnectionData,
        },
        notification::{
            ack::{Ack, AckDecorators},
            problem_report::NotificationProblemReport,
        },
    },
};
use url::Url;
use uuid::Uuid;

use crate::{
    common::{
        ledger::transactions::get_service,
        signing::{decode_signed_connection_response, sign_connection_response},
    },
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
};

use self::connection::{
    invitee::{state::BootstrapInfo, InviteeConnection},
    inviter::InviterConnection,
    ConnectionSM,
};

pub mod connection;

/// Enum that can represent any Aries state machine, in any of their states.
#[derive(Clone, Debug)]
pub enum AriesSM {
    Connection(ConnectionSM),
}

/// Interface for handling the storage and retrieval of [`AriesSM`].
#[async_trait]
pub trait StateMachineStorage: Send + Sync {
    /// Type is used for identifying a particular [`AriesSM`] instance.
    type SmInfo: Send + Sync;

    /// Retrieves the state machine with the given information.
    /// This is intended to transfer the state machine's ownership, if possible.
    ///
    /// If, for instance, you (also) store your state machines in an
    /// in-memory cache, on a cache hit you should remove the instance
    /// from the cache and return the owned state machine (not clone it).
    ///
    /// Also see [`StateMachineStorage::put_different_state`] and [`StateMachineStorage::put_same_state`].
    async fn get(&self, sm_info: &Self::SmInfo) -> Result<AriesSM, AriesVcxError>;

    /// Used for storing a state machine in the event that its state *DID* change.
    /// This should update ALL places where you store your state machines.
    ///
    /// If, for instance, you store your state machines in a disk-based database
    /// and an in-memory cache, this should update both.
    ///
    /// Also see [`StateMachineStorage::get`] and [`StateMachineStorage::put_same_state`].
    async fn put_new_state(&self, sm_info: Self::SmInfo, sm: AriesSM) -> Result<(), AriesVcxError>;

    /// Used for storing a state machine in the event that its state *DID NOT* change.
    /// This is present to allow storage optimizations.
    ///
    /// If, for instance, you store your state machines in a disk-based database
    /// and an in-memory cache, this should ONLY update the in-memory cache.
    ///
    /// Also see [`StateMachineStorage::get`] and [`StateMachineStorage::put_different_state`].
    async fn put_same_state(&self, sm_info: Self::SmInfo, sm: AriesSM) -> Result<(), AriesVcxError>;
}

#[async_trait]
pub trait MessageHandler {
    type LedgerRead: IndyLedgerRead + AnoncredsLedgerRead;

    type LedgerWrite: IndyLedgerWrite + AnoncredsLedgerWrite;

    type Wallet: BaseWallet;

    type Anoncreds: BaseAnonCreds;

    type StateMachineStorage: StateMachineStorage;

    fn wallet(&self) -> &Self::Wallet;

    fn ledger_read(&self) -> &Self::LedgerRead;

    fn ledger_write(&self) -> &Self::LedgerWrite;

    fn anoncreds(&self) -> &Self::Anoncreds;

    fn sm_storage(&self) -> &Self::StateMachineStorage;

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

    async fn process_connection_invitation(
        &self,
        sm_info: <Self::StateMachineStorage as StateMachineStorage>::SmInfo,
        message: Invitation,
        service_endpoint: Url,
        routing_keys: Vec<String>,
        label: String,
    ) -> VcxResult<Request> {
        let public_invite = matches!(message.content, InvitationContent::Public(_));

        let bootstrap_info = self.bootstrap_info_from_invitation(message.content).await?;
        let (did, verkey) = self.wallet().create_and_store_my_did(None, None).await?;

        let recipient_keys = vec![verkey.clone()];

        let mut did_doc = AriesDidDoc::default();
        did_doc.id = did.clone();
        did_doc.set_service_endpoint(service_endpoint);
        did_doc.set_routing_keys(routing_keys);
        did_doc.set_recipient_keys(recipient_keys);

        let con_data = ConnectionData::new(did.clone(), did_doc);

        let request_id = Uuid::new_v4().to_string();
        let invitation_thread = Thread::new(message.id);
        let thread = self.make_connection_request_thread(invitation_thread, request_id.clone(), public_invite);

        let (sm, content) =
            InviteeConnection::new_invitee(did, verkey, label, bootstrap_info, con_data, thread.thid.clone());

        let decorators = RequestDecorators {
            thread: Some(thread),
            timing: Some(self.make_timing()),
        };

        let request = Request::with_decorators(request_id, content, decorators);
        let sm = AriesSM::Connection(ConnectionSM::InviteeRequested(sm));
        self.sm_storage().put_new_state(sm_info, sm).await?;

        Ok(request)
    }

    async fn process_connection_request(
        &self,
        sm_info: <Self::StateMachineStorage as StateMachineStorage>::SmInfo,
        message: Request,
        invitation_verkey: &str,
        service_endpoint: Url,
        routing_keys: Vec<String>,
    ) -> VcxResult<Response> {
        // If the request's DidDoc validation fails, we generate and send a ProblemReport.
        // We then return early with the provided error.
        if let Err(err) = message.content.connection.did_doc.validate() {
            error!("Request DidDoc validation failed! Sending ProblemReport...");
            // TODO: There is a problem report generated here
            Err(err)?;
        }

        // Generate new pairwise info that will be used from this point on
        // and incorporate that into the response.
        let (did, verkey) = self.wallet().create_and_store_my_did(None, None).await?;
        let thread = message.decorators.thread.unwrap_or_else(|| Thread::new(message.id));
        let did_doc = message.content.connection.did_doc;

        let content = self
            .build_response_content(
                invitation_verkey,
                did.clone(),
                vec![verkey.clone()],
                service_endpoint,
                routing_keys,
            )
            .await?;

        let id = Uuid::new_v4().to_string();

        let decorators = ResponseDecorators {
            thread: self.make_reply_thread(thread),
            please_ack: None,
            timing: Some(self.make_timing()),
        };

        let response = Response::with_decorators(id, content, decorators);

        let sm = InviterConnection::new_inviter(did, verkey, did_doc);
        let sm = AriesSM::Connection(ConnectionSM::InviterComplete(sm));
        self.sm_storage().put_new_state(sm_info, sm).await?;

        Ok(response)
    }

    async fn process_connection_response(
        &self,
        sm_info: <Self::StateMachineStorage as StateMachineStorage>::SmInfo,
        message: Response,
    ) -> VcxResult<Ack> {
        let sm = match self.sm_storage().get(&sm_info).await? {
            AriesSM::Connection(ConnectionSM::InviteeRequested(sm)) => sm,
            _ => todo!("Add some error here in the event of unexpected state machine"),
        };

        match Self::thread_id_matches(&message.decorators.thread, &sm.thread_id) {
            Ok(_) => (),
            Err(e) => {
                self.sm_storage().put_same_state(sm_info, sm.into()).await?;
                return Err(e);
            }
        };

        let Some(verkey) = sm.state.bootstrap_info.recipient_keys.first() else {
            self.sm_storage().put_same_state(sm_info, sm.into()).await?;
            todo!("Add some error in case no recipient key is found")
        };

        let did_doc = match decode_signed_connection_response(self.wallet(), message.content, verkey).await {
            Ok(con_data) => con_data.did_doc,
            Err(err) => {
                // TODO: Theres a ProblemReport being built here.
                // Might be nice to either have a different type for the Err()
                // variant or incorporate ProblemReports into AriesVcxError
                self.sm_storage().put_same_state(sm_info, sm.into()).await?;
                error!("Request DidDoc validation failed! Sending ProblemReport...");
                return Err(err);
            }
        };

        let (sm, content) = sm.into_complete(did_doc);

        let decorators = AckDecorators {
            thread: self.make_reply_thread(message.decorators.thread),
            timing: Some(self.make_timing()),
        };

        let msg_id = Uuid::new_v4().to_string();

        let ack = Ack::with_decorators(msg_id, content, decorators);
        let sm = AriesSM::Connection(ConnectionSM::InviteeComplete(sm));
        self.sm_storage().put_new_state(sm_info, sm).await?;

        Ok(ack)
    }

    async fn process_connection_problem_report(&self, message: ProblemReport) {
        todo!()
    }

    async fn process_notification_ack(&self, message: Ack) {
        todo!()
    }

    async fn process_notification_problem_report(&self, message: NotificationProblemReport) {
        todo!()
    }

    fn thread_id_matches(thread: &Thread, thread_id: &str) -> VcxResult<()> {
        if thread.thid == thread_id || thread.pthid.as_deref() == Some(thread_id) {
            return Ok(());
        }

        let msg = format!(
            "Thread id does not match, expected {thread_id}, found thread ID: {}; parent thread ID: {:?}",
            thread.thid, thread.pthid
        );
        Err(AriesVcxError::from_msg(AriesVcxErrorKind::InvalidJson, msg))
    }
}

#[cfg(test)]
#[cfg(feature = "vdrtools")]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use aries_vcx_core::{
        anoncreds::indy_anoncreds::IndySdkAnonCreds,
        ledger::indy_ledger::{IndySdkLedgerRead, IndySdkLedgerWrite},
        wallet::indy::IndySdkWallet,
    };
    use tokio::sync::Mutex;

    use crate::{global::settings, utils::devsetup::SetupPoolDirectory};

    use aries_vcx_core::{
        ledger::indy::pool::{create_pool_ledger_config, indy_close_pool, indy_delete_pool, indy_open_pool},
        PoolHandle,
    };

    use crate::utils::devsetup::setup_issuer_wallet;

    use super::*;

    struct SmStorage(Arc<Mutex<HashMap<String, AriesSM>>>);

    #[async_trait]
    impl StateMachineStorage for SmStorage {
        type SmInfo = String;

        async fn get(&self, sm_info: &Self::SmInfo) -> Result<AriesSM, AriesVcxError> {
            self.0
                .lock()
                .await
                .remove(sm_info)
                .ok_or_else(|| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidJson, "state machine not found"))
        }
        async fn put_new_state(&self, sm_info: Self::SmInfo, sm: AriesSM) -> Result<(), AriesVcxError> {
            self.0.lock().await.insert(sm_info, sm);
            Ok(())
        }
        async fn put_same_state(&self, sm_info: Self::SmInfo, sm: AriesSM) -> Result<(), AriesVcxError> {
            self.put_new_state(sm_info, sm).await
        }
    }

    struct MsgHandler {
        ledger_read: IndySdkLedgerRead,
        ledger_write: IndySdkLedgerWrite,
        wallet: IndySdkWallet,
        anoncreds: IndySdkAnonCreds,
        sm_storage: SmStorage,
    }

    #[async_trait]
    impl MessageHandler for MsgHandler {
        type LedgerRead = IndySdkLedgerRead;

        type LedgerWrite = IndySdkLedgerWrite;

        type Wallet = IndySdkWallet;

        type Anoncreds = IndySdkAnonCreds;

        type StateMachineStorage = SmStorage;

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

        fn sm_storage(&self) -> &Self::StateMachineStorage {
            &self.sm_storage
        }
    }

    async fn indy_teardown(pool_handle: PoolHandle, pool_name: String) {
        indy_close_pool(pool_handle).await.unwrap();
        indy_delete_pool(&pool_name).await.unwrap();
    }

    async fn build_msg_handler(genesis_file_path: &str) -> (String, MsgHandler) {
        let pool_name = Uuid::new_v4().to_string();
        create_pool_ledger_config(&pool_name, genesis_file_path).unwrap();
        let pool_handle = indy_open_pool(&pool_name, None).await.unwrap();

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
            sm_storage: SmStorage(Arc::new(Mutex::new(HashMap::new()))),
        };

        (did, msg_handler)
    }

    #[tokio::test]
    async fn test_connection_handler() {
        SetupPoolDirectory::run(|setup| async move {
            let (faber_did, faber) = build_msg_handler(&setup.genesis_file_path).await;
            let (alice_did, alice) = build_msg_handler(&setup.genesis_file_path).await;

            let invitation_content = PublicInvitationContent::new("faber".to_owned(), faber_did.clone());
            let invitation_id = Uuid::new_v4().to_string();
            let invitation = Invitation::with_decorators(
                Uuid::new_v4().to_string(),
                InvitationContent::Public(invitation_content),
                Default::default(),
            );

            // let request = alice.process_connection_invitation(
            //     sm_info,
            //     invitation,
            //     "https://dummy.dummy/dummy".parse().unwrap(),
            //     Vec::new(),
            //     "alice".to_owned(),
            // );
        })
        .await
    }
}
