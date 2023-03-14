use messages::protocols::{
    issuance::credential_offer::CredentialOffer, proof_presentation::presentation_request::PresentationRequest,
};
use serde_json;

use crate::errors::error::prelude::*;

fn _filter_proof_requests_by_name(requests: &str, match_name: &str) -> VcxResult<Vec<PresentationRequest>> {
    let presentation_requests: Vec<PresentationRequest> = serde_json::from_str(requests).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!(
                "Failed to deserialize Vec<PresentationRequest>: {}\nObtained error: {:?}",
                requests, err
            ),
        )
    })?;
    let filtered = presentation_requests
        .into_iter()
        .filter_map(|presentation_request| {
            match presentation_request.request_presentations_attach.content().ok() {
                Some(content) => {
                    match serde_json::from_str::<serde_json::Value>(&content) {
                        Ok(value) => match value.get("name") {
                            Some(name) => match name.as_str() {
                                Some(name) if *name == *match_name => Some(presentation_request),
                                _ => None, // Not a string or not equal
                            },
                            _ => None, // No name field found in the content
                        },
                        _ => None, // Content deserialization failed
                    }
                }
                _ => None, // No content
            }
        })
        .collect();
    Ok(filtered)
}

fn _filter_offers_by_comment(offers: &str, match_comment: &str) -> VcxResult<Vec<CredentialOffer>> {
    let credential_offers: Vec<CredentialOffer> = serde_json::from_str(offers).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!(
                "Failed to deserialize Vec<CredentialOffer>: {}\nObtained error: {:?}",
                offers, err
            ),
        )
    })?;
    let filtered = credential_offers
        .into_iter()
        .filter_map(|credential_offer| match &credential_offer.comment {
            Some(comment) if *comment == *match_comment => Some(credential_offer),
            _ => None,
        })
        .collect();
    Ok(filtered)
}

// todo: need not to return Result, can be modified to return String, never error - likely for other
// functions in this file as well
pub fn filter_proof_requests_by_name(requests: &str, name: &str) -> VcxResult<String> {
    let presentation_requests: Vec<PresentationRequest> = _filter_proof_requests_by_name(requests, name)?;
    let filtered: String = serde_json::to_string(&presentation_requests).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!(
                "Failed to serialize filtered proof requests: {}\nObtained error: {:?}",
                requests, err
            ),
        )
    })?;
    Ok(filtered)
}

pub fn filter_credential_offers_by_comment(offers: &str, comment: &str) -> VcxResult<String> {
    let credential_offers: Vec<CredentialOffer> = _filter_offers_by_comment(offers, comment)?;
    let filtered: String = serde_json::to_string(&credential_offers).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!(
                "Failed to serialize filtered credential offers: {}\nObtained error: {:?}",
                offers, err
            ),
        )
    })?;
    Ok(filtered)
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use super::*;
    use crate::utils::mockdata::mockdata_proof;

    #[test]
    fn test_filter_proof_requests_by_name() {
        let filtered =
            _filter_proof_requests_by_name(mockdata_proof::PRESENTATION_REQUEST_MESSAGE_ARRAY, "request1").unwrap();
        assert_eq!(filtered.len(), 1);
        let filtered =
            _filter_proof_requests_by_name(mockdata_proof::PRESENTATION_REQUEST_MESSAGE_ARRAY, "request2").unwrap();
        assert_eq!(filtered.len(), 1);
        let filtered =
            _filter_proof_requests_by_name(mockdata_proof::PRESENTATION_REQUEST_MESSAGE_ARRAY, "not there").unwrap();
        assert_eq!(filtered.len(), 0);
        let filtered = _filter_proof_requests_by_name(mockdata_proof::PRESENTATION_REQUEST_MESSAGE_ARRAY, "").unwrap();
        assert_eq!(filtered.len(), 0);
        let filtered = _filter_proof_requests_by_name(
            mockdata_proof::PRESENTATION_REQUEST_MESSAGE_ARRAY_EMPTY_ATTACH,
            "not there",
        )
        .unwrap();
        assert_eq!(filtered.len(), 0);
        let filtered =
            _filter_proof_requests_by_name(mockdata_proof::PRESENTATION_REQUEST_MESSAGE_ARRAY_EMPTY_ATTACH, "")
                .unwrap();
        assert_eq!(filtered.len(), 0);
    }
}
