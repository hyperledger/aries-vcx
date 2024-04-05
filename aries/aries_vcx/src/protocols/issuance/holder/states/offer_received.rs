use aries_vcx_ledger::ledger::base_ledger::AnoncredsLedgerRead;
use messages::msg_fields::protocols::cred_issuance::v1::offer_credential::OfferCredentialV1;

use crate::{
    errors::error::prelude::*,
    handlers::util::get_attach_as_string,
    protocols::issuance::{
        holder::state_machine::parse_cred_def_id_from_cred_offer, is_cred_def_revokable,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OfferReceivedState {
    pub offer: OfferCredentialV1,
}

impl OfferReceivedState {
    pub fn new(offer: OfferCredentialV1) -> Self {
        OfferReceivedState { offer }
    }

    pub fn get_attributes(&self) -> VcxResult<String> {
        let mut new_map = serde_json::map::Map::new();
        self.offer
            .content
            .credential_preview
            .attributes
            .iter()
            .for_each(|attribute| {
                new_map.insert(
                    attribute.name.clone(),
                    serde_json::Value::String(attribute.value.clone()),
                );
            });
        Ok(serde_json::Value::Object(new_map).to_string())
    }

    pub async fn is_revokable(&self, ledger: &impl AnoncredsLedgerRead) -> VcxResult<bool> {
        let offer = self.get_attachment()?;

        let cred_def_id = parse_cred_def_id_from_cred_offer(&offer).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!(
                    "Failed to parse credential definition id from credential offer: {}",
                    err
                ),
            )
        })?;
        is_cred_def_revokable(ledger, &cred_def_id).await
    }

    pub fn get_attachment(&self) -> VcxResult<String> {
        Ok(get_attach_as_string!(self.offer.content.offers_attach))
    }
}
