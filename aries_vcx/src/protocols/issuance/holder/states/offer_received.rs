use std::sync::Arc;

use messages::protocols::issuance::credential_offer::CredentialOffer;

use crate::{
    core::profile::profile::Profile,
    errors::error::prelude::*,
    protocols::issuance::{
        holder::{state_machine::parse_cred_def_id_from_cred_offer, states::request_sent::RequestSentState},
        is_cred_def_revokable,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OfferReceivedState {
    pub offer: CredentialOffer,
}

impl From<(OfferReceivedState, String, String)> for RequestSentState {
    fn from((_state, req_meta, cred_def_json): (OfferReceivedState, String, String)) -> Self {
        trace!("SM is now in RequestSent state");
        trace!("cred_def_json={:?}", cred_def_json);
        RequestSentState {
            req_meta,
            cred_def_json,
        }
    }
}

impl OfferReceivedState {
    pub fn new(offer: CredentialOffer) -> Self {
        OfferReceivedState { offer }
    }

    pub fn get_attributes(&self) -> VcxResult<String> {
        let mut new_map = serde_json::map::Map::new();
        self.offer.credential_preview.attributes.iter().for_each(|attribute| {
            new_map.insert(
                attribute.name.clone(),
                serde_json::Value::String(attribute.value.clone()),
            );
        });
        Ok(serde_json::Value::Object(new_map).to_string())
    }

    pub async fn is_revokable(&self, profile: &Arc<dyn Profile>) -> VcxResult<bool> {
        let offer = self.offer.offers_attach.content().map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Failed to get credential offer attachment content: {}", err),
            )
        })?;
        let cred_def_id = parse_cred_def_id_from_cred_offer(&offer).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!(
                    "Failed to parse credential definition id from credential offer: {}",
                    err
                ),
            )
        })?;
        is_cred_def_revokable(profile, &cred_def_id).await
    }

    pub fn get_attachment(&self) -> VcxResult<String> {
        self.offer.offers_attach.content().map_err(|err| err.into())
    }
}
