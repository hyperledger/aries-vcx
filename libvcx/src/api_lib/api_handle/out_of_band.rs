use std::collections::HashMap;

use crate::aries_vcx::handlers::out_of_band::GoalCode;
use crate::aries_vcx::handlers::out_of_band::sender::sender::OutOfBandSender;
use crate::aries_vcx::handlers::out_of_band::receiver::receiver::OutOfBandReceiver;
use crate::aries_vcx::messages::connection::service::{ServiceResolvable, FullService};
use crate::aries_vcx::messages::a2a::A2AMessage;
use crate::api_lib::api_handle::object_cache::ObjectCache;
use crate::api_lib::api_handle::connection::CONNECTION_MAP;
use crate::error::prelude::*;

lazy_static! {
    pub static ref OUT_OF_BAND_SENDER_MAP: ObjectCache<OutOfBandSender> = ObjectCache::<OutOfBandSender>::new("out-of-band-sender-cache");
    pub static ref OUT_OF_BAND_RECEIVER_MAP: ObjectCache<OutOfBandReceiver> = ObjectCache::<OutOfBandReceiver>::new("out-of-band-receiver-cache");
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OOBConfig {
    pub label: Option<String>,
    pub goal_code: Option<GoalCode>,
    pub goal: Option<String>,
}

pub fn is_valid_handle(handle: u32) -> bool {
    OUT_OF_BAND_SENDER_MAP.has_handle(handle) || OUT_OF_BAND_RECEIVER_MAP.has_handle(handle)
}

fn store_out_of_band_receiver(oob: OutOfBandReceiver) -> VcxResult<u32> {
    OUT_OF_BAND_RECEIVER_MAP.add(oob)
        .or(Err(VcxError::from(VcxErrorKind::CreateOutOfBand)))
}

fn store_out_of_band_sender(oob: OutOfBandSender) -> VcxResult<u32> {
    OUT_OF_BAND_SENDER_MAP.add(oob)
        .or(Err(VcxError::from(VcxErrorKind::CreateOutOfBand)))
}

pub fn create_out_of_band(config: &str) -> VcxResult<u32> {
    trace!("create_out_of_band >>> config: {}", config);
    let config: OOBConfig = serde_json::from_str(config)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize out of band message config: {:?}", err)))?;
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
    store_out_of_band_sender(oob)
}

pub fn create_out_of_band_msg_from_msg(msg: &str) -> VcxResult<u32> {
    trace!("create_out_of_band_msg_from_msg >>> msg: {}", msg);
    let msg: A2AMessage = serde_json::from_str(msg)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize supplied message: {:?}", err)))?;
    store_out_of_band_receiver(OutOfBandReceiver::create_from_a2a_msg(&msg)?)
}

pub fn append_message(handle: u32, msg: &str) -> VcxResult<()> {
    trace!("append_message >>> handle: {}, msg: {}", handle, msg);
    OUT_OF_BAND_SENDER_MAP.get_mut(handle, |oob| {
        let msg: A2AMessage = serde_json::from_str(msg)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize supplied message: {:?}", err)))?;
        oob.append_a2a_message(msg).map_err(|err| err.into())
    })
}

pub fn append_service(handle: u32, service: &str) -> VcxResult<()> {
    trace!("append_service >>> handle: {}, service: {}", handle, service);
    OUT_OF_BAND_SENDER_MAP.get_mut(handle, |oob| {
        let service: FullService = serde_json::from_str(service)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize service: {:?}", err)))?;
        oob.append_service(&ServiceResolvable::FullService(service)).map_err(|err| err.into())
    })
}

pub fn extract_a2a_message(handle: u32) -> VcxResult<String> {
    trace!("extract_a2a_message >>> handle: {}", handle);
    OUT_OF_BAND_RECEIVER_MAP.get(handle, |oob| {
        if let Some(msg) = oob.extract_a2a_message()? {
            let msg = serde_json::to_string(&msg)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize message {:?}, err: {:?}", msg, err)))?;
            Ok(msg)
        } else {
            Ok("".to_string())
        }
    })
}

pub fn to_a2a_message(handle: u32) -> VcxResult<String> {
    OUT_OF_BAND_SENDER_MAP.get(handle, |oob| {
        let msg = oob.to_a2a_message();
        Ok(serde_json::to_string(&msg)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Cannot serialize message {:?}, err: {:?}", msg, err)))?)
    })
}

pub fn connection_exists(handle: u32, conn_handles: Vec<u32>) -> VcxResult<(u32, bool)> {
    trace!("connection_exists >>> handle: {}, conn_handles: {:?}", handle, conn_handles);
    let mut conn_map = HashMap::new();
    for conn_handle in conn_handles {
        let connection = CONNECTION_MAP.get(conn_handle, |connection| {
                Ok(connection.clone())
            },
        )?;
        conn_map.insert(conn_handle, connection);
    };
    let connections = conn_map.values().collect();
    OUT_OF_BAND_RECEIVER_MAP.get(handle, |oob| {
        if let Some(connection) = oob.connection_exists(&connections)? {
            if let Some((&handle, _)) = conn_map
                .iter()
                .find(|(_, conn)| *conn == connection) {
                    Ok((handle, true))
                } else {
                    Err(VcxError::from(VcxErrorKind::InvalidState))
                }
        } else {
            Ok((0, false))
        }
    })
}

pub fn build_connection(handle: u32) -> VcxResult<String> {
    OUT_OF_BAND_RECEIVER_MAP.get(handle, |oob| {
        oob.build_connection(false)?.to_string().map_err(|err| err.into())
    })
}

pub fn to_string_sender(handle: u32) -> VcxResult<String> {
    OUT_OF_BAND_SENDER_MAP.get(handle, |oob| {
        oob.to_string().map_err(|err| err.into())
    })
}

pub fn to_string_receiver(handle: u32) -> VcxResult<String> {
    OUT_OF_BAND_RECEIVER_MAP.get(handle, |oob| {
        oob.to_string().map_err(|err| err.into())
    })
}

pub fn from_string_sender(oob_data: &str) -> VcxResult<u32> {
    let oob = OutOfBandSender::from_string(oob_data)?;
    OUT_OF_BAND_SENDER_MAP.add(oob).map_err(|err| err.into())
}

pub fn from_string_receiver(oob_data: &str) -> VcxResult<u32> {
    let oob = OutOfBandReceiver::from_string(oob_data)?;
    OUT_OF_BAND_RECEIVER_MAP.add(oob).map_err(|err| err.into())
}

pub fn release_sender(handle: u32) -> VcxResult<()> {
    OUT_OF_BAND_SENDER_MAP.release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidHandle)))
}

pub fn release_receiver(handle: u32) -> VcxResult<()> {
    OUT_OF_BAND_RECEIVER_MAP.release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidHandle)))
}
