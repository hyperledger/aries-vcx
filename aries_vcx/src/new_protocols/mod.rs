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
            invitation::Invitation,
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
    handlers::util::matches_thread_id,
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
    type Id: Send + Sync;

    /// Retrieves the state machine with the given id.
    /// This is intended to transfer the state machine's ownership, if possible.
    ///
    /// If, for instance, you (also) store your state machines in an
    /// in-memory cache, on a cache hit you should remove the instance
    /// from the cache and return the owned state machine (not clone it).
    ///
    /// Also see [`StateMachineStorage::put_different_state`] and [`StateMachineStorage::put_same_state`].
    async fn get(&self, id: &Self::Id) -> Result<AriesSM, AriesVcxError>;

    /// Used for storing a state machine in the event that its state *DID* change.
    /// This should update ALL places where you store your state machines.
    ///
    /// If, for instance, you store your state machines in a disk-based database
    /// and an in-memory cache, this should update both.
    ///
    /// Also see [`StateMachineStorage::get`] and [`StateMachineStorage::put_same_state`].
    async fn put_new_state(&self, id: Self::Id, sm: AriesSM) -> Result<(), AriesVcxError>;

    /// Used for storing a state machine in the event that its state *DID NOT* change.
    /// This is present to allow storage optimizations.
    ///
    /// If, for instance, you store your state machines in a disk-based database
    /// and an in-memory cache, this should ONLY update the in-memory cache.
    ///
    /// Also see [`StateMachineStorage::get`] and [`StateMachineStorage::put_different_state`].
    async fn put_same_state(&self, id: Self::Id, sm: AriesSM) -> Result<(), AriesVcxError>;
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

    async fn bootstrap_info_from_invitation(&self, invitation: Invitation) -> VcxResult<BootstrapInfo> {
        let (service_endpoint, recipient_keys, routing_keys, did, service_endpoint_did) = match invitation {
            Invitation::Public(invitation) => {
                let service = match get_service(self.ledger_read(), &invitation.content.did).await {
                    Ok(s) => s,
                    Err(err) => {
                        error!("Failed to obtain service definition from the ledger: {}", err);
                        return Err(err);
                    }
                };

                (
                    service.service_endpoint,
                    service.recipient_keys,
                    service.routing_keys,
                    Some(invitation.content.did),
                    None,
                )
            }
            Invitation::Pairwise(invitation) => (
                invitation.content.service_endpoint,
                invitation.content.recipient_keys,
                invitation.content.routing_keys,
                None,
                None,
            ),
            Invitation::PairwiseDID(mut invitation) => {
                let service = match get_service(self.ledger_read(), &invitation.content.service_endpoint).await {
                    Ok(s) => s,
                    Err(err) => {
                        error!("Failed to obtain service definition from the ledger: {}", err);
                        return Err(err);
                    }
                };

                // See https://github.com/hyperledger/aries-rfcs/blob/main/features/0160-connection-protocol/README.md#agency-endpoint
                invitation.content.routing_keys.extend(service.recipient_keys);

                (
                    service.service_endpoint,
                    invitation.content.recipient_keys,
                    invitation.content.routing_keys,
                    None,
                    Some(invitation.content.service_endpoint),
                )
            }
        };

        let bootstrap_info = BootstrapInfo {
            service_endpoint,
            recipient_keys,
            routing_keys,
            did,
            service_endpoint_did,
        };

        Ok(bootstrap_info)
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

    async fn process_connection_invitation(
        &self,
        sm_id: <Self::StateMachineStorage as StateMachineStorage>::Id,
        message: Invitation,
        service_endpoint: Url,
        routing_keys: Vec<String>,
        label: String,
    ) -> VcxResult<Request> {
        let msg_id = Uuid::new_v4().to_string();

        let thread = match &message {
            Invitation::Public(i) => {
                let mut thread = Thread::new(msg_id.clone());
                thread.pthid = Some(i.id.clone());
                thread
            }
            Invitation::Pairwise(i) => Thread::new(i.id.clone()),
            Invitation::PairwiseDID(i) => Thread::new(i.id.clone()),
        };

        let bootstrap_info = self.bootstrap_info_from_invitation(message).await?;
        let (did, verkey) = self.wallet().create_and_store_my_did(None, None).await?;

        let recipient_keys = vec![verkey.clone()];

        let mut did_doc = AriesDidDoc::default();
        did_doc.id = did.clone();
        did_doc.set_service_endpoint(service_endpoint);
        did_doc.set_routing_keys(routing_keys);
        did_doc.set_recipient_keys(recipient_keys);

        let con_data = ConnectionData::new(did.clone(), did_doc);

        let (sm, content) =
            InviteeConnection::new_invitee(did, verkey, label, bootstrap_info, con_data, thread.thid.clone());

        let timing = Timing {
            out_time: Some(Utc::now()),
            ..Default::default()
        };

        let decorators = RequestDecorators {
            thread: Some(thread),
            timing: Some(timing),
        };

        let request = Request::with_decorators(msg_id, content, decorators);
        let sm = AriesSM::Connection(ConnectionSM::InviteeRequested(sm));
        self.sm_storage().put_new_state(sm_id, sm).await?;

        Ok(request)
    }

    async fn process_connection_request(
        &self,
        sm_id: <Self::StateMachineStorage as StateMachineStorage>::Id,
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
        let thread_id = message.decorators.thread.map(|t| t.thid).unwrap_or(message.id);
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

        let timing = Timing {
            out_time: Some(Utc::now()),
            ..Default::default()
        };

        let decorators = ResponseDecorators {
            thread: Thread::new(thread_id),
            please_ack: None,
            timing: Some(timing),
        };

        let response = Response::with_decorators(id, content, decorators);

        let sm = InviterConnection::new_inviter(did, verkey, did_doc);
        let sm = AriesSM::Connection(ConnectionSM::InviterComplete(sm));
        self.sm_storage().put_new_state(sm_id, sm).await?;

        Ok(response)
    }

    async fn process_connection_response(
        &self,
        sm_id: <Self::StateMachineStorage as StateMachineStorage>::Id,
        message: Response,
    ) -> VcxResult<Ack> {
        let sm = match self.sm_storage().get(&sm_id).await? {
            AriesSM::Connection(ConnectionSM::InviteeRequested(sm)) => sm,
            _ => todo!("Add some error here in the event of unexpected state machine"),
        };

        match Self::thread_id_matches(&message.decorators.thread, &sm.thread_id) {
            Ok(_) => (),
            Err(e) => {
                self.sm_storage().put_same_state(sm_id, sm.into()).await?;
                return Err(e);
            }
        };

        let Some(verkey) = sm.state.bootstrap_info.recipient_keys.first() else {
            self.sm_storage().put_same_state(sm_id, sm.into()).await?;
            todo!("Add some error in case no recipient key is found")
        };

        let did_doc = match decode_signed_connection_response(self.wallet(), message.content, verkey).await {
            Ok(con_data) => con_data.did_doc,
            Err(err) => {
                // TODO: Theres a ProblemReport being built here.
                // Might be nice to either have a different type for the Err()
                // variant or incorporate ProblemReports into AriesVcxError
                let sm = AriesSM::Connection(ConnectionSM::InviteeRequested(sm));
                self.sm_storage().put_same_state(sm_id, sm).await?;
                error!("Request DidDoc validation failed! Sending ProblemReport...");
                return Err(err);
            }
        };

        let (sm, content) = sm.into_complete(did_doc);

        let timing = Timing {
            out_time: Some(Utc::now()),
            ..Default::default()
        };

        let thread = Thread::new(message.decorators.thread.thid);
        let decorators = AckDecorators {
            thread,
            timing: Some(timing),
        };

        let msg_id = Uuid::new_v4().to_string();

        let ack = Ack::with_decorators(msg_id, content, decorators);
        let sm = AriesSM::Connection(ConnectionSM::InviteeComplete(sm));
        self.sm_storage().put_new_state(sm_id, sm).await?;

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

        let msg = format!("Thread id does not match, expected {thread_id}, found: {}", thread.thid);
        Err(AriesVcxError::from_msg(AriesVcxErrorKind::InvalidJson, msg))
    }
}
