use aries_vcx::{global::settings::enable_indy_mocks, agency_client::testing::mocking::enable_agency_mocks};
use napi_derive::napi;

pub mod agency_client;
pub mod wallet;
pub mod pool;

#[napi]
pub fn enable_mocks() -> ::napi::Result<()> {
    enable_indy_mocks()
        .map_err(|err| Into::<::napi::Error>::into(err))?;
    enable_agency_mocks();
    Ok(())
}
