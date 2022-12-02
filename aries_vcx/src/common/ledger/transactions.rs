use std::{collections::HashMap, sync::Arc};

use messages::{
    connection::{did::Did, invite::Invitation},
    did_doc::{
        service_aries::AriesService, service_aries_public::EndpointDidSov, service_resolvable::ServiceResolvable,
        DidDoc,
    },
};
use serde_json::Value;

use crate::{
    common::keys::get_verkey_from_ledger,
    core::profile::profile::Profile,
    error::{VcxError, VcxErrorKind, VcxResult, VcxResultExt},
    global::settings,
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

pub async fn resolve_service(profile: &Arc<dyn Profile>, service: &ServiceResolvable) -> VcxResult<AriesService> {
    match service {
        ServiceResolvable::AriesService(service) => Ok(service.clone()),
        ServiceResolvable::Did(did) => get_service(profile, did).await,
    }
}

pub async fn add_new_did(
    profile: &Arc<dyn Profile>,
    submitter_did: &str,
    role: Option<&str>,
) -> VcxResult<(String, String)> {
    let (did, verkey) = profile.inject_wallet().create_and_store_my_did(None, None).await?;

    let ledger = Arc::clone(profile).inject_ledger();

    ledger
        .publish_nym(submitter_did, &did, Some(&verkey), None, role)
        .await?;

    Ok((did, verkey))
}

pub async fn into_did_doc(profile: &Arc<dyn Profile>, invitation: &Invitation) -> VcxResult<DidDoc> {
    let mut did_doc: DidDoc = DidDoc::default();
    let (service_endpoint, recipient_keys, routing_keys) = match invitation {
        Invitation::Public(invitation) => {
            did_doc.set_id(invitation.did.to_string());
            let service = get_service(profile, &invitation.did).await.unwrap_or_else(|err| {
                error!("Failed to obtain service definition from the ledger: {}", err);
                AriesService::default()
            });
            (service.service_endpoint, service.recipient_keys, service.routing_keys)
        }
        Invitation::Pairwise(invitation) => {
            did_doc.set_id(invitation.id.0.clone());
            (
                invitation.service_endpoint.clone(),
                invitation.recipient_keys.clone(),
                invitation.routing_keys.clone(),
            )
        }
        Invitation::OutOfBand(invitation) => {
            did_doc.set_id(invitation.id.0.clone());
            let service = resolve_service(profile, &invitation.services[0])
                .await
                .unwrap_or_else(|err| {
                    error!("Failed to obtain service definition from the ledger: {}", err);
                    AriesService::default()
                });
            (service.service_endpoint, service.recipient_keys, service.routing_keys)
        }
    };
    did_doc.set_service_endpoint(service_endpoint);
    did_doc.set_recipient_keys(recipient_keys);
    did_doc.set_routing_keys(routing_keys);
    Ok(did_doc)
}

pub async fn get_service(profile: &Arc<dyn Profile>, did: &Did) -> VcxResult<AriesService> {
    let did_raw = did.to_string();
    let did_raw = match did_raw.rsplit_once(':') {
        None => did_raw,
        Some((_, value)) => value.to_string(),
    };
    let ledger = Arc::clone(profile).inject_ledger();
    let attr_resp = ledger.get_attr(&did_raw, "endpoint").await?;
    let data = get_data_from_response(&attr_resp)?;
    if data["endpoint"].is_object() {
        let endpoint: EndpointDidSov = serde_json::from_value(data["endpoint"].clone())?;
        let recipient_keys = vec![get_verkey_from_ledger(profile, &did_raw).await.unwrap()];
        return Ok(AriesService::create()
            .set_recipient_keys(recipient_keys)
            .set_service_endpoint(endpoint.endpoint)
            .set_routing_keys(endpoint.routing_keys.unwrap()));
    }
    parse_legacy_endpoint_attrib(profile, &did_raw).await
}

pub async fn parse_legacy_endpoint_attrib(profile: &Arc<dyn Profile>, did_raw: &String) -> VcxResult<AriesService> {
    let ledger = Arc::clone(profile).inject_ledger();
    let attr_resp = ledger.get_attr(&did_raw, "service").await?;
    let data = get_data_from_response(&attr_resp)?;
    let ser_service = match data["service"].as_str() {
        Some(ser_service) => ser_service.to_string(),
        None => {
            warn!("Failed converting service read from ledger {:?} to string, falling back to new single-serialized format", data["service"]);
            data["service"].to_string()
        }
    };
    serde_json::from_str(&ser_service).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!("Failed to deserialize service read from the ledger: {:?}", err),
        )
    })
}

pub async fn write_endpoint_legacy(profile: &Arc<dyn Profile>, did: &str, service: &AriesService) -> VcxResult<String> {
    let attrib_json = json!({ "service": service }).to_string();
    let ledger = Arc::clone(profile).inject_ledger();
    let res = ledger.add_attr(did, &attrib_json).await?;
    check_response(&res)?;
    Ok(res)
}

pub async fn write_endpoint(profile: &Arc<dyn Profile>, did: &str, service: &EndpointDidSov) -> VcxResult<String> {
    let attrib_json = json!({ "endpoint": service }).to_string();
    let ledger = Arc::clone(profile).inject_ledger();
    let res = ledger.add_attr(did, &attrib_json).await?;
    check_response(&res)?;
    Ok(res)
}

pub async fn clear_attr(profile: &Arc<dyn Profile>, did: &str, attr_name: &str) -> VcxResult<String> {
    let ledger = Arc::clone(profile).inject_ledger();

    ledger.add_attr(did, &json!({ attr_name: Value::Null }).to_string()).await
}

pub(self) fn check_response(response: &str) -> VcxResult<()> {
    if settings::indy_mocks_enabled() {
        return Ok(());
    }
    match parse_response(response)? {
        Response::Reply(_) => Ok(()),
        Response::Reject(res) | Response::ReqNACK(res) => Err(VcxError::from_msg(
            VcxErrorKind::InvalidLedgerResponse,
            format!("{:?}", res),
        )),
    }
}

fn parse_response(response: &str) -> VcxResult<Response> {
    serde_json::from_str::<Response>(response)
        .to_vcx(VcxErrorKind::InvalidJson, "Cannot deserialize transaction response")
}

fn get_data_from_response(resp: &str) -> VcxResult<serde_json::Value> {
    let resp: serde_json::Value = serde_json::from_str(resp)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("{:?}", err)))?;
    serde_json::from_str(resp["result"]["data"].as_str().unwrap_or("{}"))
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("{:?}", err)))
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod test {
    use messages::a2a::MessageId;
    use messages::connection::invite::test_utils::_pairwise_invitation;
    use messages::did_doc::test_utils::{_recipient_keys, _routing_keys, _service_endpoint};

    use crate::common::test_utils::mock_profile;

    use super::*;

    #[tokio::test]
    async fn test_did_doc_from_invitation_works() {
        let mut did_doc = DidDoc::default();
        did_doc.set_id(MessageId::id().0);
        did_doc.set_service_endpoint(_service_endpoint());
        did_doc.set_recipient_keys(_recipient_keys());
        did_doc.set_routing_keys(_routing_keys());
        assert_eq!(
            did_doc,
            into_did_doc(&mock_profile(), &Invitation::Pairwise(_pairwise_invitation()))
                .await
                .unwrap()
        );
    }
}
