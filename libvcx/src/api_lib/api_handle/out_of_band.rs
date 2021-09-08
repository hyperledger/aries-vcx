use crate::aries_vcx::handlers::out_of_band::{OutOfBand, GoalCode, HandshakeProtocol};
use crate::aries_vcx::messages::a2a::message_type::MessageType;
use crate::aries_vcx::messages::connection::service::ServiceResolvable;
use crate::aries_vcx::messages::a2a::A2AMessage;
use crate::api_lib::api_handle::object_cache::ObjectCache;
use crate::error::prelude::*;

lazy_static! {
    pub static ref OUT_OF_BAND_MAP: ObjectCache<OutOfBand> = ObjectCache::<OutOfBand>::new("out-of-band-cache");
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OOBConfig {
    pub label: Option<String>,
    pub goal_code: Option<GoalCode>,
    pub goal: Option<String>,
}

pub fn is_valid_handle(handle: u32) -> bool {
    OUT_OF_BAND_MAP.has_handle(handle)
}

fn store_out_of_band(oob: OutOfBand) -> VcxResult<u32> {
    OUT_OF_BAND_MAP.add(oob)
        .or(Err(VcxError::from(VcxErrorKind::CreateOutOfBand)))
}

pub fn create_out_of_band_msg(config: &str) -> VcxResult<u32> {
    trace!("create_out_of_band_msg >>> config: {:?}", config);
    let config: OOBConfig = serde_json::from_str(config)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize out of band message config: {:?}", err)))?;
    let mut oob = OutOfBand::create();
    if let Some(label) = &config.label {
        oob = oob.set_label(&label);
    };
    if let Some(goal) = &config.goal {
        oob = oob.set_goal(&goal);
    };
    if let Some(goal_code) = &config.goal_code {
        oob = oob.set_goal_code(&goal_code);
    };
    return store_out_of_band(oob);
}

pub fn append_message(handle: u32, msg: &str) -> VcxResult<()> {
    trace!("append_message >>> handle: {}, msg: {}", handle, msg);
    OUT_OF_BAND_MAP.get_mut(handle, |oob| {
        let msg: A2AMessage = serde_json::from_str(msg)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize supplied message: {:?}", err)))?;
        oob.append_a2a_message(msg).map_err(|err| err.into())
    })
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    OUT_OF_BAND_MAP.get(handle, |oob| {
        oob.to_string().map_err(|err| err.into())
    })
}

pub fn from_string(oob_data: &str) -> VcxResult<u32> {
    let oob = OutOfBand::from_string(oob_data)?;
    OUT_OF_BAND_MAP.add(oob).map_err(|err| err.into())
}

pub fn release(handle: u32) -> VcxResult<()> {
    OUT_OF_BAND_MAP.release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidHandle)))
}
