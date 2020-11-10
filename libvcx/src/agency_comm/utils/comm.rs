use agency_comm::agency_settings;
use agency_comm::utils::error::prelude::*;
use agency_comm::httpclient;

pub fn post_to_agency(body_content: &Vec<u8>) -> VcxResult<Vec<u8>> {
    let endpoint = format!("{}/agency/msg", agency_settings::get_config_value(agency_settings::CONFIG_AGENCY_ENDPOINT)?);
    httpclient::post_message(body_content, &endpoint)
}
