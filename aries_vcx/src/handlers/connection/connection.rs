use std::clone::Clone;
use std::sync::Arc;

use messages::a2a::A2AMessage;
use messages::connection::response::SignedResponse;
use serde::{Deserialize, Serialize};

use crate::core::profile::profile::Profile;
use crate::error::prelude::*;
use crate::protocols::connection::invitee::state_machine::{InviteeFullState, InviteeState, SmConnectionInvitee};
use crate::protocols::connection::inviter::state_machine::{InviterFullState, InviterState, SmConnectionInviter};
use crate::protocols::connection::pairwise_info::PairwiseInfo;
use crate::protocols::{SendClosure, SendClosureConnection};
use crate::utils::send_message;
use messages::connection::invite::Invitation;
use messages::connection::request::Request;
use messages::did_doc::DidDoc;

#[derive(Clone, PartialEq)]
pub struct Connection {
    connection_sm: SmConnection,
}

#[derive(Clone, PartialEq)]
pub enum SmConnection {
    Inviter(SmConnectionInviter),
    Invitee(SmConnectionInvitee),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmConnectionState {
    Inviter(InviterFullState),
    Invitee(InviteeFullState),
}

#[derive(Debug, PartialEq)]
pub enum ConnectionState {
    Inviter(InviterState),
    Invitee(InviteeState),
}

impl Connection {
    // ----------------------------- CONSTRUCTORS ------------------------------------
    pub async fn create_inviter(profile: &Arc<dyn Profile>) -> VcxResult<Self> {
        trace!("Connection::create >>>");
        let pairwise_info = PairwiseInfo::create(&profile.inject_wallet()).await?;
        Ok(Self {
            connection_sm: SmConnection::Inviter(SmConnectionInviter::new("", pairwise_info)),
        })
    }

    pub async fn create_invitee(profile: &Arc<dyn Profile>, did_doc: DidDoc) -> VcxResult<Self> {
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

    pub fn their_did_doc(&self) -> Option<DidDoc> {
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
                return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid action"));
            }
            SmConnection::Invitee(sm_invitee) => {
                SmConnection::Invitee(sm_invitee.clone().handle_invitation(invitation)?)
            }
        };
        Ok(Self { connection_sm, ..self })
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
                return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid action"));
            }
        };
        Ok(Self { connection_sm, ..self })
    }

    pub async fn process_response(
        self,
        profile: &Arc<dyn Profile>,
        response: SignedResponse,
        send_message: Option<SendClosureConnection>,
    ) -> VcxResult<Self> {
        let connection_sm = match &self.connection_sm {
            SmConnection::Inviter(_) => {
                return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid action"));
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
        Ok(Self { connection_sm, ..self })
    }

    pub async fn process_ack(self, message: A2AMessage) -> VcxResult<Self> {
        trace!("Connection::process_ack >>> message: {:?}", message);
        let connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                SmConnection::Inviter(sm_inviter.clone().handle_confirmation_message(&message).await?)
            }
            SmConnection::Invitee(_) => {
                return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid action"));
            }
        };
        Ok(Self { connection_sm, ..self })
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
                    return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid action"));
                }
            }
            SmConnection::Invitee(_) => {
                return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Invalid action"));
            }
        };
        Ok(Self { connection_sm, ..self })
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
                return Err(VcxError::from_msg(
                    VcxErrorKind::NotReady,
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
        Ok(Self { connection_sm, ..self })
    }

    pub async fn send_ack(
        self,
        profile: &Arc<dyn Profile>,
        send_message: Option<SendClosureConnection>,
    ) -> VcxResult<Self> {
        trace!("Connection::send_request");
        let connection_sm = match &self.connection_sm {
            SmConnection::Inviter(_) => {
                return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Inviter cannot send ack"));
            }
            SmConnection::Invitee(sm_invitee) => SmConnection::Invitee(
                sm_invitee
                    .clone()
                    .handle_send_ack(send_message.unwrap_or(self.send_message_closure_connection(profile)))
                    .await?,
            ),
        };
        Ok(Self { connection_sm, ..self })
    }

    pub async fn create_invite(self, service_endpoint: String, routing_keys: Vec<String>) -> VcxResult<Self> {
        trace!("Connection::create_invite >>>");
        let connection_sm = match &self.connection_sm {
            SmConnection::Inviter(sm_inviter) => {
                SmConnection::Inviter(sm_inviter.clone().create_invitation(routing_keys, service_endpoint)?)
            }
            SmConnection::Invitee(_) => {
                return Err(VcxError::from_msg(
                    VcxErrorKind::NotReady,
                    "Invitee cannot create invite",
                ));
            }
        };
        Ok(Self { connection_sm, ..self })
    }

    pub async fn send_message_closure(
        &self,
        profile: &Arc<dyn Profile>,
        send_message: Option<SendClosureConnection>,
    ) -> VcxResult<SendClosure> {
        trace!("send_message_closure >>>");
        let did_doc = self.their_did_doc().ok_or(VcxError::from_msg(
            VcxErrorKind::NotReady,
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
        Box::new(move |message: A2AMessage, sender_vk: String, did_doc: DidDoc| {
            Box::pin(send_message(wallet, sender_vk, did_doc, message))
        })
    }
}

#[cfg(feature = "test_utils")]
#[cfg(test)]
pub mod test_utils {
    use async_channel::Sender;
    use vdrtools::PoolHandle;

    use super::*;

    pub(super) fn _pool_handle() -> PoolHandle {
        0
    }

    pub(super) fn _routing_keys() -> Vec<String> {
        vec![]
    }

    pub(super) fn _service_endpoint() -> String {
        String::from("https://service-endpoint.org")
    }

    pub(super) fn _send_message(sender: Sender<A2AMessage>) -> Option<SendClosureConnection> {
        Some(Box::new(
            move |message: A2AMessage, _sender_vk: String, _did_doc: DidDoc| {
                Box::pin(async move {
                    sender.send(message).await.map_err(|err| {
                        VcxError::from_msg(VcxErrorKind::IOError, format!("Failed to send message: {:?}", err))
                    })
                })
            },
        ))
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use crate::xyz::ledger::transactions::into_did_doc;
    use crate::utils::devsetup::{SetupMocks, SetupProfile};
    use crate::xyz::test_utils::mock_profile;

    use async_channel::bounded;
    use messages::basic_message::message::BasicMessage;
    use messages::connection::invite::test_utils::{
        _pairwise_invitation, _pairwise_invitation_random_id, _public_invitation, _public_invitation_random_id,
    };
    use messages::connection::request::unit_tests::_request;

    use super::test_utils::*;
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_create_with_pairwise_invite() {
        let _setup = SetupMocks::init();
        let invite = Invitation::Pairwise(_pairwise_invitation());
        let connection = Connection::create_invitee(&mock_profile(), DidDoc::default())
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
        let connection = Connection::create_invitee(&mock_profile(), DidDoc::default())
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
        let connection = Connection::create_invitee(&mock_profile(), DidDoc::default())
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
        let connection = Connection::create_invitee(&mock_profile(), DidDoc::default())
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

        let connection = Connection::create_inviter(&mock_profile())
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

    // TODO - consider moving test, as this isn't really a unit test
    #[tokio::test]
    async fn test_connection_e2e() {
        SetupProfile::run(|setup| async move {

        let (sender, receiver) = bounded(1);

        // Inviter creates connection and sends invite
        let inviter = Connection::create_inviter(&setup.profile)
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
        let invitee = Connection::create_invitee(&setup.profile, did_doc)
            .await
            .unwrap()
            .process_invite(Invitation::Pairwise(invite))
            .unwrap();
        assert_eq!(invitee.get_state(), ConnectionState::Invitee(InviteeState::Invited));
        let invitee = invitee
            .send_request(
                &setup.profile,
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
                &setup.profile,
                request,
                _service_endpoint(),
                _routing_keys(),
                _send_message(sender.clone()),
            )
            .await
            .unwrap();
        assert_eq!(inviter.get_state(), ConnectionState::Inviter(InviterState::Requested));
        let inviter = inviter
            .send_response(&setup.profile, _send_message(sender.clone()))
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
            .process_response(&setup.profile, response, _send_message(sender.clone()))
            .await
            .unwrap();
        assert_eq!(invitee.get_state(), ConnectionState::Invitee(InviteeState::Responded));
        let invitee = invitee
            .send_ack(&setup.profile, _send_message(sender.clone()))
            .await
            .unwrap();
        assert_eq!(invitee.get_state(), ConnectionState::Invitee(InviteeState::Completed));

        // Inviter receives an ack
        let ack = receiver.recv().await.unwrap();
        let inviter = inviter.process_ack(ack).await.unwrap();
        assert_eq!(inviter.get_state(), ConnectionState::Inviter(InviterState::Completed));

        // Invitee sends basic message
        let content = "Hello";
        let basic_message = BasicMessage::create().set_content(content.to_string()).to_a2a_message();
        invitee
            .send_message_closure(&setup.profile, _send_message(sender.clone()))
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
        }).await;
    }
}
