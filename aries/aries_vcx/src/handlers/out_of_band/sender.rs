use std::fmt::Display;

use base64::Engine;
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
use shared::maybe_known::MaybeKnown;
use url::Url;
use uuid::Uuid;

use crate::{
    errors::error::prelude::*,
    handlers::util::{make_attach_from_str, AttachmentId},
    utils::base64::URL_SAFE_LENIENT,
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

    pub fn create_from_invitation(invitation: Invitation) -> Self {
        Self { oob: invitation }
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
            Protocol::ConnectionType(_) | Protocol::DidExchangeType(_) => {
                MaybeKnown::Known(protocol)
            }
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

    pub fn invitation_to_aries_message(&self) -> AriesMessage {
        self.oob.clone().into()
    }

    pub fn invitation_to_json_string(&self) -> String {
        self.invitation_to_aries_message().to_string()
    }

    fn invitation_to_base64_url(&self) -> String {
        URL_SAFE_LENIENT.encode(self.invitation_to_json_string())
    }

    pub fn invitation_to_url(&self, domain_path: &str) -> VcxResult<Url> {
        let oob_url = Url::parse(domain_path)?
            .query_pairs_mut()
            .append_pair("oob", &self.invitation_to_base64_url())
            .finish()
            .to_owned();
        Ok(oob_url)
    }
}

impl Display for OutOfBandSender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", json!(AriesMessage::from(self.oob.clone())))
    }
}

#[cfg(test)]
mod tests {
    use messages::{
        msg_fields::protocols::out_of_band::{
            invitation::{Invitation, InvitationContent, InvitationDecorators, OobService},
            OobGoalCode,
        },
        msg_types::{
            connection::{ConnectionType, ConnectionTypeV1},
            protocols::did_exchange::{DidExchangeType, DidExchangeTypeV1},
            Protocol,
        },
    };
    use shared::maybe_known::MaybeKnown;

    use super::*;

    // Example invite formats referenced (with change to use OOB 1.1) from example invite in RFC 0434 - https://github.com/hyperledger/aries-rfcs/tree/main/features/0434-outofband
    const JSON_OOB_INVITE_NO_WHITESPACE: &str = r#"{"@type":"https://didcomm.org/out-of-band/1.1/invitation","@id":"69212a3a-d068-4f9d-a2dd-4741bca89af3","label":"Faber College","goal_code":"issue-vc","goal":"To issue a Faber College Graduate credential","handshake_protocols":["https://didcomm.org/didexchange/1.0","https://didcomm.org/connections/1.0"],"services":["did:sov:LjgpST2rjsoxYegQDRm7EL"]}"#;
    const OOB_BASE64_URL_ENCODED: &str = "eyJAdHlwZSI6Imh0dHBzOi8vZGlkY29tbS5vcmcvb3V0LW9mLWJhbmQvMS4xL2ludml0YXRpb24iLCJAaWQiOiI2OTIxMmEzYS1kMDY4LTRmOWQtYTJkZC00NzQxYmNhODlhZjMiLCJsYWJlbCI6IkZhYmVyIENvbGxlZ2UiLCJnb2FsX2NvZGUiOiJpc3N1ZS12YyIsImdvYWwiOiJUbyBpc3N1ZSBhIEZhYmVyIENvbGxlZ2UgR3JhZHVhdGUgY3JlZGVudGlhbCIsImhhbmRzaGFrZV9wcm90b2NvbHMiOlsiaHR0cHM6Ly9kaWRjb21tLm9yZy9kaWRleGNoYW5nZS8xLjAiLCJodHRwczovL2RpZGNvbW0ub3JnL2Nvbm5lY3Rpb25zLzEuMCJdLCJzZXJ2aWNlcyI6WyJkaWQ6c292OkxqZ3BTVDJyanNveFllZ1FEUm03RUwiXX0";
    const OOB_URL: &str = "http://example.com/ssi?oob=eyJAdHlwZSI6Imh0dHBzOi8vZGlkY29tbS5vcmcvb3V0LW9mLWJhbmQvMS4xL2ludml0YXRpb24iLCJAaWQiOiI2OTIxMmEzYS1kMDY4LTRmOWQtYTJkZC00NzQxYmNhODlhZjMiLCJsYWJlbCI6IkZhYmVyIENvbGxlZ2UiLCJnb2FsX2NvZGUiOiJpc3N1ZS12YyIsImdvYWwiOiJUbyBpc3N1ZSBhIEZhYmVyIENvbGxlZ2UgR3JhZHVhdGUgY3JlZGVudGlhbCIsImhhbmRzaGFrZV9wcm90b2NvbHMiOlsiaHR0cHM6Ly9kaWRjb21tLm9yZy9kaWRleGNoYW5nZS8xLjAiLCJodHRwczovL2RpZGNvbW0ub3JnL2Nvbm5lY3Rpb25zLzEuMCJdLCJzZXJ2aWNlcyI6WyJkaWQ6c292OkxqZ3BTVDJyanNveFllZ1FEUm03RUwiXX0";

    // Params mimic example invitation in RFC 0434 - https://github.com/hyperledger/aries-rfcs/tree/main/features/0434-outofband
    fn _create_invitation() -> Invitation {
        let id = "69212a3a-d068-4f9d-a2dd-4741bca89af3";
        let did = "did:sov:LjgpST2rjsoxYegQDRm7EL";
        let service = OobService::Did(did.to_string());
        let handshake_protocols = vec![
            MaybeKnown::Known(Protocol::DidExchangeType(DidExchangeType::V1(
                DidExchangeTypeV1::new_v1_0(),
            ))),
            MaybeKnown::Known(Protocol::ConnectionType(ConnectionType::V1(
                ConnectionTypeV1::new_v1_0(),
            ))),
        ];
        let content = InvitationContent::builder()
            .services(vec![service])
            .goal("To issue a Faber College Graduate credential".to_string())
            .goal_code(MaybeKnown::Known(OobGoalCode::IssueVC))
            .label("Faber College".to_string())
            .handshake_protocols(handshake_protocols)
            .build();
        let decorators = InvitationDecorators::default();

        let invitation: Invitation = Invitation::builder()
            .id(id.to_string())
            .content(content)
            .decorators(decorators)
            .build();

        invitation
    }

    #[test]
    fn invitation_to_json() {
        let out_of_band_sender = OutOfBandSender::create_from_invitation(_create_invitation());

        let json_invite = out_of_band_sender.invitation_to_json_string();

        assert_eq!(JSON_OOB_INVITE_NO_WHITESPACE, json_invite);
    }

    #[test]
    fn invitation_to_base64_url() {
        let out_of_band_sender = OutOfBandSender::create_from_invitation(_create_invitation());

        let base64_url_invite = out_of_band_sender.invitation_to_base64_url();

        assert_eq!(OOB_BASE64_URL_ENCODED, base64_url_invite);
    }

    #[test]
    fn invitation_to_url() {
        let out_of_band_sender = OutOfBandSender::create_from_invitation(_create_invitation());

        let oob_url = out_of_band_sender
            .invitation_to_url("http://example.com/ssi")
            .unwrap()
            .to_string();

        assert_eq!(OOB_URL, oob_url);
    }
}

// #[cfg(test)]
// mod unit_tests {
//     use crate::utils::devsetup::SetupMocks;
//     use messages::diddoc::aries::service::AriesService;
//     use messages::protocols::connection::did::Did;
//     use messages::protocols::issuance::credential_offer::CredentialOffer;
//
//     use super::*;
//
//     fn _create_oob() -> OutOfBandSender {
//         OutOfBandSender::create()
//             .set_label("oob-label")
//             .set_goal("issue-vc")
//             .set_goal_code(&GoalCode::IssueVC)
//     }
//
//     fn _create_service() -> ServiceOob {
//         ServiceOob::AriesService(
//             AriesService::create()
//                 .set_service_endpoint("http://example.org/agent".into())
//                 .set_routing_keys(vec!["12345".into()])
//                 .set_recipient_keys(vec!["abcde".into()]),
//         )
//     }
//
//     #[test]
//     fn test_append_aries_service_object_to_oob_services() {
//         let _setup = SetupMocks::init();
//
//         let service = _create_service();
//         let oob = _create_oob().append_service(&service);
//         let resolved_service = oob.get_services();
//
//         assert_eq!(resolved_service.len(), 1);
//         assert_eq!(service, resolved_service[0]);
//     }
//
//     #[test]
//     fn test_append_did_service_object_to_oob_services() {
//         let _setup = SetupMocks::init();
//
//         let service = ServiceOob::Did(Did::new("V4SGRU86Z58d6TV7PBUe6f").unwrap());
//         let oob = _create_oob().append_service(&service);
//         let resolved_service = oob.get_services();
//
//         assert_eq!(resolved_service.len(), 1);
//         assert_eq!(service, resolved_service[0]);
//     }
//
//     #[test]
//     fn test_oob_sender_to_a2a_message() {
//         let _setup = SetupMocks::init();
//
//         let inserted_offer = CredentialOffer::create();
//         let basic_msg = A2AMessage::CredentialOffer(inserted_offer.clone());
//         let oob = _create_oob().append_a2a_message(basic_msg).unwrap();
//         let oob_msg = oob.to_a2a_message();
//         assert!(matches!(oob_msg, A2AMessage::OutOfBandInvitation(..)));
//         if let A2AMessage::OutOfBandInvitation(oob_msg) = oob_msg {
//             let attachment = oob_msg.requests_attach.content().unwrap();
//             let attachment: A2AMessage = serde_json::from_str(&attachment).unwrap();
//             assert!(matches!(attachment, A2AMessage::CredentialOffer(..)));
//             if let A2AMessage::CredentialOffer(offer) = attachment {
//                 assert_eq!(offer, inserted_offer)
//             }
//         }
//     }
// }
