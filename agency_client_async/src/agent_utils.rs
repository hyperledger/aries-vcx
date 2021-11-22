use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::{A2AMessage, A2AMessageKinds, A2AMessageV2, agency_settings, parse_response_from_agency, prepare_message_for_agency};
use crate::error::{AgencyClientError, AgencyClientErrorKind, AgencyClientResult};
use crate::message_type::MessageTypes;
use crate::mocking::{agency_mocks_enabled, AgencyMockDecrypted};
use crate::utils::{constants, error_utils};
use crate::utils::comm::post_to_agency;

#[derive(Serialize, Deserialize, Debug)]
pub struct Connect {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "fromDID")]
    from_did: String,
    #[serde(rename = "fromDIDVerKey")]
    from_vk: String,
}

impl Connect {
    fn build(from_did: &str, from_vk: &str) -> Connect {
        Connect {
            msg_type: MessageTypes::build(A2AMessageKinds::Connect),
            from_did: from_did.to_string(),
            from_vk: from_vk.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "withPairwiseDID")]
    from_did: String,
    #[serde(rename = "withPairwiseDIDVerKey")]
    from_vk: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignUp {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
}

impl SignUp {
    fn build() -> SignUp {
        SignUp {
            msg_type: MessageTypes::build(A2AMessageKinds::SignUp),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignUpResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAgent {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
}

impl CreateAgent {
    fn build() -> CreateAgent {
        CreateAgent {
            msg_type: MessageTypes::build(A2AMessageKinds::CreateAgent),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAgentResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "withPairwiseDID")]
    from_did: String,
    #[serde(rename = "withPairwiseDIDVerKey")]
    from_vk: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComMethodUpdated {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateComMethod {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "comMethod")]
    com_method: ComMethod,
}

#[derive(Debug, PartialEq)]
pub enum ComMethodType {
    A2A,
    Webhook,
}

impl Serialize for ComMethodType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let value = match self {
            ComMethodType::A2A => "1",
            ComMethodType::Webhook => "2",
        };
        Value::String(value.to_string()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ComMethodType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;
        match value.as_str() {
            Some("1") => Ok(ComMethodType::A2A),
            Some("2") => Ok(ComMethodType::Webhook),
            _ => Err(de::Error::custom("Unexpected communication method type."))
        }
    }
}

impl UpdateComMethod {
    fn build(com_method: ComMethod) -> UpdateComMethod {
        UpdateComMethod {
            msg_type: MessageTypes::build(A2AMessageKinds::UpdateComMethod),
            com_method,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ComMethod {
    id: String,
    #[serde(rename = "type")]
    e_type: ComMethodType,
    value: String,
}

pub async fn connect(my_did: &str, my_vk: &str, agency_did: &str) -> AgencyClientResult<(String, String)> {
    trace!("connect >>> my_did: {}, my_vk: {}, agency_did: {}", my_did, my_vk, agency_did);
    /* STEP 1 - CONNECT */
    let message = A2AMessage::Version2(
        A2AMessageV2::Connect(Connect::build(my_did, my_vk))
    );

    let mut response = send_message_to_agency(&message, agency_did).await?;

    let ConnectResponse { from_vk: agency_pw_vk, from_did: agency_pw_did, .. } =
        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::ConnectResponse(resp)) =>
                resp,
            _ => return
                Err(AgencyClientError::from_msg(
                    AgencyClientErrorKind::InvalidHttpResponse,
                    "Message does not match any variant of ConnectResponse")
                )
        };

    agency_settings::set_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_VERKEY, &agency_pw_vk);
    agency_settings::set_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_DID, &agency_pw_did);

    trace!("connect <<< agency_pw_did: {}, agency_pw_vk: {}", agency_pw_did, agency_pw_vk);
    Ok((agency_pw_did, agency_pw_vk))
}

pub async fn onboarding(my_did: &str, my_vk: &str, agency_did: &str) -> AgencyClientResult<(String, String)> {
    info!("onboarding >>> my_did: {}, my_vk: {}, agency_did: {}", my_did, my_vk, agency_did);
    AgencyMockDecrypted::set_next_decrypted_response(constants::CONNECTED_RESPONSE_DECRYPTED);
    let (agency_pw_did, _) = connect(my_did, my_vk, agency_did).await?;

    /* STEP 2 - REGISTER */
    let message = A2AMessage::Version2(
        A2AMessageV2::SignUp(SignUp::build())
    );

    AgencyMockDecrypted::set_next_decrypted_response(constants::REGISTER_RESPONSE_DECRYPTED);
    let mut response = send_message_to_agency(&message, &agency_pw_did).await?;

    let _response: SignUpResponse =
        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::SignUpResponse(resp)) => resp,
            _ => return Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of SignUpResponse"))
        };

    /* STEP 3 - CREATE AGENT */
    let message = A2AMessage::Version2(
        A2AMessageV2::CreateAgent(CreateAgent::build())
    );
    AgencyMockDecrypted::set_next_decrypted_response(constants::AGENT_CREATED_DECRYPTED);
    let mut response = send_message_to_agency(&message, &agency_pw_did).await?;

    let response: CreateAgentResponse =
        match response.remove(0) {
            A2AMessage::Version2(A2AMessageV2::CreateAgentResponse(resp)) => resp,
            _ => return Err(AgencyClientError::from_msg(AgencyClientErrorKind::InvalidHttpResponse, "Message does not match any variant of CreateAgentResponse"))
        };

    trace!("onboarding <<< from_did: {}, from_vk: {}", response.from_did, response.from_vk);
    Ok((response.from_did, response.from_vk))
}

pub async fn update_agent_webhook(webhook_url: &str) -> AgencyClientResult<()> {
    info!("update_agent_webhook >>> webhook_url: {:?}", webhook_url);

    let com_method: ComMethod = ComMethod {
        id: String::from("123"),
        e_type: ComMethodType::Webhook,
        value: String::from(webhook_url),
    };

    match agency_settings::get_config_value(agency_settings::CONFIG_REMOTE_TO_SDK_DID) {
        Ok(to_did) => {
            update_agent_webhook_v2(&to_did, com_method).await?;
        }
        Err(e) => warn!("Unable to update webhook (did you provide remote did in the config?): {}", e)
    }
    Ok(())
}

async fn update_agent_webhook_v2(to_did: &str, com_method: ComMethod) -> AgencyClientResult<()> {
    info!("> update_agent_webhook_v2");
    if agency_mocks_enabled() {
        warn!("update_agent_webhook_v2 ::: Indy mocks enabled, skipping updating webhook url.");
        return Ok(());
    }

    let message = A2AMessage::Version2(
        A2AMessageV2::UpdateComMethod(UpdateComMethod::build(com_method))
    );
    send_message_to_agency(&message, &to_did).await?;
    Ok(())
}

pub async fn send_message_to_agency(message: &A2AMessage, did: &str) -> AgencyClientResult<Vec<A2AMessage>> {
    trace!("send_message_to_agency >>> message: ..., did: {}", did);
    let data = prepare_message_for_agency(message, &did)?;

    let response = post_to_agency(&data).await
        .map_err(|err| err.map(AgencyClientErrorKind::InvalidHttpResponse, error_utils::INVALID_HTTP_RESPONSE.message))?;

    parse_response_from_agency(&response)
}

#[cfg(test)]
mod tests {
    use std::env;
    use crate::agency_client::AgencyClient;

    use crate::agent_utils::{ComMethodType, update_agent_webhook};

    #[test]
    #[cfg(feature = "general_test")]
    fn test_method_type_serialization() {
        assert_eq!("\"1\"", serde_json::to_string::<ComMethodType>(&ComMethodType::A2A).unwrap());
        assert_eq!("\"2\"", serde_json::to_string::<ComMethodType>(&ComMethodType::Webhook).unwrap());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_method_type_deserialization() {
        assert_eq!(ComMethodType::A2A, serde_json::from_str::<ComMethodType>("\"1\"").unwrap());
        assert_eq!(ComMethodType::Webhook, serde_json::from_str::<ComMethodType>("\"2\"").unwrap());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_onboard_receive_download() {
        // open wallet
        let wallet_config = WalletConfig {
            wallet_name: wallet_name.into(),
            wallet_key: settings::DEFAULT_WALLET_KEY.into(),
            wallet_key_derivation: settings::WALLET_KDF_RAW.into(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };
        create_wallet(&wallet_config);

        // create agency client
        // let (my_did, my_vk) = signus::create_and_store_my_did(provision_agent_config.agent_seed.as_ref().map(String::as_str), None)?;
        let mut agency_client = AgencyClient::default();
        agency_client.set_agency_did(&provision_agent_config.agency_did);
        agency_client.set_agency_vk(&provision_agent_config.agency_verkey);
        agency_client.set_agency_url(&provision_agent_config.agency_endpoint);
        agency_client.set_my_vk(&my_vk);
        agency_client.set_my_pwdid(&my_did);
        agency_client.set_agent_vk(&provision_agent_config.agency_verkey); // This is reset when connection is established and agent did needs not be set before onboarding

        // let (agent_did, agent_vk) = agent_utils::onboarding(&my_did, &my_vk, &provision_agent_config.agency_did)?;
        //
        // Ok(AgencyClientConfig {
        //     agency_did: provision_agent_config.agency_did.clone(),
        //     agency_endpoint: provision_agent_config.agency_endpoint.clone(),
        //     agency_verkey: provision_agent_config.agency_verkey.clone(),
        //     remote_to_sdk_did: agent_did,
        //     remote_to_sdk_verkey: agent_vk,
        //     sdk_to_remote_did: my_did,
        //     sdk_to_remote_verkey: my_vk,
        // })

        // set wallet handle
        // create instance
        // onboard
        // create agent connection
        // receive some message
        // download message
    }

    pub fn open_as_main_wallet(wallet_config: &WalletConfig) -> VcxResult<WalletHandle> {
        trace!("open_as_main_wallet >>> {}", &wallet_config.wallet_name);
        let config = build_wallet_config(&wallet_config.wallet_name, wallet_config.wallet_type.as_deref(), wallet_config.storage_config.as_deref());
        let credentials = build_wallet_credentials(&wallet_config.wallet_key, wallet_config.storage_credentials.as_deref(), &wallet_config.wallet_key_derivation, wallet_config.rekey.as_deref(), wallet_config.rekey_derivation_method.as_deref())?;

        let handle = indy::wallet::open_wallet(&config, &credentials)
            .wait()
            .map_err(|err|
                match err.error_code.clone() {
                    ErrorCode::WalletAlreadyOpenedError => {
                        err.to_indy_facade_err(VcxErrorKind::WalletAlreadyOpen,
                                               format!("Wallet \"{}\" already opened.", wallet_config.wallet_name))
                    }
                    ErrorCode::WalletAccessFailed => {
                        err.to_indy_facade_err(VcxErrorKind::WalletAccessFailed,
                                               format!("Can not open wallet \"{}\". Invalid key has been provided.", wallet_config.wallet_name))
                    }
                    ErrorCode::WalletNotFoundError => {
                        err.to_indy_facade_err(VcxErrorKind::WalletNotFound,
                                               format!("Wallet \"{}\" not found or unavailable", wallet_config.wallet_name))
                    }
                    error_code => {
                        err.to_indy_facade_err(VcxErrorKind::LibndyError(error_code as u32), "Indy error occurred")
                    }
                })?;

        set_wallet_handle(handle);

        Ok(handle)
    }

    pub fn set_wallet_handle(handle: WalletHandle) -> WalletHandle {
        trace!("set_wallet_handle >>> handle: {:?}", handle);
        unsafe { WALLET_HANDLE = handle; }
        settings::get_agency_client_mut().unwrap().set_wallet_handle(handle.0);
        unsafe { WALLET_HANDLE }
    }

    pub fn get_wallet_handle() -> WalletHandle { unsafe { WALLET_HANDLE } }

    pub fn reset_wallet_handle() -> VcxResult<()> {
        set_wallet_handle(INVALID_WALLET_HANDLE);
        settings::get_agency_client_mut()?.reset_wallet_handle();
        Ok(())
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct WalletConfig {
        pub wallet_name: String,
        pub wallet_key: String,
        pub wallet_key_derivation: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub wallet_type: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub storage_config: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub storage_credentials: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rekey: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub rekey_derivation_method: Option<String>,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    struct WalletCredentials {
        key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        rekey: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        storage_credentials: Option<serde_json::Value>,
        key_derivation_method: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        rekey_derivation_method: Option<String>,
    }

    pub fn create_wallet(config: &WalletConfig) -> VcxResult<()> {
        let wh = create_and_open_as_main_wallet(&config)?;
        trace!("Created wallet with handle {:?}", wh);

        // If MS is already in wallet then just continue
        anoncreds::libindy_prover_create_master_secret(settings::DEFAULT_LINK_SECRET_ALIAS).ok();

        close_main_wallet()?;
        Ok(())
    }


    // #[test]
    // #[cfg(feature = "to_restore")]
    // #[cfg(feature = "general_test")]
    // fn test_update_agent_info() {
    //     let _setup = SetupMocks::init();
    //     // todo: Need to mock agency v2 response, only agency v1 mocking works
    //     update_agent_info("123", "value").unwrap();
    // }

    #[cfg(feature = "agency_pool_tests")]
    #[test]
    fn test_update_agent_webhook_real() {
        let _setup = SetupLibraryAgencyV2::init();

        ::utils::devsetup::set_consumer(None);
        update_agent_webhook("https://example.org").unwrap();
    }
}
