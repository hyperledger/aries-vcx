use crate::{agency_settings, parse_response_from_agency, prepare_message_for_agency, validation};
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::message_type::MessageType;
use crate::messages::a2a_message::{AgencyMsg, A2AMessageKinds, AgencyMessageTypes};
use crate::testing::{mocking, test_constants};
use crate::testing::mocking::AgencyMock;
use crate::utils::comm::post_to_agency;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateKey {
    #[serde(rename = "@type")]
    msg_type: MessageType,
    #[serde(rename = "forDID")]
    for_did: String,
    #[serde(rename = "forDIDVerKey")]
    for_verkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateKeyResponse {
    #[serde(rename = "@type")]
    msg_type: MessageType,
    #[serde(rename = "withPairwiseDID")]
    for_did: String,
    #[serde(rename = "withPairwiseDIDVerKey")]
    for_verkey: String,
}

#[derive(Debug)]
pub struct CreateKeyBuilder {
    for_did: String,
    for_verkey: String,
}

impl CreateKeyBuilder {
    pub fn create() -> CreateKeyBuilder {
        trace!("CreateKeyBuilder::create_message >>>");

        CreateKeyBuilder {
            for_did: String::new(),
            for_verkey: String::new(),
        }
    }

    pub fn for_did(&mut self, did: &str) -> AgencyClientResult<&mut Self> {
        trace!("CreateKeyBuilder::for_did >>> did: {}", did);
        validation::validate_did(did)?;
        self.for_did = did.to_string();
        Ok(self)
    }

    pub fn for_verkey(&mut self, verkey: &str) -> AgencyClientResult<&mut Self> {
        trace!("CreateKeyBuilder::for_verkey >>> verkey: {}", verkey);
        validation::validate_verkey(verkey)?;
        self.for_verkey = verkey.to_string();
        Ok(self)
    }

    pub async fn send_secure(&self) -> AgencyClientResult<(String, String)> {
        trace!("CreateKeyBuilder::send_secure >>>");

        if mocking::agency_mocks_enabled() {
            warn!("CreateKeyBuilder::send_secure >>> agency mocks enabled, setting next mocked response");
            AgencyMock::set_next_response(test_constants::CREATE_KEYS_V2_RESPONSE.to_vec());
        }

        let data = self.prepare_request().await?;

        let response = post_to_agency(&data).await?;

        self.parse_response(&response).await
    }

    async fn prepare_request(&self) -> AgencyClientResult<Vec<u8>> {
        trace!("CreateKeyBuilder::prepare_request >>>");
        let message = AgencyMsg::Version2(
            AgencyMessageTypes::CreateKey(CreateKey {
                msg_type: MessageType::build_v2(A2AMessageKinds::CreateKey),
                for_did: self.for_did.to_string(),
                for_verkey: self.for_verkey.to_string(),
            })
        );

        let agency_did = agency_settings::get_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_DID)?;

        prepare_message_for_agency(&message, &agency_did).await
    }

    async fn parse_response(&self, response: &Vec<u8>) -> AgencyClientResult<(String, String)> {
        let mut response = parse_response_from_agency(response).await?;
        match response.remove(0) {
            AgencyMsg::Version2(AgencyMessageTypes::CreateKeyResponse(res)) => Ok((res.for_did, res.for_verkey)),
            _ => Err(AgencyClientError::from(AgencyClientErrorKind::InvalidHttpResponse))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::create_keys;
    use crate::error::AgencyClientErrorKind;
    use crate::testing::test_constants;
    use crate::testing::test_utils::SetupMocks;

    use super::*;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_key_set_values() {
        let for_did = "11235yBzrpJQmNyZzgoTqB";
        let for_verkey = "EkVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A";

        CreateKeyBuilder::create()
            .for_did(for_did).unwrap()
            .for_verkey(for_verkey).unwrap();
    }

    #[async_std::test]
    #[cfg(feature = "general_test")]
    async fn test_create_key_set_values_and_serialize() {
        let _setup = SetupMocks::init();
        let to_did = "8XFh8yBzrpJQmNyZzgoTqB";
        let my_vk = "C73MRnns4qUjR5N4LRwTyiXVPKPrA5q4LCT8PZzxVdt9";
        let bytes = CreateKeyBuilder::create()
            .for_did(&to_did).unwrap()
            .for_verkey(&my_vk).unwrap()
            .prepare_request().await.unwrap();
        assert!(bytes.len() > 0);
    }

    #[async_std::test]
    #[cfg(feature = "general_test")]
    async fn test_parse_create_keys_v2_response() {
        let _setup = SetupMocks::init();

        let mut builder = CreateKeyBuilder::create();

        let (for_did, for_verkey) = builder.parse_response(&test_constants::CREATE_KEYS_V2_RESPONSE.to_vec()).await.unwrap();

        assert_eq!(for_did, "MNepeSWtGfhnv8jLB1sFZC");
        assert_eq!(for_verkey, "C73MRnns4qUjR5N4LRwTyiXVPKPrA5q4LCT8PZzxVdt9");
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_key_set_invalid_did_errors() {
        let for_did = "11235yBzrpJQmNyZzgoT";
        let res = CreateKeyBuilder::create()
            .for_did(for_did)
            .unwrap_err();
        assert_eq!(res.kind(), AgencyClientErrorKind::InvalidDid);
    }
}

