use shared_vcx::validation::{did::validate_did, verkey::validate_verkey};

use crate::{
    errors::error::AgencyClientResult,
    messages::{a2a_message::A2AMessageKinds, message_type::MessageType},
};

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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CreateKeyResponse {
    #[serde(rename = "@type")]
    msg_type: MessageType,
    #[serde(rename = "withPairwiseDID")]
    pub(crate) for_did: String,
    #[serde(rename = "withPairwiseDIDVerKey")]
    pub(crate) for_verkey: String,
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
        validate_did(did)?;
        self.for_did = did.to_string();
        Ok(self)
    }

    pub fn for_verkey(&mut self, verkey: &str) -> AgencyClientResult<&mut Self> {
        trace!("CreateKeyBuilder::for_verkey >>> verkey: {}", verkey);
        validate_verkey(verkey)?;
        self.for_verkey = verkey.to_string();
        Ok(self)
    }

    pub fn build(&self) -> CreateKey {
        CreateKey {
            msg_type: MessageType::build_v2(A2AMessageKinds::CreateKey),
            for_did: self.for_did.to_string(),
            for_verkey: self.for_verkey.to_string(),
        }
    }
}

#[cfg(feature = "general_test")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{agency_client::AgencyClient, errors::error::AgencyClientErrorKind, testing::test_utils::SetupMocks};

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_key_set_values() {
        let for_did = "11235yBzrpJQmNyZzgoTqB";
        let for_verkey = "EkVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A";

        CreateKeyBuilder::create()
            .for_did(for_did)
            .unwrap()
            .for_verkey(for_verkey)
            .unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_parse_create_keys_v2_response() {
        let _setup = SetupMocks::init();

        let for_did = "11235yBzrpJQmNyZzgoTqB";
        let for_verkey = "EkVTa7SCJ5SntpYyX7CSb2pcBhiVGT9kWSagA8a9T69A";
        let client = AgencyClient::new();
        let (res_did, res_vk) = client.create_connection_agent(for_did, for_verkey).await.unwrap();

        assert_eq!(res_did, "MNepeSWtGfhnv8jLB1sFZC");
        assert_eq!(res_vk, "C73MRnns4qUjR5N4LRwTyiXVPKPrA5q4LCT8PZzxVdt9");
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_key_set_invalid_did_errors() {
        let for_did = "11235yBzrpJQmNyZzgoT";
        let res = CreateKeyBuilder::create().for_did(for_did).unwrap_err();
        assert_eq!(res.kind(), AgencyClientErrorKind::InvalidDid);
    }
}
