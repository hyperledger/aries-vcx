use agency_comm::agency_settings;
use error::VcxResult;
use utils;

pub fn post_to_agency(body_content: &Vec<u8>) -> VcxResult<Vec<u8>> {
    let endpoint = format!("{}/agency/msg", agency_settings::get_config_value(agency_settings::CONFIG_AGENCY_ENDPOINT)?);
    utils::httpclient::post_message(body_content, &endpoint)
}
