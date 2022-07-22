use std::ops::Deref;
use std::sync::{RwLock, RwLockWriteGuard};

use agency_client::agency_client::AgencyClient;
use agency_client::configuration::AgencyClientConfig;
use agency_client::testing::mocking;

use crate::error::VcxResult;

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

pub fn create_agency_client_for_main_wallet(config: &AgencyClientConfig) -> VcxResult<()> {
    get_main_agency_client_mut()?
        .configure(config)?;
    Ok(())
}

pub fn enable_main_agency_client_mocks() -> VcxResult<()> {
    info!("enable_agency_mocks >>>");
    mocking::enable_agency_mocks();
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
