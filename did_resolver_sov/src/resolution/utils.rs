use chrono::{DateTime, NaiveDateTime, Utc};
use did_resolver::{
    did_doc_builder::schema::{did_doc::DIDDocument, service::Service, types::uri::Uri},
    did_parser::ParsedDID,
    shared_types::did_document_metadata::DIDDocumentMetadata,
    traits::resolvable::{
        resolution_metadata::DIDResolutionMetadata, resolution_output::DIDResolutionOutput,
    },
};
use serde_json::Value;

use crate::{
    error::{parsing::ParsingErrorSource, DIDSovError},
    service::{DidSovServiceType, EndpointDidSov},
};

fn prepare_ids(did: &str) -> Result<(Uri, ParsedDID), DIDSovError> {
    let service_id = Uri::new(did.to_string())?;
    let ddo_id = ParsedDID::parse(did.to_string())?;
    Ok((service_id, ddo_id))
}

fn get_data_from_response(resp: &str) -> Result<Value, DIDSovError> {
    let resp: serde_json::Value = serde_json::from_str(resp)?;
    match &resp["result"]["data"] {
        Value::String(ref data) => serde_json::from_str(data).map_err(|err| err.into()),
        Value::Null => Err(DIDSovError::NotFound("DID not found".to_string())),
        resp @ _ => Err(DIDSovError::ParsingError(
            ParsingErrorSource::LedgerResponseParsingError(format!(
                "Unexpected data format in ledger response: {resp}"
            )),
        )),
    }
}

fn get_txn_time_from_response(resp: &str) -> Result<i64, DIDSovError> {
    let resp: serde_json::Value = serde_json::from_str(resp)?;
    let txn_time = resp["result"]["txnTime"]
        .as_i64()
        .ok_or(DIDSovError::ParsingError(
            ParsingErrorSource::LedgerResponseParsingError("Failed to parse txnTime".to_string()),
        ))?;
    Ok(txn_time)
}

fn posix_to_datetime(posix_timestamp: i64) -> Option<DateTime<Utc>> {
    if let Some(date_time) = NaiveDateTime::from_timestamp_opt(posix_timestamp, 0) {
        Some(DateTime::<Utc>::from_utc(date_time, Utc))
    } else {
        None
    }
}

pub(super) fn is_valid_sovrin_did_id(id: &str) -> bool {
    if id.len() < 21 || id.len() > 22 {
        return false;
    }
    for c in id.chars() {
        match c {
            '1'..='9' | 'A'..='H' | 'J'..='N' | 'P'..='Z' | 'a'..='k' | 'm'..='z' => {}
            _ => return false,
        }
    }
    true
}

pub(super) async fn resolve_ddo(did: &str, resp: &str) -> Result<DIDResolutionOutput, DIDSovError> {
    let (service_id, ddo_id) = prepare_ids(did)?;

    let service_data = get_data_from_response(&resp)?;
    let endpoint: EndpointDidSov = serde_json::from_value(service_data["endpoint"].clone())?;

    let txn_time = get_txn_time_from_response(&resp)?;
    let datetime = posix_to_datetime(txn_time);

    let service = {
        let mut service_builder = Service::builder(service_id, endpoint.endpoint)?;
        for t in endpoint.types {
            if t != DidSovServiceType::Unknown {
                service_builder = service_builder.add_type(t.to_string())?;
            };
        }
        service_builder.build()?
    };

    let ddo = DIDDocument::builder(ddo_id).add_service(service).build();

    let ddo_metadata = {
        let mut metadata_builder = DIDDocumentMetadata::builder().deactivated(false);
        if let Some(datetime) = datetime {
            metadata_builder = metadata_builder.updated(datetime);
        };
        metadata_builder.build()
    };

    let resolution_metadata = DIDResolutionMetadata::builder()
        .content_type("application/did+json".to_string())
        .build();

    Ok(DIDResolutionOutput::builder(ddo)
        .did_document_metadata(ddo_metadata)
        .did_resolution_metadata(resolution_metadata)
        .build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_prepare_ids() {
        let did = "did:example:1234567890".to_string();
        let (service_id, ddo_id) = prepare_ids(&did).unwrap();
        assert_eq!(service_id.to_string(), "did:example:1234567890");
        assert_eq!(ddo_id.to_string(), "did:example:1234567890");
    }

    #[test]
    fn test_get_data_from_response() {
        let resp = r#"{
            "result": {
                "data": "{\"endpoint\":{\"endpoint\":\"https://example.com\"}}"
            }
        }"#;
        let data = get_data_from_response(&resp).unwrap();
        assert_eq!(
            data["endpoint"]["endpoint"].as_str().unwrap(),
            "https://example.com"
        );
    }

    #[test]
    fn test_get_txn_time_from_response() {
        let resp = r#"{
            "result": {
                "txnTime": 1629272938
            }
        }"#;
        let txn_time = get_txn_time_from_response(&resp).unwrap();
        assert_eq!(txn_time, 1629272938);
    }

    #[test]
    fn test_posix_to_datetime() {
        let posix_timestamp = 1629272938;
        let datetime = posix_to_datetime(posix_timestamp).unwrap();
        assert_eq!(
            datetime,
            chrono::Utc.timestamp_opt(posix_timestamp, 0).unwrap()
        );
    }

    #[tokio::test]
    async fn test_resolve_ddo() {
        let did = "did:example:1234567890";
        let resp = r#"{
            "result": {
                "data": "{\"endpoint\":{\"endpoint\":\"https://example.com\"}}",
                "txnTime": 1629272938
            }
        }"#;
        let resolution_output = resolve_ddo(&did, &resp).await.unwrap();
        let ddo = resolution_output.did_document();
        assert_eq!(ddo.id().to_string(), "did:example:1234567890");
        assert_eq!(ddo.service()[0].id().to_string(), "did:example:1234567890");
        assert_eq!(
            ddo.service()[0].service_endpoint().to_string(),
            "https://example.com"
        );
        assert_eq!(
            resolution_output.did_document_metadata().updated().unwrap(),
            chrono::Utc.timestamp_opt(1629272938, 0).unwrap()
        );
        assert_eq!(
            resolution_output
                .did_resolution_metadata()
                .content_type()
                .unwrap(),
            "application/did+json"
        );
    }
}
