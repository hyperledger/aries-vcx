use std::ops::Deref;
use std::sync::{RwLock, RwLockWriteGuard};

use aries_vcx::agency_client::agency_client::AgencyClient;
use aries_vcx::agency_client::configuration::AgencyClientConfig;
use aries_vcx::error::{VcxResult, VcxError, VcxErrorKind};

use napi_derive::napi;

use crate::api_lib::global::wallet::get_main_wallet_handle;

lazy_static! {
    pub static ref AGENCY_CLIENT: RwLock<AgencyClient> = RwLock::new(AgencyClient::new());
}

pub fn get_main_agency_client_mut() -> VcxResult<RwLockWriteGuard<'static, AgencyClient>> {
    let agency_client = AGENCY_CLIENT.write()?;
    Ok(agency_client)
}

pub fn get_main_agency_client() -> VcxResult<AgencyClient> {
    let agency_client = AGENCY_CLIENT.read()?.deref().clone();
    Ok(agency_client)
}

#[napi]
pub fn create_agency_client_for_main_wallet(config: String) -> ::napi::Result<()> {
    let config = serde_json::from_str::<AgencyClientConfig>(&config)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Serialization error: {:?}", err)))?;
    let client = get_main_agency_client()?.configure(get_main_wallet_handle(), &config)
        .map_err(|err| Into::<VcxError>::into(err))?;
    set_main_agency_client(client);
    Ok(())
}

pub fn reset_main_agency_client() {
    trace!("reset_agency_client >>>");
    let mut agency_client = AGENCY_CLIENT.write().unwrap();
    *agency_client = AgencyClient::new();
}

pub fn set_main_agency_client(new_agency_client: AgencyClient) {
    trace!("set_main_agency_client >>>");
    let mut agency_client = AGENCY_CLIENT.write().unwrap();
    *agency_client = new_agency_client;
}
