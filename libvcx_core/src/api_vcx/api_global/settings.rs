use aries_vcx::agency_client::testing::mocking::enable_agency_mocks;
use aries_vcx::aries_vcx_core::wallet::indy::IssuerConfig;
use aries_vcx::global::settings;
use aries_vcx::global::settings::aries_vcx_enable_indy_mocks;

use crate::errors::error::LibvcxResult;

use crate::errors::mapping_from_ariesvcx::map_ariesvcx_result;

pub fn enable_mocks() -> LibvcxResult<()> {
    enable_agency_mocks();
    map_ariesvcx_result(aries_vcx_enable_indy_mocks())
}

pub fn get_config_value(key: &str) -> LibvcxResult<String> {
    map_ariesvcx_result(settings::get_config_value(key))
}
