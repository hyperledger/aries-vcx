use crate::utils::error::{VcxErrorKind, VcxResult, VcxError};
use crate::{A2AMessageV2, A2AMessage, parse_response_from_agency, prepare_message_for_agency, agency_settings, A2AMessageKinds, mocking};
use crate::message_type::MessageTypes;
use crate::utils::comm::post_to_agency;
use crate::utils::{constants, validation};
use crate::mocking::AgencyMock;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateKey {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "forDID")]
    for_did: String,
    #[serde(rename = "forDIDVerKey")]
    for_verkey: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateKeyResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
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

    pub fn for_did(&mut self, did: &str) -> VcxResult<&mut Self> {
        validation::validate_did(did)?;
        self.for_did = did.to_string();
        Ok(self)
    }

    pub fn for_verkey(&mut self, verkey: &str) -> VcxResult<&mut Self> {
        validation::validate_verkey(verkey)?;
        self.for_verkey = verkey.to_string();
        Ok(self)
    }

    pub fn send_secure(&self) -> VcxResult<(String, String)> {
        trace!("CreateKeyBuilder::send_secure >>>");

        if mocking::agency_mocks_enabled() {
            AgencyMock::set_next_response(constants::CREATE_KEYS_V2_RESPONSE.to_vec());
        }

        let data = self.prepare_request()?;

        let response = post_to_agency(&data)?;

        self.parse_response(&response)
    }

    fn prepare_request(&self) -> VcxResult<Vec<u8>> {
        let message = A2AMessage::Version2(
            A2AMessageV2::CreateKey(CreateKey {
                msg_type: MessageTypes::MessageType(MessageTypes::build_v2(A2AMessageKinds::CreateKey)),
                for_did: self.for_did.to_string(),
                for_verkey: self.for_verkey.to_string(),
            })
        );

        let agency_did = agency_settings::get_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_DID)?;

        prepare_message_for_agency(&message, &agency_did)
    }

    fn parse_response(&self, response: &Vec<u8>) -> VcxResult<(String, String)> {
        let mut response = parse_response_from_agency(response)?;
        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::CreateKeyResponse(res)) => Ok((res.for_did, res.for_verkey)),
            _ => Err(VcxError::from(VcxErrorKind::InvalidHttpResponse))
        }
    }
}

#[cfg(test)]
mod tests {
    // use agency_comm::create_keys;
    // use utils::devsetup::*;

    use super::*;
    use crate::utils::error::VcxErrorKind;
    use crate::create_keys;
    use crate::utils::constants;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_key_set_values() {
        let _setup = SetupDefaults::init();

        let for_did = "11235yBzrpJQmNyZzgoTqB";
        let for_verkey = "EkVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A";

        create_keys()
            .for_did(for_did).unwrap()
            .for_verkey(for_verkey).unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_parse_create_keys_v2_response() {
        let _setup = SetupMocks::init();

        let mut builder = create_keys();

        let (for_did, for_verkey) = builder.parse_response(&constants::CREATE_KEYS_V2_RESPONSE.to_vec()).unwrap();

        assert_eq!(for_did, "MNepeSWtGfhnv8jLB1sFZC");
        assert_eq!(for_verkey, "C73MRnns4qUjR5N4LRwTyiXVPKPrA5q4LCT8PZzxVdt9");
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_key_set_invalid_did_errors() {
        let _setup = SetupDefaults::init();

        let for_did = "11235yBzrpJQmNyZzgoT";
        let res = create_keys()
            .for_did(for_did)
            .unwrap_err();
        assert_eq!(res.kind(), VcxErrorKind::InvalidDid);
    }
}

