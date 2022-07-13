use crate::{agency_settings, AgencyClientError, AgencyClientErrorKind, AgencyClientResult, Client2AgencyMessage, CreateKeyBuilder, parse_response_from_agency, prepare_message_for_agency};
use crate::testing::{mocking, test_constants};
use crate::testing::mocking::AgencyMock;
use crate::utils::comm::post_to_agency;

pub async fn create_keys(pw_did: &str, pw_verkey: &str) -> AgencyClientResult<(String, String)> {
    trace!("create_keys >>> pw_did: {}, pw_verkey: {}", pw_did, pw_verkey);

    if mocking::agency_mocks_enabled() {
        warn!("CreateKeyBuilder::send_secure >>> agency mocks enabled, setting next mocked response");
        AgencyMock::set_next_response(test_constants::CREATE_KEYS_V2_RESPONSE.to_vec());
    }

    let message = CreateKeyBuilder::create()
        .for_did(pw_did)?
        .for_verkey(pw_verkey)?
        .build();

    let agency_did = agency_settings::get_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_DID)?;

    let data = prepare_message_for_agency(&Client2AgencyMessage::CreateKey(message), &agency_did).await?;
    let response = post_to_agency(&data).await?;
    let mut response = parse_response_from_agency(&response).await?;

    match response.remove(0) {
        Client2AgencyMessage::CreateKeyResponse(res) => Ok((res.for_did, res.for_verkey)),
        _ => Err(AgencyClientError::from(AgencyClientErrorKind::InvalidHttpResponse))
    }
}
