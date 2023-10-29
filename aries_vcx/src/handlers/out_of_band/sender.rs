use std::fmt::Display;

use messages::{
    msg_fields::protocols::{
        cred_issuance::{v1::CredentialIssuanceV1, CredentialIssuance},
        out_of_band::{
            invitation::{Invitation, InvitationContent, InvitationDecorators, OobService},
            OobGoalCode,
        },
        present_proof::{v1::PresentProofV1, PresentProof},
    },
    msg_types::Protocol,
    AriesMessage,
};
use shared_vcx::maybe_known::MaybeKnown;
use uuid::Uuid;

use crate::{
    errors::error::prelude::*,
    handlers::util::{make_attach_from_str, AttachmentId},
};

#[derive(Debug, PartialEq, Clone)]
pub struct OutOfBandSender {
    pub oob: Invitation,
}

impl OutOfBandSender {
    pub fn create() -> Self {
        let id = Uuid::new_v4().to_string();
        let content = InvitationContent::builder().services(Vec::new()).build();
        let decorators = InvitationDecorators::default();

        Self {
            oob: Invitation::builder()
                .id(id)
                .content(content)
                .decorators(decorators)
                .build(),
        }
    }

    pub fn set_label(mut self, label: &str) -> Self {
        self.oob.content.label = Some(label.to_string());
        self
    }

    pub fn set_goal_code(mut self, goal_code: OobGoalCode) -> Self {
        self.oob.content.goal_code = Some(MaybeKnown::Known(goal_code));
        self
    }

    pub fn set_goal(mut self, goal: &str) -> Self {
        self.oob.content.goal = Some(goal.to_string());
        self
    }

    pub fn append_service(mut self, service: &OobService) -> Self {
        self.oob.content.services.push(service.clone());
        self
    }

    pub fn get_services(&self) -> Vec<OobService> {
        self.oob.content.services.clone()
    }

    pub fn get_id(&self) -> String {
        self.oob.id.clone()
    }

    pub fn append_handshake_protocol(mut self, protocol: Protocol) -> VcxResult<Self> {
        let new_protocol = match protocol {
            Protocol::ConnectionType(_) => MaybeKnown::Known(protocol),
            _ => {
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::ActionNotSupported,
                    "Protocol not supported".to_string(),
                ))
            }
        };

        match self.oob.content.handshake_protocols {
            Some(ref mut protocols) => {
                protocols.push(new_protocol);
            }
            None => {
                self.oob.content.handshake_protocols = Some(vec![new_protocol]);
            }
        };
        Ok(self)
    }

    pub fn append_a2a_message(mut self, msg: AriesMessage) -> VcxResult<Self> {
        let (attach_id, attach) = match msg {
            a2a_msg @ AriesMessage::PresentProof(PresentProof::V1(
                PresentProofV1::RequestPresentation(_),
            )) => (
                AttachmentId::PresentationRequest,
                json!(&a2a_msg).to_string(),
            ),
            a2a_msg @ AriesMessage::CredentialIssuance(CredentialIssuance::V1(
                CredentialIssuanceV1::OfferCredential(_),
            )) => (AttachmentId::CredentialOffer, json!(&a2a_msg).to_string()),
            _ => {
                error!("Appended message type {:?} is not allowed.", msg);
                return Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidMessageFormat,
                    format!("Appended message type {:?} is not allowed.", msg),
                ));
            }
        };

        self.oob
            .content
            .requests_attach
            .get_or_insert(Vec::with_capacity(1))
            .push(make_attach_from_str!(
                &attach,
                attach_id.as_ref().to_string()
            ));

        Ok(self)
    }

    pub fn to_aries_message(&self) -> AriesMessage {
        self.oob.clone().into()
    }

    pub fn from_string(oob_data: &str) -> VcxResult<Self> {
        Ok(Self {
            oob: serde_json::from_str(oob_data)?,
        })
    }
}

impl Display for OutOfBandSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", json!(AriesMessage::from(self.oob.clone())))
    }
}
