use aries_vcx::agency_client::testing::mocking::enable_agency_mocks;
use aries_vcx::errors::error::VcxResult;
use aries_vcx::global::settings;
use aries_vcx::global::settings::enable_indy_mocks;
use aries_vcx::global::settings::init_issuer_config;
use aries_vcx::indy::wallet::IssuerConfig;

use crate::api_lib::errors::error::LibvcxResult;

use crate::api_lib::errors::mapping_from_ariesvcx::map_ariesvcx_result;

pub fn vcxcore_enable_mocks() -> VcxResult<()> {
    enable_agency_mocks();
    enable_indy_mocks()
}

pub fn get_config_value(key: &str) -> LibvcxResult<String> {
    map_ariesvcx_result(settings::get_config_value(key))
}

pub fn settings_init_issuer_config(issuer_config: &IssuerConfig) -> LibvcxResult<()> {
    map_ariesvcx_result(init_issuer_config(issuer_config))
}
