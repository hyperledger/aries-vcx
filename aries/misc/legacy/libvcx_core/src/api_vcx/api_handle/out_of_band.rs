use aries_vcx::{
    handlers::out_of_band::{receiver::OutOfBandReceiver, sender::OutOfBandSender},
    messages::{
        msg_fields::protocols::out_of_band::{invitation::OobService, OobGoalCode},
        msg_types::Protocol,
        AriesMessage,
    },
};

use crate::{
    api_vcx::api_handle::object_cache::ObjectCache,
    errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
};

lazy_static! {
    pub static ref OUT_OF_BAND_SENDER_MAP: ObjectCache<OutOfBandSender> =
        ObjectCache::<OutOfBandSender>::new("out-of-band-sender-cache");
    pub static ref OUT_OF_BAND_RECEIVER_MAP: ObjectCache<OutOfBandReceiver> =
        ObjectCache::<OutOfBandReceiver>::new("out-of-band-receiver-cache");
}

#[derive(Deserialize)]
pub struct OOBConfig {
    pub label: Option<String>,
    pub goal_code: Option<OobGoalCode>,
    pub goal: Option<String>,
    #[serde(default)]
    pub handshake_protocols: Vec<Protocol>,
}

fn store_out_of_band_receiver(oob: OutOfBandReceiver) -> LibvcxResult<u32> {
    OUT_OF_BAND_RECEIVER_MAP
        .add(oob)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::CreateOutOfBand, e.to_string()))
}

fn store_out_of_band_sender(oob: OutOfBandSender) -> LibvcxResult<u32> {
    OUT_OF_BAND_SENDER_MAP
        .add(oob)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::CreateOutOfBand, e.to_string()))
}

pub fn create_out_of_band(config: &str) -> LibvcxResult<u32> {
    trace!("create_out_of_band >>> config: {}", config);
    let config: OOBConfig = serde_json::from_str(config).map_err(|err| {
        LibvcxError::from_msg(
            LibvcxErrorKind::InvalidJson,
            format!("Cannot deserialize out of band message config: {:?}", err),
        )
    })?;
    let mut oob = OutOfBandSender::create();
    if let Some(label) = &config.label {
        oob = oob.set_label(label);
    };
    if let Some(goal) = &config.goal {
        oob = oob.set_goal(goal);
    };
    if let Some(goal_code) = &config.goal_code {
        oob = oob.set_goal_code(*goal_code);
    };
    for protocol in config.handshake_protocols {
        oob = oob.append_handshake_protocol(protocol)?;
    }
    store_out_of_band_sender(oob)
}

pub fn create_out_of_band_msg_from_msg(msg: &str) -> LibvcxResult<u32> {
    trace!("create_out_of_band_msg_from_msg >>> msg: {}", msg);
    let msg: AriesMessage = serde_json::from_str(msg).map_err(|err| {
        LibvcxError::from_msg(
            LibvcxErrorKind::InvalidJson,
            format!("Cannot deserialize supplied message: {:?}", err),
        )
    })?;
    store_out_of_band_receiver(OutOfBandReceiver::create_from_a2a_msg(&msg)?)
}

pub fn append_message(handle: u32, msg: &str) -> LibvcxResult<()> {
    trace!("append_message >>> handle: {}, msg: {}", handle, msg);
    let mut oob = OUT_OF_BAND_SENDER_MAP.get_cloned(handle)?;
    let msg = serde_json::from_str(msg).map_err(|err| {
        LibvcxError::from_msg(
            LibvcxErrorKind::InvalidJson,
            format!("Cannot deserialize supplied message: {:?}", err),
        )
    })?;
    oob = oob.clone().append_a2a_message(msg)?;
    OUT_OF_BAND_SENDER_MAP.insert(handle, oob)
}

pub fn append_service(handle: u32, service: &str) -> LibvcxResult<()> {
    trace!(
        "append_service >>> handle: {}, service: {}",
        handle,
        service
    );
    let mut oob = OUT_OF_BAND_SENDER_MAP.get_cloned(handle)?;
    let service = serde_json::from_str(service).map_err(|err| {
        LibvcxError::from_msg(
            LibvcxErrorKind::InvalidJson,
            format!("Cannot deserialize supplied message: {:?}", err),
        )
    })?;
    oob = oob
        .clone()
        .append_service(&OobService::AriesService(service));
    OUT_OF_BAND_SENDER_MAP.insert(handle, oob)
}

pub fn append_service_did(handle: u32, did: &str) -> LibvcxResult<()> {
    trace!("append_service_did >>> handle: {}, did: {}", handle, did);
    let mut oob = OUT_OF_BAND_SENDER_MAP.get_cloned(handle)?;
    oob = oob
        .clone()
        .append_service(&OobService::Did(did.to_string()));
    OUT_OF_BAND_SENDER_MAP.insert(handle, oob)
}

pub fn get_services(handle: u32) -> LibvcxResult<Vec<OobService>> {
    trace!("get_services >>> handle: {}", handle);
    OUT_OF_BAND_SENDER_MAP.get(handle, |oob| Ok(oob.get_services()))
}

pub fn extract_a2a_message(handle: u32) -> LibvcxResult<String> {
    trace!("extract_a2a_message >>> handle: {}", handle);
    OUT_OF_BAND_RECEIVER_MAP.get(handle, |oob| {
        if let Some(msg) = oob.extract_a2a_message()? {
            let msg = serde_json::to_string(&msg).map_err(|err| {
                LibvcxError::from_msg(
                    LibvcxErrorKind::SerializationError,
                    format!("Cannot serialize message {:?}, err: {:?}", msg, err),
                )
            })?;
            Ok(msg)
        } else {
            Ok("".to_string())
        }
    })
}

pub fn to_a2a_message(handle: u32) -> LibvcxResult<String> {
    OUT_OF_BAND_SENDER_MAP.get(handle, |oob| {
        let msg = oob.to_aries_message();
        serde_json::to_string(&msg).map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::SerializationError,
                format!("Cannot serialize message {:?}, err: {:?}", msg, err),
            )
        })
    })
}

pub fn get_thread_id_sender(handle: u32) -> LibvcxResult<String> {
    trace!("get_thread_id_sender >>> handle: {}", handle);
    OUT_OF_BAND_SENDER_MAP.get(handle, |oob| Ok(oob.get_id()))
}

pub fn get_thread_id_receiver(handle: u32) -> LibvcxResult<String> {
    trace!("get_thread_id_receiver >>> handle: {}", handle);
    OUT_OF_BAND_RECEIVER_MAP.get(handle, |oob| Ok(oob.get_id()))
}

pub fn to_string_sender(handle: u32) -> LibvcxResult<String> {
    OUT_OF_BAND_SENDER_MAP.get(handle, |oob| Ok(oob.to_string()))
}

pub fn to_string_receiver(handle: u32) -> LibvcxResult<String> {
    OUT_OF_BAND_RECEIVER_MAP.get(handle, |oob| Ok(oob.to_string()))
}

pub fn from_string_sender(oob_data: &str) -> LibvcxResult<u32> {
    let oob = OutOfBandSender::from_string(oob_data)?;
    OUT_OF_BAND_SENDER_MAP.add(oob)
}

pub fn from_string_receiver(oob_data: &str) -> LibvcxResult<u32> {
    let oob = OutOfBandReceiver::from_string(oob_data)?;
    OUT_OF_BAND_RECEIVER_MAP.add(oob)
}

pub fn release_sender(handle: u32) -> LibvcxResult<()> {
    OUT_OF_BAND_SENDER_MAP
        .release(handle)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::InvalidHandle, e.to_string()))
}

pub fn release_receiver(handle: u32) -> LibvcxResult<()> {
    OUT_OF_BAND_RECEIVER_MAP
        .release(handle)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::InvalidHandle, e.to_string()))
}

#[cfg(test)]

mod tests {
    use aries_vcx::messages::msg_types::connection::{ConnectionType, ConnectionTypeV1};
    use diddoc_legacy::aries::service::AriesService;

    use super::*;

    async fn build_and_append_service(did: &str) {
        let config = json!({
            "label": "foo",
            "goal_code": OobGoalCode::IssueVC,
            "goal": "foobar"
        })
        .to_string();
        let oob_handle = create_out_of_band(&config).unwrap();
        assert!(oob_handle > 0);
        let service = OobService::AriesService(
            AriesService::create()
                .set_service_endpoint("http://example.org/agent".parse().expect("valid url"))
                .set_routing_keys(vec!["12345".into()])
                .set_recipient_keys(vec!["abcde".into()]),
        );
        append_service(oob_handle, &json!(service).to_string()).unwrap();
        append_service_did(oob_handle, did).unwrap();
        let resolved_services = get_services(oob_handle).unwrap();
        assert_eq!(resolved_services.len(), 2);
        assert_eq!(service, resolved_services[0]);
        assert_eq!(OobService::Did(did.to_owned()), resolved_services[1]);
    }

    #[tokio::test]
    async fn test_build_oob_sender_append_services() {
        build_and_append_service("V4SGRU86Z58d6TV7PBUe6f").await
    }

    #[tokio::test]
    async fn test_build_oob_sender_append_services_prefix_did_sov() {
        build_and_append_service("did:sov:V4SGRU86Z58d6TV7PBUe6f").await
    }

    #[test]
    fn test_serde_oob_config_handshake_protocols() {
        let config_str =
            json!({ "handshake_protocols": vec!["https://didcomm.org/connections/1.0"] })
                .to_string();
        let config_actual: OOBConfig = serde_json::from_str(&config_str).unwrap();
        assert_eq!(
            config_actual.handshake_protocols,
            vec![Protocol::ConnectionType(ConnectionType::V1(
                ConnectionTypeV1::new_v1_0()
            ))]
        );

        let config_str = json!({}).to_string();
        let config_actual: OOBConfig = serde_json::from_str(&config_str).unwrap();
        assert_eq!(config_actual.handshake_protocols, vec![]);
    }
}
