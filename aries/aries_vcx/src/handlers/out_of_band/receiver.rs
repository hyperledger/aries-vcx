use std::{clone::Clone, fmt::Display, str::FromStr};

use base64::{engine::general_purpose, Engine};
use messages::{
    decorators::attachment::{Attachment, AttachmentType},
    msg_fields::protocols::{
        cred_issuance::v1::offer_credential::OfferCredentialV1,
        out_of_band::{invitation::Invitation, OutOfBand},
        present_proof::v1::request::RequestPresentationV1,
    },
    AriesMessage,
};
use serde::Deserialize;
use serde_json::Value;
use url::Url;

use crate::{
    errors::error::prelude::*, handlers::util::AttachmentId, utils::base64::URL_SAFE_LENIENT,
};

#[derive(Debug, PartialEq, Clone)]
pub struct OutOfBandReceiver {
    pub oob: Invitation,
}

impl OutOfBandReceiver {
    pub fn create_from_a2a_msg(msg: &AriesMessage) -> VcxResult<Self> {
        trace!("OutOfBandReceiver::create_from_a2a_msg >>> msg: {:?}", msg);
        match msg {
            AriesMessage::OutOfBand(OutOfBand::Invitation(oob)) => {
                Ok(OutOfBandReceiver { oob: oob.clone() })
            }
            m => Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidMessageFormat,
                format!(
                    "Expected OutOfBandInvitation message to create OutOfBandReceiver, but \
                     received message of unknown type: {:?}",
                    m
                ),
            )),
        }
    }

    pub fn create_from_json_encoded_oob(oob_json: &str) -> VcxResult<Self> {
        Ok(Self {
            oob: extract_encoded_invitation_from_json_string(oob_json)?,
        })
    }

    pub fn create_from_url_encoded_oob(oob_url_string: &str) -> VcxResult<Self> {
        // TODO - URL Shortening
        Ok(Self {
            oob: extract_encoded_invitation_from_json_string(
                &extract_encoded_invitation_from_base64_url(&extract_encoded_invitation_from_url(
                    oob_url_string,
                )?)?,
            )?,
        })
    }

    pub fn get_id(&self) -> String {
        self.oob.id.clone()
    }

    // TODO: There may be multiple A2AMessages in a single OoB msg
    pub fn extract_a2a_message(&self) -> VcxResult<Option<AriesMessage>> {
        trace!("OutOfBandReceiver::extract_a2a_message >>>");
        if let Some(attach) = self
            .oob
            .content
            .requests_attach
            .as_ref()
            .and_then(|v| v.first())
        {
            attachment_to_aries_message(attach)
        } else {
            Ok(None)
        }
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

fn extract_encoded_invitation_from_json_string(oob_json: &str) -> VcxResult<Invitation> {
    Ok(serde_json::from_str(oob_json)?)
}

fn extract_encoded_invitation_from_base64_url(base64_url_encoded_oob: &str) -> VcxResult<String> {
    Ok(String::from_utf8(
        URL_SAFE_LENIENT.decode(base64_url_encoded_oob)?,
    )?)
}

fn extract_encoded_invitation_from_url(oob_url_string: &str) -> VcxResult<String> {
    let oob_url = Url::parse(oob_url_string)?;
    let (_oob_query, base64_url_encoded_oob) = oob_url
        .query_pairs()
        .find(|(name, _value)| name == "oob")
        .ok_or_else(|| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidInput,
                "OutOfBand Invitation URL is missing 'oob' query parameter",
            )
        })?;

    Ok(base64_url_encoded_oob.into_owned())
}

impl Display for OutOfBandReceiver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", json!(AriesMessage::from(self.oob.clone())))
    }
}

fn attachment_to_aries_message(attach: &Attachment) -> VcxResult<Option<AriesMessage>> {
    let AttachmentType::Base64(encoded_attach) = &attach.data.content else {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!("Attachment is not base 64 encoded JSON: {attach:?}"),
        ));
    };

    let Ok(bytes) = general_purpose::STANDARD.decode(encoded_attach) else {
        return Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!("Attachment is not base 64 encoded JSON: {attach:?}"),
        ));
    };

    let attach_json: Value = serde_json::from_slice(&bytes).map_err(|_| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!("Attachment is not base 64 encoded JSON: {attach:?}"),
        )
    })?;

    let attach_id = if let Some(attach_id) = attach.id.as_deref() {
        AttachmentId::from_str(attach_id).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::SerializationError,
                format!("Failed to deserialize attachment ID: {}", err),
            )
        })
    } else {
        Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidMessageFormat,
            format!("Missing attachment ID on attach: {attach:?}"),
        ))
    }?;

    match attach_id {
        AttachmentId::CredentialOffer => {
            let offer = OfferCredentialV1::deserialize(&attach_json).map_err(|_| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::SerializationError,
                    format!("Failed to deserialize attachment: {attach_json:?}"),
                )
            })?;
            Ok(Some(offer.into()))
        }
        AttachmentId::PresentationRequest => {
            let request = RequestPresentationV1::deserialize(&attach_json).map_err(|_| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::SerializationError,
                    format!("Failed to deserialize attachment: {attach_json:?}"),
                )
            })?;
            Ok(Some(request.into()))
        }
        _ => Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidMessageFormat,
            format!("unexpected attachment type: {:?}", attach_id),
        )),
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
    const JSON_OOB_INVITE: &str = r#"{
        "@type": "https://didcomm.org/out-of-band/1.1/invitation",
        "@id": "69212a3a-d068-4f9d-a2dd-4741bca89af3",
        "label": "Faber College",
        "goal_code": "issue-vc",
        "goal": "To issue a Faber College Graduate credential",
        "handshake_protocols": ["https://didcomm.org/didexchange/1.0", "https://didcomm.org/connections/1.0"],
        "services": ["did:sov:LjgpST2rjsoxYegQDRm7EL"]
      }"#;
    const JSON_OOB_INVITE_NO_WHITESPACE: &str = r#"{"@type":"https://didcomm.org/out-of-band/1.1/invitation","@id":"69212a3a-d068-4f9d-a2dd-4741bca89af3","label":"Faber College","goal_code":"issue-vc","goal":"To issue a Faber College Graduate credential","handshake_protocols":["https://didcomm.org/didexchange/1.0","https://didcomm.org/connections/1.0"],"services":["did:sov:LjgpST2rjsoxYegQDRm7EL"]}"#;
    const OOB_BASE64_URL_ENCODED: &str = "eyJAdHlwZSI6Imh0dHBzOi8vZGlkY29tbS5vcmcvb3V0LW9mLWJhbmQvMS4xL2ludml0YXRpb24iLCJAaWQiOiI2OTIxMmEzYS1kMDY4LTRmOWQtYTJkZC00NzQxYmNhODlhZjMiLCJsYWJlbCI6IkZhYmVyIENvbGxlZ2UiLCJnb2FsX2NvZGUiOiJpc3N1ZS12YyIsImdvYWwiOiJUbyBpc3N1ZSBhIEZhYmVyIENvbGxlZ2UgR3JhZHVhdGUgY3JlZGVudGlhbCIsImhhbmRzaGFrZV9wcm90b2NvbHMiOlsiaHR0cHM6Ly9kaWRjb21tLm9yZy9kaWRleGNoYW5nZS8xLjAiLCJodHRwczovL2RpZGNvbW0ub3JnL2Nvbm5lY3Rpb25zLzEuMCJdLCJzZXJ2aWNlcyI6WyJkaWQ6c292OkxqZ3BTVDJyanNveFllZ1FEUm03RUwiXX0";
    const OOB_URL: &str = "http://example.com/ssi?oob=eyJAdHlwZSI6Imh0dHBzOi8vZGlkY29tbS5vcmcvb3V0LW9mLWJhbmQvMS4xL2ludml0YXRpb24iLCJAaWQiOiI2OTIxMmEzYS1kMDY4LTRmOWQtYTJkZC00NzQxYmNhODlhZjMiLCJsYWJlbCI6IkZhYmVyIENvbGxlZ2UiLCJnb2FsX2NvZGUiOiJpc3N1ZS12YyIsImdvYWwiOiJUbyBpc3N1ZSBhIEZhYmVyIENvbGxlZ2UgR3JhZHVhdGUgY3JlZGVudGlhbCIsImhhbmRzaGFrZV9wcm90b2NvbHMiOlsiaHR0cHM6Ly9kaWRjb21tLm9yZy9kaWRleGNoYW5nZS8xLjAiLCJodHRwczovL2RpZGNvbW0ub3JnL2Nvbm5lY3Rpb25zLzEuMCJdLCJzZXJ2aWNlcyI6WyJkaWQ6c292OkxqZ3BTVDJyanNveFllZ1FEUm03RUwiXX0";
    const OOB_URL_WITH_PADDING: &str = "http://example.com/ssi?oob=eyJAdHlwZSI6Imh0dHBzOi8vZGlkY29tbS5vcmcvb3V0LW9mLWJhbmQvMS4xL2ludml0YXRpb24iLCJAaWQiOiI2OTIxMmEzYS1kMDY4LTRmOWQtYTJkZC00NzQxYmNhODlhZjMiLCJsYWJlbCI6IkZhYmVyIENvbGxlZ2UiLCJnb2FsX2NvZGUiOiJpc3N1ZS12YyIsImdvYWwiOiJUbyBpc3N1ZSBhIEZhYmVyIENvbGxlZ2UgR3JhZHVhdGUgY3JlZGVudGlhbCIsImhhbmRzaGFrZV9wcm90b2NvbHMiOlsiaHR0cHM6Ly9kaWRjb21tLm9yZy9kaWRleGNoYW5nZS8xLjAiLCJodHRwczovL2RpZGNvbW0ub3JnL2Nvbm5lY3Rpb25zLzEuMCJdLCJzZXJ2aWNlcyI6WyJkaWQ6c292OkxqZ3BTVDJyanNveFllZ1FEUm03RUwiXX0%3D";
    const OOB_URL_WITH_PADDING_NOT_PERCENT_ENCODED: &str = "http://example.com/ssi?oob=eyJAdHlwZSI6Imh0dHBzOi8vZGlkY29tbS5vcmcvb3V0LW9mLWJhbmQvMS4xL2ludml0YXRpb24iLCJAaWQiOiI2OTIxMmEzYS1kMDY4LTRmOWQtYTJkZC00NzQxYmNhODlhZjMiLCJsYWJlbCI6IkZhYmVyIENvbGxlZ2UiLCJnb2FsX2NvZGUiOiJpc3N1ZS12YyIsImdvYWwiOiJUbyBpc3N1ZSBhIEZhYmVyIENvbGxlZ2UgR3JhZHVhdGUgY3JlZGVudGlhbCIsImhhbmRzaGFrZV9wcm90b2NvbHMiOlsiaHR0cHM6Ly9kaWRjb21tLm9yZy9kaWRleGNoYW5nZS8xLjAiLCJodHRwczovL2RpZGNvbW0ub3JnL2Nvbm5lY3Rpb25zLzEuMCJdLCJzZXJ2aWNlcyI6WyJkaWQ6c292OkxqZ3BTVDJyanNveFllZ1FEUm03RUwiXX0=";

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
    fn receive_invitation_by_json() {
        let base_invite = _create_invitation();
        let parsed_invite = OutOfBandReceiver::create_from_json_encoded_oob(JSON_OOB_INVITE)
            .unwrap()
            .oob;
        assert_eq!(base_invite, parsed_invite);
    }

    #[test]
    fn receive_invitation_by_json_no_whitespace() {
        let base_invite = _create_invitation();
        let parsed_invite =
            OutOfBandReceiver::create_from_json_encoded_oob(JSON_OOB_INVITE_NO_WHITESPACE)
                .unwrap()
                .oob;
        assert_eq!(base_invite, parsed_invite);
    }

    #[test]
    fn receive_invitation_by_url() {
        let base_invite = _create_invitation();
        let parsed_invite = OutOfBandReceiver::create_from_url_encoded_oob(OOB_URL)
            .unwrap()
            .oob;
        assert_eq!(base_invite, parsed_invite);
    }

    #[test]
    fn receive_invitation_by_url_with_padding() {
        let base_invite = _create_invitation();
        let parsed_invite = OutOfBandReceiver::create_from_url_encoded_oob(OOB_URL_WITH_PADDING)
            .unwrap()
            .oob;
        assert_eq!(base_invite, parsed_invite);
    }

    #[test]
    fn receive_invitation_by_url_with_padding_no_percent_encoding() {
        let base_invite = _create_invitation();
        let parsed_invite = OutOfBandReceiver::create_from_url_encoded_oob(
            OOB_URL_WITH_PADDING_NOT_PERCENT_ENCODED,
        )
        .unwrap()
        .oob;
        assert_eq!(base_invite, parsed_invite);
    }

    #[test]
    fn invitation_to_json() {
        let out_of_band_receiver =
            OutOfBandReceiver::create_from_json_encoded_oob(JSON_OOB_INVITE).unwrap();

        let json_invite = out_of_band_receiver.invitation_to_json_string();

        assert_eq!(JSON_OOB_INVITE_NO_WHITESPACE, json_invite);
    }

    #[test]
    fn invitation_to_base64_url() {
        let out_of_band_receiver =
            OutOfBandReceiver::create_from_json_encoded_oob(JSON_OOB_INVITE).unwrap();

        let base64_url_invite = out_of_band_receiver.invitation_to_base64_url();

        assert_eq!(OOB_BASE64_URL_ENCODED, base64_url_invite);
    }

    #[test]
    fn invitation_to_url() {
        let out_of_band_receiver =
            OutOfBandReceiver::create_from_json_encoded_oob(JSON_OOB_INVITE).unwrap();

        let oob_url = out_of_band_receiver
            .invitation_to_url("http://example.com/ssi")
            .unwrap()
            .to_string();

        assert_eq!(OOB_URL, oob_url);
    }
}
