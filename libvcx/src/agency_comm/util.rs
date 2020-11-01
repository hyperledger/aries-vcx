use agency_comm::agency_settings;
use error::VcxResult;
use utils;

//Todo: change this RC to a u32
pub fn post_u8(body_content: &Vec<u8>) -> VcxResult<Vec<u8>> {
    let endpoint = format!("{}/agency/msg", agency_settings::get_config_value(agency_settings::CONFIG_AGENCY_ENDPOINT)?);
    utils::httpclient::post_message(body_content, &endpoint)
}
