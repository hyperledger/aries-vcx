use std::collections::HashMap;

use aries_vcx_core::ledger::{
    base_ledger::{IndyLedgerRead, IndyLedgerWrite},
    indy_vdr_ledger::{LedgerRole, UpdateRole},
};

use aries_vcx_wallet::wallet::base_wallet::BaseWallet;
use did_doc::schema::service::Service;
use did_parser::Did;
use diddoc_legacy::aries::service::AriesService;
use messages::msg_fields::protocols::out_of_band::invitation::OobService;
use public_key::{Key, KeyType};
use serde_json::Value;

use crate::{
    common::{keys::get_verkey_from_ledger, ledger::service_didsov::EndpointDidSov},
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub req_id: u64,
    pub identifier: String,
    pub signature: Option<String>,
    pub signatures: Option<HashMap<String, String>>,
    pub endorser: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "op")]
pub enum Response {
    #[serde(rename = "REQNACK")]
    ReqNACK(Reject),
    #[serde(rename = "REJECT")]
    Reject(Reject),
    #[serde(rename = "REPLY")]
    Reply(Reply),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Reject {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Reply {
    ReplyV0(ReplyV0),
    ReplyV1(ReplyV1),
}

#[derive(Debug, Deserialize)]
pub struct ReplyV0 {
    pub result: Value,
}

#[derive(Debug, Deserialize)]
pub struct ReplyV1 {
    pub data: ReplyDataV1,
}

#[derive(Debug, Deserialize)]
pub struct ReplyDataV1 {
    pub result: Value,
}

pub async fn resolve_service(
    indy_ledger: &impl IndyLedgerRead,
    service: &OobService,
) -> VcxResult<AriesService> {
    match service {
        OobService::AriesService(service) => Ok(service.clone()),
        OobService::Did(did) => get_service(indy_ledger, &did.clone().parse()?).await,
    }
}

pub async fn add_new_did(
    wallet: &impl BaseWallet,
    indy_ledger_write: &impl IndyLedgerWrite,
    submitter_did: &Did,
    role: Option<&str>,
) -> VcxResult<(Did, String)> {
    let did_data = wallet.create_and_store_my_did(None, None).await?;

    let res = indy_ledger_write
        .publish_nym(
            wallet,
            submitter_did,
            &did_data.did().parse()?,
            Some(did_data.verkey()),
            None,
            role,
        )
        .await?;
    check_response(&res)?;

    Ok((did_data.did().parse()?, did_data.verkey().base58()))
}

pub async fn get_service(ledger: &impl IndyLedgerRead, did: &Did) -> VcxResult<AriesService> {
    let attr_resp = ledger.get_attr(did, "endpoint").await?;
    let data = get_data_from_response(&attr_resp)?;
    if data["endpoint"].is_object() {
        let endpoint: EndpointDidSov = serde_json::from_value(data["endpoint"].clone())?;
        let recipient_keys = vec![get_verkey_from_ledger(ledger, did).await?];
        let endpoint_url = endpoint.endpoint;

        return Ok(AriesService::create()
            .set_recipient_keys(recipient_keys)
            .set_service_endpoint(endpoint_url)
            .set_routing_keys(endpoint.routing_keys.unwrap_or_default()));
    }
    parse_legacy_endpoint_attrib(ledger, did).await
}

pub async fn parse_legacy_endpoint_attrib(
    indy_ledger: &impl IndyLedgerRead,
    did_raw: &Did,
) -> VcxResult<AriesService> {
    let attr_resp = indy_ledger.get_attr(did_raw, "service").await?;
    let data = get_data_from_response(&attr_resp)?;
    let ser_service = match data["service"].as_str() {
        Some(ser_service) => ser_service.to_string(),
        None => {
            warn!(
                "Failed converting service read from ledger {:?} to string, falling back to new \
                 single-serialized format",
                data["service"]
            );
            data["service"].to_string()
        }
    };
    serde_json::from_str(&ser_service).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!(
                "Failed to deserialize service read from the ledger: {:?}",
                err
            ),
        )
    })
}

pub async fn write_endorser_did(
    wallet: &impl BaseWallet,
    indy_ledger_write: &impl IndyLedgerWrite,
    submitter_did: &Did,
    target_did: &Did,
    target_vk: &str,
    alias: Option<String>,
) -> VcxResult<String> {
    let res = indy_ledger_write
        .write_did(
            wallet,
            submitter_did,
            target_did,
            &Key::from_base58(target_vk, KeyType::Ed25519)?,
            Some(UpdateRole::Set(LedgerRole::Endorser)),
            alias,
        )
        .await?;
    check_response(&res)?;
    Ok(res)
}

pub async fn write_endpoint_legacy(
    wallet: &impl BaseWallet,
    indy_ledger_write: &impl IndyLedgerWrite,
    did: &Did,
    service: &AriesService,
) -> VcxResult<String> {
    let attrib_json = json!({ "service": service }).to_string();
    let res = indy_ledger_write
        .add_attr(wallet, did, &attrib_json)
        .await?;
    check_response(&res)?;
    Ok(res)
}

pub async fn write_endpoint(
    wallet: &impl BaseWallet,
    indy_ledger_write: &impl IndyLedgerWrite,
    did: &Did,
    service: &EndpointDidSov,
) -> VcxResult<String> {
    let attrib_json = json!({ "endpoint": service }).to_string();
    let res = indy_ledger_write
        .add_attr(wallet, did, &attrib_json)
        .await?;
    check_response(&res)?;
    Ok(res)
}

fn _service_to_didsov_endpoint_attribute(service: &Service) -> EndpointDidSov {
    let routing_keys: Option<Vec<String>> = service
        .extra_field_routing_keys()
        .ok()
        .map(|keys| keys.iter().map(|key| key.to_string()).collect());

    let service_types = service.service_types();
    let types_str: Vec<String> = service_types.iter().map(|t| t.to_string()).collect();
    EndpointDidSov::create()
        .set_routing_keys(routing_keys)
        .set_types(Some(types_str))
        .set_service_endpoint(service.service_endpoint().clone())
}

pub async fn write_endpoint_from_service(
    wallet: &impl BaseWallet,
    indy_ledger_write: &impl IndyLedgerWrite,
    did: &Did,
    service: &Service,
) -> VcxResult<(String, EndpointDidSov)> {
    let attribute = _service_to_didsov_endpoint_attribute(service);
    let res = write_endpoint(wallet, indy_ledger_write, did, &attribute).await?;
    Ok((res, attribute))
}

pub async fn add_attr(
    wallet: &impl BaseWallet,
    indy_ledger_write: &impl IndyLedgerWrite,
    did: &Did,
    attr: &str,
) -> VcxResult<()> {
    let res = indy_ledger_write.add_attr(wallet, did, attr).await?;
    check_response(&res)
}

pub async fn get_attr(
    ledger: &impl IndyLedgerRead,
    did: &Did,
    attr_name: &str,
) -> VcxResult<String> {
    let attr_resp = ledger.get_attr(did, attr_name).await?;
    let data = get_data_from_response(&attr_resp)?;
    match data.get(attr_name) {
        None => Ok("".into()),
        Some(attr) if attr.is_null() => Ok("".into()),
        Some(attr) => Ok(attr.to_string()),
    }
}

pub async fn clear_attr(
    wallet: &impl BaseWallet,
    indy_ledger_write: &impl IndyLedgerWrite,
    did: &Did,
    attr_name: &str,
) -> VcxResult<String> {
    indy_ledger_write
        .add_attr(wallet, did, &json!({ attr_name: Value::Null }).to_string())
        .await
        .map_err(|err| err.into())
}

fn check_response(response: &str) -> VcxResult<()> {
    match parse_response(response)? {
        Response::Reply(_) => Ok(()),
        Response::Reject(res) | Response::ReqNACK(res) => Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidLedgerResponse,
            format!("{res:?}"),
        )),
    }
}

fn parse_response(response: &str) -> VcxResult<Response> {
    serde_json::from_str::<Response>(response).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!("Cannot deserialize transaction response: {err:?}"),
        )
    })
}

fn get_data_from_response(resp: &str) -> VcxResult<serde_json::Value> {
    let resp: serde_json::Value = serde_json::from_str(resp).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidLedgerResponse,
            format!("{:?}", err),
        )
    })?;
    serde_json::from_str(resp["result"]["data"].as_str().unwrap_or("{}")).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidLedgerResponse,
            format!("{:?}", err),
        )
    })
}

// #[cfg(test)]
// mod test {
//     use crate::common::test_utils::mock_profile;
//     use messages::a2a::MessageId;
//     use messages::diddoc::aries::diddoc::test_utils::{
//         _key_1, _key_1_did_key, _key_2, _key_2_did_key, _recipient_keys, _routing_keys,
// _service_endpoint,     };
//     use messages::protocols::connection::invite::test_utils::_pairwise_invitation;
//     use messages::protocols::out_of_band::invitation::OutOfBandInvitation;

//     use super::*;

//     #[tokio::test]
//     async fn test_did_doc_from_invitation_works() {
//         let mut did_doc = AriesDidDoc::default();
//         did_doc.set_id(MessageId::id().0);
//         did_doc.set_service_endpoint(_service_endpoint());
//         did_doc.set_recipient_keys(_recipient_keys());
//         did_doc.set_routing_keys(_routing_keys());
//         assert_eq!(
//             did_doc,
//             into_did_doc(&mock_profile(), &Invitation::Pairwise(_pairwise_invitation()))
//                 .await
//                 .unwrap()
//         );
//     }

//     #[tokio::test]
//     async fn test_did_doc_from_invitation_with_didkey_encoding_works() {
//         let recipient_keys = vec![_key_2()];
//         let routing_keys_did_key = vec![_key_2_did_key()];
//         let id = Uuid::new_v4().to_string();

//         let mut did_doc = AriesDidDoc::default();
//         did_doc.set_id(id.clone());
//         did_doc.set_service_endpoint(_service_endpoint());
//         did_doc.set_recipient_keys(recipient_keys);
//         did_doc.set_routing_keys(_routing_keys());

//         let aries_service = ServiceOob::AriesService(
//             AriesService::create()
//                 .set_service_endpoint(_service_endpoint())
//                 .set_routing_keys(_routing_keys())
//                 .set_recipient_keys(routing_keys_did_key),
//         );
//         invitation.services.push(aries_service);

//         assert_eq!(
//             did_doc,
//             into_did_doc(&mock_profile(), &Invitation::OutOfBand(invitation))
//                 .await
//                 .unwrap()
//         );
//     }

//     #[tokio::test]
//     async fn test_did_key_to_did_raw() {
//         let recipient_keys = vec![_key_1()];
//         let expected_output = vec![_key_1()];
//         assert_eq!(normalize_keys_as_naked(recipient_keys).unwrap(), expected_output);
//         let recipient_keys = vec!["abc".to_string(), "def".to_string(), "ghi".to_string()];
//         let expected_output = vec!["abc".to_string(), "def".to_string(), "ghi".to_string()];
//         assert_eq!(normalize_keys_as_naked(recipient_keys).unwrap(), expected_output);
//     }

//     #[tokio::test]
//     async fn test_did_naked_to_did_raw() {
//         let recipient_keys = vec![_key_1_did_key(), _key_2_did_key()];
//         let expected_output = vec![_key_1(), _key_2()];
//         assert_eq!(normalize_keys_as_naked(recipient_keys).unwrap(), expected_output);
//     }

//     #[tokio::test]
//     async fn test_did_bad_format_without_z_prefix() {
//         let recipient_keys = vec!["did:key:invalid".to_string()];
//         let test = normalize_keys_as_naked(recipient_keys).map_err(|e| e.kind());
//         let expected_error_kind = AriesVcxErrorKind::InvalidDid;
//         assert_eq!(test.unwrap_err(), expected_error_kind);
//     }

//     #[tokio::test]
//     async fn test_did_bad_format_without_ed25519_public() {
//         let recipient_keys = vec!["did:key:zInvalid".to_string()];
//         let test = normalize_keys_as_naked(recipient_keys).map_err(|e| e.kind());
//         let expected_error_kind = AriesVcxErrorKind::InvalidDid;
//         assert_eq!(test.unwrap_err(), expected_error_kind);
//     }

//     #[tokio::test]
//     async fn test_public_key_to_did_naked_with_previously_known_keys_suggested() {
//         let did_pub_with_key =
// "did:key:z6MkwHgArrRJq3tTdhQZKVAa1sdFgSAs5P5N1C4RJcD11Ycv".to_string();         let did_pub =
// "HqR8GcAsVWPzXCZrdvCjAn5Frru1fVq1KB9VULEz6KqY".to_string();         let did_raw =
// _ed25519_public_key_to_did_key(&did_pub).unwrap();         let recipient_keys = vec![did_raw];
//         let expected_output = vec![did_pub_with_key];
//         assert_eq!(recipient_keys, expected_output);
//     }

//     #[tokio::test]
//     async fn test_public_key_to_did_naked_with_previously_known_keys_rfc_0360() {
//         let did_pub_with_key_rfc_0360 =
// "did:key:z6MkmjY8GnV5i9YTDtPETC2uUAW6ejw3nk5mXF5yci5ab7th".to_string();         let
// did_pub_rfc_0360 = "8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K".to_string();         let
// did_raw = _ed25519_public_key_to_did_key(&did_pub_rfc_0360).unwrap();         let recipient_keys
// = vec![did_raw];         let expected_output = vec![did_pub_with_key_rfc_0360];
//         assert_eq!(recipient_keys, expected_output);
//     }

//     #[tokio::test]
//     async fn test_did_naked_with_previously_known_keys_suggested() {
//         let did_pub_with_key =
// vec!["did:key:z6MkwHgArrRJq3tTdhQZKVAa1sdFgSAs5P5N1C4RJcD11Ycv".to_string()];         let did_pub
// = vec!["HqR8GcAsVWPzXCZrdvCjAn5Frru1fVq1KB9VULEz6KqY".to_string()];         assert_eq!
// (normalize_keys_as_naked(did_pub_with_key).unwrap(), did_pub);     }

//     #[tokio::test]
//     async fn test_did_naked_with_previously_known_keys_rfc_0360() {
//         let did_pub_with_key_rfc_0360 =
// vec!["did:key:z6MkmjY8GnV5i9YTDtPETC2uUAW6ejw3nk5mXF5yci5ab7th".to_string()];         let
// did_pub_rfc_0360 = vec!["8HH5gYEeNc3z7PYXmd54d4x6qAfCNrqQqEB3nS7Zfu7K".to_string()];
//         assert_eq!(
//             normalize_keys_as_naked(did_pub_with_key_rfc_0360).unwrap(),
//             did_pub_rfc_0360
//         );
//     }
// }
