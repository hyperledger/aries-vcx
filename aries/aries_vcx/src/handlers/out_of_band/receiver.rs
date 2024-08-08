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

    pub fn from_json_string(oob_json: &str) -> VcxResult<Self> {
        Ok(Self {
            oob: from_json_string(oob_json)?,
        })
    }

    pub fn from_base64_url(base64_url_encoded_oob: &str) -> VcxResult<Self> {
        Ok(Self {
            oob: from_json_string(&from_base64_url(base64_url_encoded_oob)?)?,
        })
    }

    pub fn from_url(oob_url_string: &str) -> VcxResult<Self> {
        // TODO - URL Shortening
        Ok(Self {
            oob: from_json_string(&from_base64_url(&from_url(oob_url_string)?)?)?,
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

    pub fn to_aries_message(&self) -> AriesMessage {
        self.oob.clone().into()
    }

    pub fn to_json_string(&self) -> VcxResult<String> {
        Ok(serde_json::to_string(&self.oob)?)
    }

    pub fn to_base64_url(&self) -> String {
        URL_SAFE_LENIENT.encode(self.oob.to_string())
    }

    pub fn to_url(&self, domain_path: &str) -> VcxResult<Url> {
        let mut oob_url = Url::parse(&(domain_path.to_owned() + &self.oob.to_string()))?;
        let oob_query = "oob=".to_owned() + &self.to_base64_url();
        oob_url.set_query(Some(&oob_query));
        Ok(oob_url)
    }
}

fn from_json_string(oob_json: &str) -> VcxResult<Invitation> {
    Ok(serde_json::from_str(oob_json)?)
}

fn from_base64_url(base64_url_encoded_oob: &str) -> VcxResult<String> {
    Ok(String::from_utf8(
        URL_SAFE_LENIENT.decode(base64_url_encoded_oob)?,
    )?)
}

fn from_url(oob_url_string: &str) -> VcxResult<String> {
    let oob_url = Url::parse(oob_url_string)?;
    let (_oob_query, base64_url_encoded_oob) = oob_url
        .query_pairs()
        .find(|(name, _value)| name == &"oob")
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
