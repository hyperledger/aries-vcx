use crate::error::prelude::*;
use crate::handlers::issuance::holder::states::finished::FinishedHolderState;
use crate::handlers::issuance::holder::states::request_sent::RequestSentState;
use crate::messages::error::ProblemReport;
use crate::messages::issuance::credential_offer::CredentialOffer;
use crate::messages::status::Status;
use crate::handlers::issuance::holder::state_machine::parse_cred_def_id_from_cred_offer;
use crate::libindy::utils::anoncreds::get_cred_def_json;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OfferReceivedState {
    pub offer: CredentialOffer
}

impl From<(OfferReceivedState, String, String)> for RequestSentState {
    fn from((_state, req_meta, cred_def_json): (OfferReceivedState, String, String)) -> Self {
        trace!("SM is now in RequestSent state");
        trace!("cred_def_json={:?}", cred_def_json);
        RequestSentState {
            req_meta,
            cred_def_json
        }
    }
}

impl From<(OfferReceivedState, ProblemReport)> for FinishedHolderState {
    fn from((_state, problem_report): (OfferReceivedState, ProblemReport)) -> Self {
        trace!("SM is now in Finished state");
        FinishedHolderState {
            cred_id: None,
            credential: None,
            status: Status::Failed(problem_report),
            rev_reg_def_json: None,
        }
    }
}

impl OfferReceivedState {
    pub fn new(offer: CredentialOffer) -> Self {
        OfferReceivedState {
            offer,
        }
    }

    pub fn get_attributes(&self) -> VcxResult<String> {
        let mut new_map = serde_json::map::Map::new();
        self.offer.credential_preview.attributes.iter().for_each(|attribute| {
            new_map.insert(attribute.name.clone(), serde_json::Value::String(attribute.value.clone()));
        });
        Ok(serde_json::Value::Object(new_map).to_string())
    }

    pub fn is_revokable(&self) -> VcxResult<bool> {
        let offer = self.offer.offers_attach.content()
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Failed to get credential offer attachment content: {}", err)))?;
        let cred_def_id = parse_cred_def_id_from_cred_offer(&offer)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Failed to parse credential definition id from credential offer: {}", err)))?;
        let (_, cred_def_json) = get_cred_def_json(&cred_def_id)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("Failed to obtain credential definition from ledger or cache: {}", err)))?;
        let parsed_cred_def: serde_json::Value = serde_json::from_str(&cred_def_json)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed deserialize credential definition json {}\nError: {}", cred_def_json, err)))?;
        Ok(!parsed_cred_def["value"]["revocation"].is_null())
    }

    pub fn get_attachment(&self) -> VcxResult<String> {
        self.offer.offers_attach.content()
    }
}
