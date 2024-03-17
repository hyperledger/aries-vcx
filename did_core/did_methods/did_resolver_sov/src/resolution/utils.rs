use chrono::{DateTime, Utc};
use did_resolver::{
    did_doc::schema::{
        did_doc::DidDocument,
        service::{typed::ServiceType, Service},
        types::uri::Uri,
        utils::OneOrList,
        verification_method::{VerificationMethod, VerificationMethodType},
    },
    did_parser_nom::{Did, DidUrl},
    shared_types::did_document_metadata::DidDocumentMetadata,
    traits::resolvable::{
        resolution_metadata::DidResolutionMetadata, resolution_output::DidResolutionOutput,
    },
};
use serde_json::Value;

use crate::{
    error::{parsing::ParsingErrorSource, DidSovError},
    service::{DidSovServiceType, EndpointDidSov},
};

fn prepare_ids(did: &str) -> Result<(Uri, Did), DidSovError> {
    let service_id = Uri::new(did)?;
    let ddo_id = Did::parse(did.to_string())?;
    Ok((service_id, ddo_id))
}

fn get_data_from_response(resp: &str) -> Result<Value, DidSovError> {
    let resp: serde_json::Value = serde_json::from_str(resp)?;
    match &resp["result"]["data"] {
        Value::String(ref data) => serde_json::from_str(data).map_err(|err| err.into()),
        Value::Null => Err(DidSovError::NotFound("DID not found".to_string())),
        resp => Err(DidSovError::ParsingError(
            ParsingErrorSource::LedgerResponseParsingError(format!(
                "Unexpected data format in ledger response: {resp}"
            )),
        )),
    }
}

fn get_txn_time_from_response(resp: &str) -> Result<i64, DidSovError> {
    let resp: serde_json::Value = serde_json::from_str(resp)?;
    let txn_time = resp["result"]["txnTime"]
        .as_i64()
        .ok_or(DidSovError::ParsingError(
            ParsingErrorSource::LedgerResponseParsingError("Failed to parse txnTime".to_string()),
        ))?;
    Ok(txn_time)
}

fn unix_to_datetime(posix_timestamp: i64) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp(posix_timestamp, 0)
}

pub(super) fn is_valid_sovrin_did_id(id: &str) -> bool {
    if id.len() < 21 || id.len() > 22 {
        return false;
    }
    let base58_chars = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    id.chars().all(|c| base58_chars.contains(c))
}

pub(super) async fn ledger_response_to_ddo(
    did: &str,
    resp: &str,
    verkey: String,
) -> Result<DidResolutionOutput, DidSovError> {
    log::info!("ledger_response_to_ddo >> did: {did}, verkey: {verkey}, resp: {resp}");
    let (service_id, ddo_id) = prepare_ids(did)?;

    let service_data = get_data_from_response(resp)?;
    log::info!("ledger_response_to_ddo >> service_data: {service_data:?}");
    let endpoint: EndpointDidSov = serde_json::from_value(service_data["endpoint"].clone())?;

    let txn_time = get_txn_time_from_response(resp)?;
    let datetime = unix_to_datetime(txn_time);

    let service_types: Vec<ServiceType> = endpoint
        .types
        .into_iter()
        .map(|t| match t {
            DidSovServiceType::Endpoint => ServiceType::AIP1,
            DidSovServiceType::DidCommunication => ServiceType::DIDCommV1,
            DidSovServiceType::DIDComm => ServiceType::DIDCommV2,
            DidSovServiceType::Unknown => ServiceType::Other("Unknown".to_string()),
        })
        .collect();
    let service = Service::new(
        service_id,
        endpoint.endpoint,
        OneOrList::List(service_types),
        Default::default(),
    );

    // TODO: Use multibase instead of base58
    let verification_method = VerificationMethod::builder(
        DidUrl::parse("#1".to_string())?,
        did.to_string().try_into()?,
        VerificationMethodType::Ed25519VerificationKey2018,
    )
    .add_public_key_base58(verkey)
    .build();

    let ddo = DidDocument::builder(ddo_id)
        .add_service(service)
        .add_verification_method(verification_method)
        .add_key_agreement_reference(DidUrl::parse("#1".to_string())?)
        .build();

    let ddo_metadata = {
        let mut metadata_builder = DidDocumentMetadata::builder().deactivated(false);
        if let Some(datetime) = datetime {
            metadata_builder = metadata_builder.updated(datetime);
        };
        metadata_builder.build()
    };

    let resolution_metadata = DidResolutionMetadata::builder()
        .content_type("application/did+json".to_string())
        .build();

    Ok(DidResolutionOutput::builder(ddo)
        .did_document_metadata(ddo_metadata)
        .did_resolution_metadata(resolution_metadata)
        .build())
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use did_resolver::did_doc::schema::verification_method::PublicKeyField;

    use super::*;

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
        let data = get_data_from_response(resp).unwrap();
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
        let txn_time = get_txn_time_from_response(resp).unwrap();
        assert_eq!(txn_time, 1629272938);
    }

    #[test]
    fn test_posix_to_datetime() {
        let posix_timestamp = 1629272938;
        let datetime = unix_to_datetime(posix_timestamp).unwrap();
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
        let verkey = "9wvq2i4xUa5umXoThe83CDgx1e5bsjZKJL4DEWvTP9qe".to_string();
        let DidResolutionOutput {
            did_document: ddo,
            did_resolution_metadata,
            did_document_metadata,
        } = ledger_response_to_ddo(did, resp, verkey).await.unwrap();
        assert_eq!(ddo.id().to_string(), "did:example:1234567890");
        assert_eq!(ddo.service()[0].id().to_string(), "did:example:1234567890");
        assert_eq!(
            ddo.service()[0].service_endpoint().as_ref(),
            "https://example.com/"
        );
        assert_eq!(
            did_document_metadata.updated().unwrap(),
            Utc.timestamp_opt(1629272938, 0).unwrap()
        );
        assert_eq!(
            did_resolution_metadata.content_type().unwrap(),
            "application/did+json"
        );
        if let PublicKeyField::Base58 { public_key_base58 } =
            ddo.verification_method()[0].public_key_field()
        {
            assert_eq!(
                public_key_base58,
                "9wvq2i4xUa5umXoThe83CDgx1e5bsjZKJL4DEWvTP9qe"
            );
        } else {
            panic!("Unexpected public key type");
        }
    }
}
