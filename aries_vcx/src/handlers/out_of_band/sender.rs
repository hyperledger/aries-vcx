use messages::{
    a2a::{message_family::MessageFamilies, message_type::MessageType, A2AMessage},
    concepts::attachment::AttachmentId,
    protocols::out_of_band::{invitation::OutOfBandInvitation, service_oob::ServiceOob, GoalCode, HandshakeProtocol},
};

use crate::errors::error::prelude::*;

#[derive(Default, Debug, PartialEq, Clone)]
pub struct OutOfBandSender {
    pub oob: OutOfBandInvitation,
}

impl OutOfBandSender {
    pub fn create() -> Self {
        Self::default()
    }

    pub fn set_label(mut self, label: &str) -> Self {
        self.oob.label = Some(label.to_string());
        self
    }

    pub fn set_goal_code(mut self, goal_code: &GoalCode) -> Self {
        self.oob.goal_code = Some(goal_code.clone());
        self
    }

    pub fn set_goal(mut self, goal: &str) -> Self {
        self.oob.goal = Some(goal.to_string());
        self
    }

    pub fn append_service(mut self, service: &ServiceOob) -> Self {
        self.oob.services.push(service.clone());
        self
    }

    pub fn get_services(&self) -> Vec<ServiceOob> {
        self.oob.services.clone()
    }

    pub fn get_id(&self) -> String {
        self.oob.id.0.clone()
    }

    pub fn append_handshake_protocol(mut self, protocol: &HandshakeProtocol) -> VcxResult<Self> {
        let new_protocol = match protocol {
            HandshakeProtocol::ConnectionV1 => MessageType::build(MessageFamilies::Connections, ""),
            HandshakeProtocol::DidExchangeV1 => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::ActionNotSupported,
                    "DidExchange protocol is not implemented".to_string(),
                ))
            }
        };
        match self.oob.handshake_protocols {
            Some(ref mut protocols) => {
                protocols.push(new_protocol);
            }
            None => {
                self.oob.handshake_protocols = Some(vec![new_protocol]);
            }
        };
        Ok(self)
    }

    pub fn append_a2a_message(mut self, msg: A2AMessage) -> VcxResult<Self> {
        let (attach_id, attach) = match msg {
            a2a_msg @ A2AMessage::PresentationRequest(_) => {
                (AttachmentId::PresentationRequest, json!(&a2a_msg).to_string())
            }
            a2a_msg @ A2AMessage::CredentialOffer(_) => (AttachmentId::CredentialOffer, json!(&a2a_msg).to_string()),
            _ => {
                error!("Appended message type {:?} is not allowed.", msg);
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidMessageFormat,
                    format!("Appended message type {:?} is not allowed.", msg),
                ));
            }
        };
        self.oob
            .requests_attach
            .add_base64_encoded_json_attachment(attach_id, ::serde_json::Value::String(attach))?;
        Ok(self)
    }

    pub fn to_a2a_message(&self) -> A2AMessage {
        self.oob.to_a2a_message()
    }

    pub fn to_string(&self) -> String {
        self.oob.to_string()
    }

    pub fn from_string(oob_data: &str) -> VcxResult<Self> {
        Ok(Self {
            oob: OutOfBandInvitation::from_string(oob_data)?,
        })
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use messages::{
        diddoc::aries::service::AriesService,
        protocols::{connection::did::Did, issuance::credential_offer::CredentialOffer},
    };

    use super::*;
    use crate::utils::devsetup::SetupMocks;

    fn _create_oob() -> OutOfBandSender {
        OutOfBandSender::create()
            .set_label("oob-label")
            .set_goal("issue-vc")
            .set_goal_code(&GoalCode::IssueVC)
    }

    fn _create_service() -> ServiceOob {
        ServiceOob::AriesService(
            AriesService::create()
                .set_service_endpoint("http://example.org/agent".into())
                .set_routing_keys(vec!["12345".into()])
                .set_recipient_keys(vec!["abcde".into()]),
        )
    }

    #[test]
    fn test_append_aries_service_object_to_oob_services() {
        let _setup = SetupMocks::init();

        let service = _create_service();
        let oob = _create_oob().append_service(&service);
        let resolved_service = oob.get_services();

        assert_eq!(resolved_service.len(), 1);
        assert_eq!(service, resolved_service[0]);
    }

    #[test]
    fn test_append_did_service_object_to_oob_services() {
        let _setup = SetupMocks::init();

        let service = ServiceOob::Did(Did::new("V4SGRU86Z58d6TV7PBUe6f").unwrap());
        let oob = _create_oob().append_service(&service);
        let resolved_service = oob.get_services();

        assert_eq!(resolved_service.len(), 1);
        assert_eq!(service, resolved_service[0]);
    }

    #[test]
    fn test_oob_sender_to_a2a_message() {
        let _setup = SetupMocks::init();

        let inserted_offer = CredentialOffer::create();
        let basic_msg = A2AMessage::CredentialOffer(inserted_offer.clone());
        let oob = _create_oob().append_a2a_message(basic_msg).unwrap();
        let oob_msg = oob.to_a2a_message();
        assert!(matches!(oob_msg, A2AMessage::OutOfBandInvitation(..)));
        if let A2AMessage::OutOfBandInvitation(oob_msg) = oob_msg {
            let attachment = oob_msg.requests_attach.content().unwrap();
            let attachment: A2AMessage = serde_json::from_str(&attachment).unwrap();
            assert!(matches!(attachment, A2AMessage::CredentialOffer(..)));
            if let A2AMessage::CredentialOffer(offer) = attachment {
                assert_eq!(offer, inserted_offer)
            }
        }
    }
}
