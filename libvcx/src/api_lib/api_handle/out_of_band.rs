use std::collections::HashMap;

use crate::api_lib::global::pool::get_main_pool_handle;
use aries_vcx::error::{VcxError, VcxErrorKind, VcxResult};
use aries_vcx::handlers::out_of_band::receiver::OutOfBandReceiver;
use aries_vcx::handlers::out_of_band::sender::OutOfBandSender;
use aries_vcx::indy::ledger::transactions::into_did_doc;
use aries_vcx::messages::a2a::A2AMessage;
use aries_vcx::messages::connection::did::Did;
use aries_vcx::messages::connection::invite::Invitation;
use aries_vcx::messages::did_doc::service_resolvable::ServiceResolvable;
use aries_vcx::messages::out_of_band::{GoalCode, HandshakeProtocol};

use napi_derive::napi;

use crate::api_lib::api_handle::mediated_connection::CONNECTION_MAP;
use crate::api_lib::api_handle::object_cache::ObjectCache;
use crate::api_lib::global::agency_client::get_main_agency_client;

lazy_static! {
    pub static ref OUT_OF_BAND_SENDER_MAP: ObjectCache<OutOfBandSender> =
        ObjectCache::<OutOfBandSender>::new("out-of-band-sender-cache");
    pub static ref OUT_OF_BAND_RECEIVER_MAP: ObjectCache<OutOfBandReceiver> =
        ObjectCache::<OutOfBandReceiver>::new("out-of-band-receiver-cache");
}

#[derive(Deserialize)]
pub struct OOBConfig {
    pub label: Option<String>,
    pub goal_code: Option<GoalCode>,
    pub goal: Option<String>,
    #[serde(default)]
    pub handshake_protocols: Vec<HandshakeProtocol>,
}

fn store_out_of_band_receiver(oob: OutOfBandReceiver) -> VcxResult<u32> {
    OUT_OF_BAND_RECEIVER_MAP
        .add(oob)
        .or(Err(VcxError::from(VcxErrorKind::CreateOutOfBand)))
}

fn store_out_of_band_sender(oob: OutOfBandSender) -> VcxResult<u32> {
    OUT_OF_BAND_SENDER_MAP
        .add(oob)
        .or(Err(VcxError::from(VcxErrorKind::CreateOutOfBand)))
}

#[napi]
pub fn create_out_of_band(config: String) -> ::napi::Result<u32> {
    trace!("create_out_of_band >>> config: {}", config);
    let config: OOBConfig = serde_json::from_str(&config).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!("Cannot deserialize out of band message config: {:?}", err),
        )
    })?;
    let mut oob = OutOfBandSender::create();
    if let Some(label) = &config.label {
        oob = oob.set_label(&label);
    };
    if let Some(goal) = &config.goal {
        oob = oob.set_goal(&goal);
    };
    if let Some(goal_code) = &config.goal_code {
        oob = oob.set_goal_code(&goal_code);
    };
    for protocol in config.handshake_protocols {
        oob = oob.append_handshake_protocol(&protocol)?;
    }
    store_out_of_band_sender(oob).map_err(|err| err.into())
}

#[napi]
pub fn create_out_of_band_msg_from_msg(msg: String) -> ::napi::Result<u32> {
    trace!("create_out_of_band_msg_from_msg >>> msg: {}", msg);
    let msg: A2AMessage = serde_json::from_str(&msg).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!("Cannot deserialize supplied message: {:?}", err),
        )
    })?;
    store_out_of_band_receiver(OutOfBandReceiver::create_from_a2a_msg(&msg)?).map_err(|err| err.into())
}

#[napi]
pub fn append_message(handle: u32, msg: String) -> ::napi::Result<()> {
    trace!("append_message >>> handle: {}, msg: {}", handle, msg);
    let mut oob = OUT_OF_BAND_SENDER_MAP.get_cloned(handle)?;
    let msg = serde_json::from_str(&msg).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!("Cannot deserialize supplied message: {:?}", err),
        )
    })?;
    oob = oob.clone().append_a2a_message(msg)?;
    OUT_OF_BAND_SENDER_MAP.insert(handle, oob).map_err(|err| err.into())
}

#[napi]
pub fn append_service(handle: u32, service: String) -> ::napi::Result<()> {
    trace!("append_service >>> handle: {}, service: {}", handle, service);
    let mut oob = OUT_OF_BAND_SENDER_MAP.get_cloned(handle)?;
    let service = serde_json::from_str(&service).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!("Cannot deserialize supplied message: {:?}", err),
        )
    })?;
    oob = oob.clone().append_service(&ServiceResolvable::AriesService(service));
    OUT_OF_BAND_SENDER_MAP.insert(handle, oob).map_err(|err| err.into())
}

#[napi]
pub fn append_service_did(handle: u32, did: String) -> ::napi::Result<()> {
    trace!("append_service_did >>> handle: {}, did: {}", handle, did);
    let mut oob = OUT_OF_BAND_SENDER_MAP.get_cloned(handle)?;
    oob = oob.clone().append_service(&ServiceResolvable::Did(
        Did::new(&did).map_err(|err| Into::<VcxError>::into(err))?,
    ));
    OUT_OF_BAND_SENDER_MAP.insert(handle, oob).map_err(|err| err.into())
}

pub fn get_services(handle: u32) -> ::napi::Result<Vec<ServiceResolvable>> {
    trace!("get_services >>> handle: {}", handle);
    OUT_OF_BAND_SENDER_MAP.get(handle, |oob| Ok(oob.get_services()))
        .map_err(|err| Into::<::napi::Error>::into(err))
}

#[napi]
pub fn extract_a2a_message(handle: u32) -> ::napi::Result<String> {
    trace!("extract_a2a_message >>> handle: {}", handle);
    OUT_OF_BAND_RECEIVER_MAP
        .get(handle, |oob| {
            if let Some(msg) = oob.extract_a2a_message()? {
                let msg = serde_json::to_string(&msg).map_err(|err| {
                    VcxError::from_msg(
                        VcxErrorKind::SerializationError,
                        format!("Cannot serialize message {:?}, err: {:?}", msg, err),
                    )
                })?;
                Ok(msg)
            } else {
                Ok("".to_string())
            }
        })
        .map_err(|err| err.into())
}

#[napi]
pub fn to_a2a_message(handle: u32) -> ::napi::Result<String> {
    OUT_OF_BAND_SENDER_MAP.get(handle, |oob| {
        let msg = oob.to_a2a_message();
        serde_json::to_string(&msg)
            .map_err(|err| {
                VcxError::from_msg(
                    VcxErrorKind::SerializationError,
                    format!("Cannot serialize message {:?}, err: {:?}", msg, err),
                )
            })
    })
    .map_err(|err| Into::<::napi::Error>::into(err))
}

#[napi]
pub async fn connection_exists(handle: u32, conn_handles: Vec<u32>) -> ::napi::Result<u32> {
    trace!(
        "connection_exists >>> handle: {}, conn_handles: {:?}",
        handle,
        conn_handles
    );
    let oob = OUT_OF_BAND_RECEIVER_MAP.get_cloned(handle)?;
    let mut conn_map = HashMap::new();
    for conn_handle in conn_handles {
        let connection = CONNECTION_MAP.get_cloned(conn_handle)?;
        conn_map.insert(conn_handle, connection);
    }
    let connections = conn_map.values().collect();

    if let Some(connection) = oob.connection_exists(get_main_pool_handle()?, &connections).await? {
        if let Some((&handle, _)) = conn_map.iter().find(|(_, conn)| *conn == connection) {
            Ok(handle)
        } else {
            Err(VcxError::from(VcxErrorKind::InvalidState).into())
        }
    } else {
        Ok(0)
    }
}

#[napi]
pub async fn build_connection(handle: u32) -> ::napi::Result<String> {
    let oob = OUT_OF_BAND_RECEIVER_MAP.get_cloned(handle)?;
    let invitation = Invitation::OutOfBand(oob.oob.clone());
    let ddo = into_did_doc(get_main_pool_handle()?, &invitation).await?;
    oob.build_connection(&get_main_agency_client().unwrap(), ddo, false)
        .await?
        .to_string()
        .map_err(|err| Into::<::napi::Error>::into(err))
}

#[napi]
pub fn get_thread_id_sender(handle: u32) -> ::napi::Result<String> {
    trace!("get_thread_id_sender >>> handle: {}", handle);
    OUT_OF_BAND_SENDER_MAP
        .get(handle, |oob| Ok(oob.get_id()))
        .map_err(|err| Into::<::napi::Error>::into(err))
}

#[napi]
pub fn get_thread_id_receiver(handle: u32) -> ::napi::Result<String> {
    trace!("get_thread_id_receiver >>> handle: {}", handle);
    OUT_OF_BAND_RECEIVER_MAP
        .get(handle, |oob| Ok(oob.get_id()))
        .map_err(|err| Into::<::napi::Error>::into(err))
}

#[napi]
pub fn to_string_sender(handle: u32) -> ::napi::Result<String> {
    OUT_OF_BAND_SENDER_MAP
        .get(handle, |oob| Ok(oob.to_string()))
        .map_err(|err| Into::<::napi::Error>::into(err))
}

#[napi]
pub fn to_string_receiver(handle: u32) -> ::napi::Result<String> {
    OUT_OF_BAND_RECEIVER_MAP
        .get(handle, |oob| Ok(oob.to_string()))
        .map_err(|err| Into::<::napi::Error>::into(err))
}

#[napi]
pub fn from_string_sender(oob_data: String) -> ::napi::Result<u32> {
    let oob = OutOfBandSender::from_string(&oob_data)?;
    OUT_OF_BAND_SENDER_MAP
        .add(oob)
        .map_err(|err| Into::<::napi::Error>::into(err))
}

#[napi]
pub fn from_string_receiver(oob_data: String) -> ::napi::Result<u32> {
    let oob = OutOfBandReceiver::from_string(&oob_data)?;
    OUT_OF_BAND_RECEIVER_MAP
        .add(oob)
        .map_err(|err| Into::<::napi::Error>::into(err))
}

#[napi]
pub fn release_sender(handle: u32) -> ::napi::Result<()> {
    OUT_OF_BAND_SENDER_MAP
        .release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidHandle)))
        .map_err(|err| Into::<::napi::Error>::into(err))
}

#[napi]
pub fn release_receiver(handle: u32) -> ::napi::Result<()> {
    OUT_OF_BAND_RECEIVER_MAP
        .release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidHandle)))
        .map_err(|err| Into::<::napi::Error>::into(err))
}

#[cfg(test)]
pub mod tests {
    use aries_vcx::messages::did_doc::service_aries::AriesService;

    use super::*;

    async fn build_and_append_service(did: &str) {
        let config = json!({
            "label": "foo",
            "goal_code": GoalCode::IssueVC,
            "goal": "foobar"
        })
        .to_string();
        let oob_handle = create_out_of_band(config).await.unwrap();
        assert!(oob_handle > 0);
        let service = ServiceResolvable::AriesService(
            AriesService::create()
                .set_service_endpoint("http://example.org/agent".into())
                .set_routing_keys(vec!["12345".into()])
                .set_recipient_keys(vec!["abcde".into()]),
        );
        append_service(oob_handle, json!(service).to_string()).unwrap();
        append_service_did(oob_handle, "V4SGRU86Z58d6TV7PBUe6f".to_string()).unwrap();
        let resolved_service = get_services(oob_handle).unwrap();
        assert_eq!(resolved_service.len(), 2);
        assert_eq!(service, resolved_service[0]);
        assert_eq!(
            ServiceResolvable::Did(Did::new(did).unwrap()),
            resolved_services[1]
        );
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_build_oob_sender_append_services() {
        build_and_append_service("V4SGRU86Z58d6TV7PBUe6f").await
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_build_oob_sender_append_services_prefix_did_sov() {
        build_and_append_service("did:sov:V4SGRU86Z58d6TV7PBUe6f").await
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_serde_oob_config_handshake_protocols() {
        let config_str = json!({ "handshake_protocols": vec!["ConnectionV1", "DidExchangeV1"] }).to_string();
        let config_actual: OOBConfig = serde_json::from_str(&config_str).unwrap();
        assert_eq!(
            config_actual.handshake_protocols,
            vec![HandshakeProtocol::ConnectionV1, HandshakeProtocol::DidExchangeV1]
        );

        let config_str = json!({}).to_string();
        let config_actual: OOBConfig = serde_json::from_str(&config_str).unwrap();
        assert_eq!(config_actual.handshake_protocols, vec![]);
    }
}
