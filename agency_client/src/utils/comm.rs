use crate::{agency_settings, AgencyClientResult, httpclient};

pub async fn post_to_agency(body_content: &Vec<u8>) -> AgencyClientResult<Vec<u8>> {
    let endpoint = agency_settings::get_config_value(agency_settings::CONFIG_AGENCY_ENDPOINT)?;
    httpclient::post_message(body_content, &endpoint).await
}
