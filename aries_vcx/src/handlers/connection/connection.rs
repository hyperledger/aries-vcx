use std::clone::Clone;
use std::sync::Arc;

use messages::a2a::A2AMessage;
use messages::protocols::basic_message::message::BasicMessage;
use messages::protocols::connection::response::SignedResponse;
use serde::{Deserialize, Serialize};

use crate::core::profile::profile::Profile;
use crate::errors::error::prelude::*;
use crate::protocols::connection::invitee::state_machine::{InviteeFullState, InviteeState, SmConnectionInvitee};
use crate::protocols::connection::inviter::state_machine::{InviterFullState, InviterState, SmConnectionInviter};
use crate::protocols::connection::pairwise_info::PairwiseInfo;
use crate::protocols::{SendClosure, SendClosureConnection};
use crate::utils::send_message;
use messages::diddoc::aries::diddoc::AriesDidDoc;
use messages::protocols::connection::invite::Invitation;
use messages::protocols::connection::request::Request;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Connection {
    connection_sm: SmConnection,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum SmConnection {
    Inviter(SmConnectionInviter),
    Invitee(SmConnectionInvitee),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmConnectionState {
    Inviter(InviterFullState),
    Invitee(InviteeFullState),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Inviter(InviterState),
    Invitee(InviteeState),
}

impl Connection {
    // ----------------------------- CONSTRUCTORS ------------------------------------
    pub async fn create_inviter(profile: &Arc<dyn Profile>, pw_info: Option<PairwiseInfo>) -> VcxResult<Self> {
        let pw_info = pw_info.unwrap_or(PairwiseInfo::create(&profile.inject_wallet()).await?);
        trace!("Connection::create_inviter >>>");
        Ok(Self {
            connection_sm: SmConnection::Inviter(SmConnectionInviter::new("", pw_info)),
        })
    }

    pub async fn create_invitee(profile: &Arc<dyn Profile>, did_doc: AriesDidDoc) -> VcxResult<Self> {
        trace!("Connection::create_with_invite >>>");
        Ok(Self {
            connection_sm: SmConnection::Invitee(SmConnectionInvitee::new(
                "",
                PairwiseInfo::create(&profile.inject_wallet()).await?,
                did_doc,
            )),
        })
    }

    // ----------------------------- GETTERS ------------------------------------
    pub fn get_thread_id(&self) -> String {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.get_thread_id(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.get_thread_id(),
        }
    }

    pub fn get_state(&self) -> ConnectionState {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => ConnectionState::Inviter(sm_inviter.get_state()),
            SmConnection::Invitee(sm_invitee) => ConnectionState::Invitee(sm_invitee.get_state()),
        }
    }

    pub fn pairwise_info(&self) -> &PairwiseInfo {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.pairwise_info(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.pairwise_info(),
        }
    }

    pub fn remote_did(&self) -> VcxResult<String> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.remote_did(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.remote_did(),
        }
    }

    pub fn remote_vk(&self) -> VcxResult<String> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.remote_vk(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.remote_vk(),
        }
    }

    pub fn state_object(&self) -> SmConnectionState {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => SmConnectionState::Inviter(sm_inviter.state_object().clone()),
            SmConnection::Invitee(sm_invitee) => SmConnectionState::Invitee(sm_invitee.state_object().clone()),
        }
    }

    pub fn their_did_doc(&self) -> Option<AriesDidDoc> {
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.their_did_doc(),
            SmConnection::Invitee(sm_invitee) => sm_invitee.their_did_doc(),
        }
    }

    pub fn get_invite_details(&self) -> Option<&Invitation> {
        trace!("Connection::get_invite_details >>>");
        match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => sm_inviter.get_invitation(),
            SmConnection::Invitee(_sm_invitee) => None,
        }
    }

    // ----------------------------- MSG PROCESSING ------------------------------------
    pub fn process_invite(self, invitation: Invitation) -> VcxResult<Self> {
        trace!("Connection::process_invite >>> invitation: {:?}", invitation);
        let connection_sm = match &self.connection_sm {
            SmConnection::Inviter(_sm_inviter) => {
                return Err(AriesVcxError::from_msg(AriesVcxErrorKind::NotReady, "Invalid action"));
            }
            SmConnection::Invitee(sm_invitee) => {
                SmConnection::Invitee(sm_invitee.clone().handle_invitation(invitation)?)
            }
        };
        Ok(Self { connection_sm })
    }

    pub async fn process_request(
        self,
        profile: &Arc<dyn Profile>,
        request: Request,
        service_endpoint: String,
        routing_keys: Vec<String>,
        send_message: Option<SendClosureConnection>,
    ) -> VcxResult<Self> {
        trace!(
            "Connection::process_request >>> request: {:?}, service_endpoint: {}, routing_keys: {:?}",
            request,
            service_endpoint,
            routing_keys,
        );
        let connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                let send_message = send_message.unwrap_or(self.send_message_closure_connection(profile));
                let new_pairwise_info = PairwiseInfo::create(&profile.inject_wallet()).await?;
                SmConnection::Inviter(
                    sm_inviter
                        .clone()
                        .handle_connection_request(
                            profile.inject_wallet(),
                            request,
                            &new_pairwise_info,
                            routing_keys,
                            service_endpoint,
                            send_message,
                        )
                        .await?,
                )
            }
            SmConnection::Invitee(_) => {
                return Err(AriesVcxError::from_msg(AriesVcxErrorKind::NotReady, "Invalid action"));
            }
        };
        Ok(Self { connection_sm })
    }

    pub async fn process_response(
        self,
        profile: &Arc<dyn Profile>,
        response: SignedResponse,
        send_message: Option<SendClosureConnection>,
    ) -> VcxResult<Self> {
        let connection_sm = match &self.connection_sm {
            SmConnection::Inviter(_) => {
                return Err(AriesVcxError::from_msg(AriesVcxErrorKind::NotReady, "Invalid action"));
            }
            SmConnection::Invitee(sm_invitee) => {
                let send_message = send_message.unwrap_or(self.send_message_closure_connection(profile));
                SmConnection::Invitee(
                    sm_invitee
                        .clone()
                        .handle_connection_response(&profile.inject_wallet(), response, send_message)
                        .await?,
                )
            }
        };
        Ok(Self { connection_sm })
    }

    pub async fn process_ack(self, message: A2AMessage) -> VcxResult<Self> {
        trace!("Connection::process_ack >>> message: {:?}", message);
        let connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                SmConnection::Inviter(sm_inviter.clone().handle_confirmation_message(&message).await?)
            }
            SmConnection::Invitee(_) => {
                return Err(AriesVcxError::from_msg(AriesVcxErrorKind::NotReady, "Invalid action"));
            }
        };
        Ok(Self { connection_sm })
    }

    // ----------------------------- MSG SENDING ------------------------------------
    pub async fn send_response(
        self,
        profile: &Arc<dyn Profile>,
        send_message: Option<SendClosureConnection>,
    ) -> VcxResult<Self> {
        trace!("Connection::send_response >>>");
        let connection_sm = match self.connection_sm.clone() {
            SmConnection::Inviter(sm_inviter) => {
                if let InviterFullState::Requested(_) = sm_inviter.state_object() {
                    let send_message = send_message.unwrap_or(self.send_message_closure_connection(profile));
                    SmConnection::Inviter(sm_inviter.handle_send_response(send_message).await?)
                } else {
                    return Err(AriesVcxError::from_msg(AriesVcxErrorKind::NotReady, "Invalid action"));
                }
            }
            SmConnection::Invitee(_) => {
                return Err(AriesVcxError::from_msg(AriesVcxErrorKind::NotReady, "Invalid action"));
            }
        };
        Ok(Self { connection_sm })
    }

    pub async fn send_request(
        self,
        profile: &Arc<dyn Profile>,
        service_endpoint: String,
        routing_keys: Vec<String>,
        send_message: Option<SendClosureConnection>,
    ) -> VcxResult<Self> {
        trace!("Connection::send_request");
        let connection_sm = match &self.connection_sm {
            SmConnection::Inviter(_) => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::NotReady,
                    "Inviter cannot send connection request",
                ));
            }
            SmConnection::Invitee(sm_invitee) => SmConnection::Invitee(
                sm_invitee
                    .clone()
                    .send_connection_request(
                        routing_keys,
                        service_endpoint,
                        send_message.unwrap_or(self.send_message_closure_connection(profile)),
                    )
                    .await?,
            ),
        };
        Ok(Self { connection_sm })
    }

    pub async fn send_ack(
        self,
        profile: &Arc<dyn Profile>,
        send_message: Option<SendClosureConnection>,
    ) -> VcxResult<Self> {
        trace!("Connection::send_ack");
        let connection_sm = match &self.connection_sm {
            SmConnection::Inviter(_) => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::NotReady,
                    "Inviter cannot send ack",
                ));
            }
            SmConnection::Invitee(sm_invitee) => SmConnection::Invitee(
                sm_invitee
                    .clone()
                    .handle_send_ack(send_message.unwrap_or(self.send_message_closure_connection(profile)))
                    .await?,
            ),
        };
        Ok(Self { connection_sm })
    }

    pub async fn create_invite(self, service_endpoint: String, routing_keys: Vec<String>) -> VcxResult<Self> {
        trace!("Connection::create_invite >>>");
        let connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                SmConnection::Inviter(sm_inviter.clone().create_invitation(routing_keys, service_endpoint)?)
            }
            SmConnection::Invitee(_) => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::NotReady,
                    "Invitee cannot create invite",
                ));
            }
        };
        Ok(Self { connection_sm })
    }

    pub async fn send_generic_message(
        &self,
        profile: &Arc<dyn Profile>,
        send_message: Option<SendClosureConnection>,
        content: String,
    ) -> VcxResult<()> {
        trace!("Connection::send_generic_message >>>");
        let message = BasicMessage::create()
            .set_content(content)
            .set_time()
            .set_out_time()
            .to_a2a_message();
        let send_message = self.send_message_closure(profile, send_message).await?;
        send_message(message).await
    }

    pub async fn send_message_closure(
        &self,
        profile: &Arc<dyn Profile>,
        send_message: Option<SendClosureConnection>,
    ) -> VcxResult<SendClosure> {
        trace!("send_message_closure >>>");
        let did_doc = self.their_did_doc().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::NotReady,
            "Cannot send message: Remote Connection information is not set",
        ))?;
        let sender_vk = self.pairwise_info().pw_vk.clone();
        let send_message = send_message.unwrap_or(self.send_message_closure_connection(profile));
        Ok(Box::new(move |message: A2AMessage| {
            Box::pin(send_message(message, sender_vk.clone(), did_doc.clone()))
        }))
    }

    fn send_message_closure_connection(&self, profile: &Arc<dyn Profile>) -> SendClosureConnection {
        trace!("send_message_closure_connection >>>");
        let wallet = profile.inject_wallet();
        Box::new(move |message: A2AMessage, sender_vk: String, did_doc: AriesDidDoc| {
            Box::pin(send_message(wallet, sender_vk, did_doc, message))
        })
    }

    // ------------------------- (DE)SERIALIZATION ----------------------------------
    pub fn to_string(&self) -> VcxResult<String> {
        serde_json::to_string(&self).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::SerializationError,
                format!("Cannot serialize Connection: {:?}", err),
            )
        })
    }

    pub fn from_string(serialized: &str) -> VcxResult<Self> {
        serde_json::from_str(serialized).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Cannot deserialize Connection: {:?}", err),
            )
        })
    }
}

#[cfg(feature = "test_utils")]
#[cfg(test)]
pub mod test_utils {
    use async_channel::Sender;

    use super::*;

    pub(super) fn _routing_keys() -> Vec<String> {
        vec![]
    }

    pub(super) fn _service_endpoint() -> String {
        String::from("https://service-endpoint.org")
    }

    pub(super) fn _send_message(sender: Sender<A2AMessage>) -> Option<SendClosureConnection> {
        Some(Box::new(
            move |message: A2AMessage, _sender_vk: String, _did_doc: AriesDidDoc| {
                Box::pin(async move {
                    sender.send(message).await.map_err(|err| {
                        AriesVcxError::from_msg(
                            AriesVcxErrorKind::IOError,
                            format!("Failed to send message: {:?}", err),
                        )
                    })
                })
            },
        ))
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use crate::common::ledger::transactions::into_did_doc;
    use crate::common::test_utils::{indy_handles_to_profile, mock_profile};
    use crate::utils::devsetup::{SetupInstitutionWallet, SetupMocks};
    use crate::utils::mockdata::mockdata_connection::{
        CONNECTION_SM_INVITEE_COMPLETED, CONNECTION_SM_INVITEE_INVITED, CONNECTION_SM_INVITEE_REQUESTED,
        CONNECTION_SM_INVITEE_RESPONDED, CONNECTION_SM_INVITER_COMPLETED, CONNECTION_SM_INVITER_REQUESTED,
        CONNECTION_SM_INVITER_RESPONDED,
    };

    use async_channel::bounded;
    use messages::protocols::basic_message::message::BasicMessage;
    use messages::protocols::connection::invite::test_utils::{
        _pairwise_invitation, _pairwise_invitation_random_id, _public_invitation, _public_invitation_random_id,
    };
    use messages::protocols::connection::request::unit_tests::_request;

    use super::test_utils::*;
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_create_with_pairwise_invite() {
        let _setup = SetupMocks::init();
        let invite = Invitation::Pairwise(_pairwise_invitation());
        let connection = Connection::create_invitee(&mock_profile(), AriesDidDoc::default())
            .await
            .unwrap()
            .process_invite(invite)
            .unwrap();
        assert_eq!(connection.get_state(), ConnectionState::Invitee(InviteeState::Invited));
    }

    #[tokio::test]
    async fn test_create_with_public_invite() {
        let _setup = SetupMocks::init();
        let invite = Invitation::Public(_public_invitation());
        let connection = Connection::create_invitee(&mock_profile(), AriesDidDoc::default())
            .await
            .unwrap()
            .process_invite(invite)
            .unwrap();
        assert_eq!(connection.get_state(), ConnectionState::Invitee(InviteeState::Invited));
    }

    #[tokio::test]
    async fn test_connect_sets_correct_thread_id_based_on_invitation_type() {
        let _setup = SetupMocks::init();

        let invite = _public_invitation_random_id();
        let connection = Connection::create_invitee(&mock_profile(), AriesDidDoc::default())
            .await
            .unwrap()
            .process_invite(Invitation::Public(invite.clone()))
            .unwrap()
            .send_request(&mock_profile(), _service_endpoint(), vec![], None)
            .await
            .unwrap();
        assert_eq!(
            connection.get_state(),
            ConnectionState::Invitee(InviteeState::Requested)
        );
        assert_ne!(connection.get_thread_id(), invite.id.0);

        let invite = _pairwise_invitation_random_id();
        let connection = Connection::create_invitee(&mock_profile(), AriesDidDoc::default())
            .await
            .unwrap()
            .process_invite(Invitation::Pairwise(invite.clone()))
            .unwrap()
            .send_request(&mock_profile(), _service_endpoint(), vec![], None)
            .await
            .unwrap();
        assert_eq!(
            connection.get_state(),
            ConnectionState::Invitee(InviteeState::Requested)
        );
        assert_eq!(connection.get_thread_id(), invite.id.0);
    }

    #[tokio::test]
    async fn test_create_with_request() {
        let _setup = SetupMocks::init();

        let connection = Connection::create_inviter(&mock_profile(), None)
            .await
            .unwrap()
            .process_request(&mock_profile(), _request(), _service_endpoint(), _routing_keys(), None)
            .await
            .unwrap();

        assert_eq!(
            connection.get_state(),
            ConnectionState::Inviter(InviterState::Requested)
        );
    }

    #[tokio::test]
    async fn test_inviter_deserialize_serialized() {
        let _setup = SetupMocks::init();
        let connection = Connection::create_inviter(&mock_profile(), None)
            .await
            .unwrap()
            .process_request(&mock_profile(), _request(), _service_endpoint(), _routing_keys(), None)
            .await
            .unwrap();
        let ser_conn = connection.to_string().unwrap();
        assert_eq!(
            ser_conn,
            Connection::from_string(&ser_conn).unwrap().to_string().unwrap()
        );
    }

    #[tokio::test]
    async fn test_invitee_deserialize_serialized() {
        let _setup = SetupMocks::init();
        let invite = _pairwise_invitation_random_id();
        let connection = Connection::create_invitee(&mock_profile(), AriesDidDoc::default())
            .await
            .unwrap()
            .process_invite(Invitation::Pairwise(invite.clone()))
            .unwrap()
            .send_request(&mock_profile(), _service_endpoint(), vec![], None)
            .await
            .unwrap();
        let ser_conn = connection.to_string().unwrap();
        assert_eq!(
            ser_conn,
            Connection::from_string(&ser_conn).unwrap().to_string().unwrap()
        );
    }

    #[test]
    fn test_deserialize_and_serialize_should_produce_the_same_object() {
        fn test_deserialize_and_serialize(sm_serialized: &str) {
            let original_object: serde_json::Value = serde_json::from_str(sm_serialized).unwrap();
            let connection = Connection::from_string(sm_serialized).unwrap();
            let reserialized = connection.to_string().unwrap();
            let reserialized_object: serde_json::Value = serde_json::from_str(&reserialized).unwrap();
            assert_eq!(original_object, reserialized_object);
        }

        // let _setup = SetupMocks::init();
        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_INVITED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_REQUESTED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_RESPONDED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_COMPLETED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITER_REQUESTED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITER_RESPONDED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITER_COMPLETED);
    }

    #[tokio::test]
    async fn test_connection_e2e() {
        let setup = SetupInstitutionWallet::init().await;
        let profile = indy_handles_to_profile(setup.wallet_handle, 0);

        let (sender, receiver) = bounded(1);

        // Inviter creates connection and sends invite
        let inviter = Connection::create_inviter(&profile, None)
            .await
            .unwrap()
            .create_invite(_service_endpoint(), _routing_keys())
            .await
            .unwrap();
        let invite = if let Invitation::Pairwise(invite) = inviter.get_invite_details().unwrap().clone() {
            invite
        } else {
            panic!("Invalid invitation type");
        };

        // Invitee receives an invite and sends request
        let did_doc = into_did_doc(&mock_profile(), &Invitation::Pairwise(invite.clone()))
            .await
            .unwrap();
        let invitee = Connection::create_invitee(&profile, did_doc)
            .await
            .unwrap()
            .process_invite(Invitation::Pairwise(invite))
            .unwrap();
        assert_eq!(invitee.get_state(), ConnectionState::Invitee(InviteeState::Invited));
        let invitee = invitee
            .send_request(
                &profile,
                _service_endpoint(),
                _routing_keys(),
                _send_message(sender.clone()),
            )
            .await
            .unwrap();
        assert_eq!(invitee.get_state(), ConnectionState::Invitee(InviteeState::Requested));

        // Inviter receives requests and sends response
        let request = if let A2AMessage::ConnectionRequest(request) = receiver.recv().await.unwrap() {
            request
        } else {
            panic!("Received invalid message type")
        };

        let inviter = inviter
            .process_request(
                &profile,
                request,
                _service_endpoint(),
                _routing_keys(),
                _send_message(sender.clone()),
            )
            .await
            .unwrap();
        assert_eq!(inviter.get_state(), ConnectionState::Inviter(InviterState::Requested));
        let inviter = inviter
            .send_response(&profile, _send_message(sender.clone()))
            .await
            .unwrap();
        assert_eq!(inviter.get_state(), ConnectionState::Inviter(InviterState::Responded));

        // Invitee receives response and sends ack
        let response = if let A2AMessage::ConnectionResponse(response) = receiver.recv().await.unwrap() {
            response
        } else {
            panic!("Received invalid message type")
        };

        let invitee = invitee
            .process_response(&profile, response, _send_message(sender.clone()))
            .await
            .unwrap();
        assert_eq!(invitee.get_state(), ConnectionState::Invitee(InviteeState::Responded));
        let invitee = invitee.send_ack(&profile, _send_message(sender.clone())).await.unwrap();
        assert_eq!(invitee.get_state(), ConnectionState::Invitee(InviteeState::Completed));

        // Inviter receives an ack
        let ack = receiver.recv().await.unwrap();
        let inviter = inviter.process_ack(ack).await.unwrap();
        assert_eq!(inviter.get_state(), ConnectionState::Inviter(InviterState::Completed));

        // Invitee sends basic message
        let content = "Hello";
        let basic_message = BasicMessage::create().set_content(content.to_string()).to_a2a_message();
        invitee
            .send_message_closure(&profile, _send_message(sender.clone()))
            .await
            .unwrap()(basic_message)
        .await
        .unwrap();

        // Inviter receives basic message
        let message = if let A2AMessage::BasicMessage(message) = receiver.recv().await.unwrap() {
            message
        } else {
            panic!("Received invalid message type")
        };
        assert_eq!(message.content, content.to_string());
    }
}
