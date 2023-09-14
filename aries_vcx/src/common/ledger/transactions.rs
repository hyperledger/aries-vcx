use bs58;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use diddoc_legacy::aries::service::AriesService;
use messages::msg_fields::protocols::connection::invitation::{Invitation, InvitationContent};
use messages::msg_fields::protocols::out_of_band::invitation::OobService;
use std::{collections::HashMap, sync::Arc};

use crate::common::ledger::service_didsov::EndpointDidSov;
use crate::handlers::util::AnyInvitation;
use aries_vcx_core::ledger::base_ledger::{IndyLedgerRead, IndyLedgerWrite};
use aries_vcx_core::ledger::indy_vdr_ledger::{LedgerRole, UpdateRole};
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use serde_json::Value;

use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};
use crate::{common::keys::get_verkey_from_ledger, global::settings};

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
    pub result: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ReplyV1 {
    pub data: ReplyDataV1,
}

#[derive(Debug, Deserialize)]
pub struct ReplyDataV1 {
    pub result: serde_json::Value,
}

const DID_KEY_PREFIX: &str = "did:key:";
const ED25519_MULTIBASE_CODEC: [u8; 2] = [0xed, 0x01];

pub async fn resolve_service(indy_ledger: &Arc<dyn IndyLedgerRead>, service: &OobService) -> VcxResult<AriesService> {
    match service {
        OobService::AriesService(service) => Ok(service.clone()),
        OobService::Did(did) => get_service(indy_ledger, did).await,
    }
}

pub async fn add_new_did(
    wallet: &Arc<dyn BaseWallet>,
    indy_ledger_write: &Arc<dyn IndyLedgerWrite>,
    submitter_did: &str,
    role: Option<&str>,
) -> VcxResult<(String, String)> {
    let (did, verkey) = wallet.create_and_store_my_did(None, None).await?;

    let res = indy_ledger_write
        .publish_nym(submitter_did, &did, Some(&verkey), None, role)
        .await?;
    check_response(&res)?;

    Ok((did, verkey))
}

pub async fn into_did_doc(indy_ledger: &Arc<dyn IndyLedgerRead>, invitation: &AnyInvitation) -> VcxResult<AriesDidDoc> {
    let mut did_doc: AriesDidDoc = AriesDidDoc::default();
    let (service_endpoint, recipient_keys, routing_keys) = match invitation {
        AnyInvitation::Con(Invitation {
            id,
            content: InvitationContent::Public(content),
            decorators,
        }) => {
            did_doc.set_id(content.did.to_string());
            let service = get_service(indy_ledger, &content.did).await.unwrap_or_else(|err| {
                error!("Failed to obtain service definition from the ledger: {}", err);
                AriesService::default()
            });
            (service.service_endpoint, service.recipient_keys, service.routing_keys)
        }
        AnyInvitation::Con(Invitation {
            id,
            content: InvitationContent::Pairwise(content),
            decorators,
        }) => {
            did_doc.set_id(id.clone());
            (
                content.service_endpoint.clone(),
                content.recipient_keys.clone(),
                content.routing_keys.clone(),
            )
        }
        AnyInvitation::Con(Invitation {
            id,
            content: InvitationContent::PairwiseDID(content),
            decorators,
        }) => {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidDid,
                format!("PairwiseDID invitation not supported yet!"),
            ))
        }
        AnyInvitation::Oob(invitation) => {
            did_doc.set_id(invitation.id.clone());
            let service = resolve_service(indy_ledger, &invitation.content.services[0])
                .await
                .unwrap_or_else(|err| {
                    error!("Failed to obtain service definition from the ledger: {}", err);
                    AriesService::default()
                });
            let recipient_keys = normalize_keys_as_naked(service.recipient_keys).unwrap_or_else(|err| {
                error!("Is not did valid: {}", err);
                Vec::new()
            });
            (service.service_endpoint, recipient_keys, service.routing_keys)
        }
    };
    did_doc.set_service_endpoint(service_endpoint);
    did_doc.set_recipient_keys(recipient_keys);
    did_doc.set_routing_keys(routing_keys);
    Ok(did_doc)
}

fn _ed25519_public_key_to_did_key(public_key_base58: &str) -> VcxResult<String> {
    let public_key_bytes = bs58::decode(public_key_base58).into_vec().map_err(|_| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidDid,
            format!("Could not base58 decode a did:key fingerprint: {}", public_key_base58),
        )
    })?;
    let mut did_key_bytes = ED25519_MULTIBASE_CODEC.to_vec();
    did_key_bytes.extend_from_slice(&public_key_bytes);
    let did_key_bytes_bs58 = bs58::encode(&did_key_bytes).into_string();
    let did_key = format!("{DID_KEY_PREFIX}z{did_key_bytes_bs58}");
    Ok(did_key)
}

fn normalize_keys_as_naked(keys_list: Vec<String>) -> VcxResult<Vec<String>> {
    let mut result = Vec::new();
    for key in keys_list {
        if let Some(stripped_didkey) = key.strip_prefix(DID_KEY_PREFIX) {
            let stripped = if let Some(stripped) = stripped_didkey.strip_prefix('z') {
                stripped
            } else {
                Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidDid,
                    format!("z prefix is missing: {}", key),
                ))?
            };
            let decoded_value = bs58::decode(stripped).into_vec().map_err(|_| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidDid,
                    format!("Could not decode base58: {} as portion of {}", stripped, key),
                )
            })?;
            let verkey = if let Some(public_key_bytes) = decoded_value.strip_prefix(&ED25519_MULTIBASE_CODEC) {
                Ok(bs58::encode(public_key_bytes).into_string())
            } else {
                Err(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidDid,
                    format!("Only Ed25519-based did:keys are currently supported, got key: {}", key),
                ))
            }?;
            result.push(verkey);
        } else {
            result.push(key);
        }
    }
    Ok(result)
}

pub async fn get_service(ledger: &Arc<dyn IndyLedgerRead>, did: &String) -> VcxResult<AriesService> {
    let did_raw = did.to_string();
    let did_raw = match did_raw.rsplit_once(':') {
        None => did_raw,
        Some((_, value)) => value.to_string(),
    };
    let attr_resp = ledger.get_attr(&did_raw, "endpoint").await?;
    let data = get_data_from_response(&attr_resp)?;
    if data["endpoint"].is_object() {
        let endpoint: EndpointDidSov = serde_json::from_value(data["endpoint"].clone())?;
        let recipient_keys = vec![get_verkey_from_ledger(ledger, &did_raw).await?];
        let endpoint_url = endpoint.endpoint;

        return Ok(AriesService::create()
            .set_recipient_keys(recipient_keys)
            .set_service_endpoint(endpoint_url)
            .set_routing_keys(endpoint.routing_keys.unwrap_or_default()));
    }
    parse_legacy_endpoint_attrib(ledger, &did_raw).await
}

pub async fn parse_legacy_endpoint_attrib(
    indy_ledger: &Arc<dyn IndyLedgerRead>,
    did_raw: &str,
) -> VcxResult<AriesService> {
    let attr_resp = indy_ledger.get_attr(did_raw, "service").await?;
    let data = get_data_from_response(&attr_resp)?;
    let ser_service = match data["service"].as_str() {
        Some(ser_service) => ser_service.to_string(),
        None => {
            warn!("Failed converting service read from ledger {:?} to string, falling back to new single-serialized format", data["service"]);
            data["service"].to_string()
        }
    };
    serde_json::from_str(&ser_service).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::SerializationError,
            format!("Failed to deserialize service read from the ledger: {:?}", err),
        )
    })
}

pub async fn write_endorser_did(
    indy_ledger_write: &Arc<dyn IndyLedgerWrite>,
    submitter_did: &str,
    target_did: &str,
    target_vk: &str,
    alias: Option<String>,
) -> VcxResult<String> {
    let res = indy_ledger_write
        .write_did(
            submitter_did,
            target_did,
            target_vk,
            Some(UpdateRole::Set(LedgerRole::Endorser)),
            alias,
        )
        .await?;
    check_response(&res)?;
    Ok(res)
}

pub async fn write_endpoint_legacy(
    indy_ledger_write: &Arc<dyn IndyLedgerWrite>,
    did: &str,
    service: &AriesService,
) -> VcxResult<String> {
    let attrib_json = json!({ "service": service }).to_string();
    let res = indy_ledger_write.add_attr(did, &attrib_json).await?;
    check_response(&res)?;
    Ok(res)
}

pub async fn write_endpoint(
    indy_ledger_write: &Arc<dyn IndyLedgerWrite>,
    did: &str,
    service: &EndpointDidSov,
) -> VcxResult<String> {
    let attrib_json = json!({ "endpoint": service }).to_string();
    let res = indy_ledger_write.add_attr(did, &attrib_json).await?;
    check_response(&res)?;
    Ok(res)
}

pub async fn add_attr(indy_ledger_write: &Arc<dyn IndyLedgerWrite>, did: &str, attr: &str) -> VcxResult<()> {
    let res = indy_ledger_write.add_attr(did, &attr).await?;
    check_response(&res)
}

pub async fn get_attr(ledger: &Arc<dyn IndyLedgerRead>, did: &str, attr_name: &str) -> VcxResult<String> {
    let attr_resp = ledger.get_attr(did, attr_name).await?;
    let data = get_data_from_response(&attr_resp)?;
    match data.get(attr_name) {
        None => Ok("".into()),
        Some(attr) if attr.is_null() => Ok("".into()),
        Some(attr) => Ok(attr.to_string()),
    }
}

pub async fn clear_attr(indy_ledger_write: &Arc<dyn IndyLedgerWrite>, did: &str, attr_name: &str) -> VcxResult<String> {
    indy_ledger_write
        .add_attr(did, &json!({ attr_name: Value::Null }).to_string())
        .await
        .map_err(|err| err.into())
}

pub(self) fn check_response(response: &str) -> VcxResult<()> {
    if settings::indy_mocks_enabled() {
        return Ok(());
    }
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
    let resp: serde_json::Value = serde_json::from_str(resp)
        .map_err(|err| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidLedgerResponse, format!("{:?}", err)))?;
    serde_json::from_str(resp["result"]["data"].as_str().unwrap_or("{}"))
        .map_err(|err| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidLedgerResponse, format!("{:?}", err)))
}
