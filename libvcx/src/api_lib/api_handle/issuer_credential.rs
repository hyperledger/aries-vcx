use serde_json;

use aries_vcx::error::{VcxError, VcxErrorKind, VcxResult};
use aries_vcx::handlers::issuance::issuer::Issuer;
use aries_vcx::messages::a2a::A2AMessage;
use aries_vcx::messages::issuance::credential_offer::OfferInfo;
use aries_vcx::utils::error;

use crate::api_lib::api_handle::connection;
use crate::api_lib::api_handle::credential_def;
use crate::api_lib::api_handle::revocation_registry::REV_REG_MAP;
use crate::api_lib::api_handle::object_cache::ObjectCache;

lazy_static! {
    static ref ISSUER_CREDENTIAL_MAP: ObjectCache<Issuer> = ObjectCache::<Issuer>::new("issuer-credentials-cache");
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "version", content = "data")]
enum IssuerCredentials {
    #[serde(rename = "2.0")]
    V3(Issuer),
}

pub  fn issuer_credential_create(source_id: String) -> VcxResult<u32> {
    ISSUER_CREDENTIAL_MAP.add(Issuer::create(&source_id)?)
}

pub async fn update_state(handle: u32, message: Option<&str>, connection_handle: u32) -> VcxResult<u32> {
    trace!("issuer_credential::update_state >>> ");
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    if credential.is_terminal_state() { return Ok(credential.get_state().into()); }
    let send_message = connection::send_message_closure(connection_handle)?;

    if let Some(message) = message {
        let message: A2AMessage = serde_json::from_str(&message)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidOption, format!("Cannot update state: Message deserialization failed: {:?}", err)))?;
        credential.step(message.into(), Some(send_message)).await?;
    } else {
        let messages = connection::get_messages(connection_handle).await?;
        if let Some((uid, msg)) = credential.find_message_to_handle(messages) {
            credential.step(msg.into(), Some(send_message)).await?;
            connection::update_message_status(connection_handle, &uid).await?;
        }
    }
    let res: u32 = credential.get_state().into();
    ISSUER_CREDENTIAL_MAP.insert(handle, credential)?;
    Ok(res)
}

pub  fn get_state(handle: u32) -> VcxResult<u32> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| {
        Ok(credential.get_state().into())
    })
}

pub fn get_credential_status(handle: u32) -> VcxResult<u32> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| {
        credential.get_credential_status().map_err(|err| err.into())
    })
}

pub fn release(handle: u32) -> VcxResult<()> {
    ISSUER_CREDENTIAL_MAP.release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidIssuerCredentialHandle)))
}

pub fn release_all() {
    ISSUER_CREDENTIAL_MAP.drain().ok();
}

pub fn is_valid_handle(handle: u32) -> bool {
    ISSUER_CREDENTIAL_MAP.has_handle(handle)
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| {
        serde_json::to_string(&IssuerCredentials::V3(credential.clone()))
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidState, format!("cannot serialize IssuerCredential credentialect: {:?}", err)))
    })
}

pub fn from_string(credential_data: &str) -> VcxResult<u32> {
    let issuer_credential: IssuerCredentials = serde_json::from_str(credential_data)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize IssuerCredential: {:?}", err)))?;

    match issuer_credential {
        IssuerCredentials::V3(credential) => ISSUER_CREDENTIAL_MAP.add(credential)
    }
}

pub async fn build_credential_offer_msg(handle: u32,
                                        cred_def_handle: u32,
                                        credential_json: &str,
                                        comment: Option<&str>) -> VcxResult<()> {
    if credential_def::has_pending_revocations_primitives_to_be_published(cred_def_handle)? {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot issue credential of specified credential definition because its revocation primitives were not published on the ledger yet.")));
    };
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    let offer_info = OfferInfo {
        credential_json: credential_json.to_string(),
        cred_def_id: credential_def::get_cred_def_id(cred_def_handle)?,
        rev_reg_id: credential_def::get_rev_reg_id(cred_def_handle).ok(),
        tails_file: credential_def::get_tails_file(cred_def_handle)?,
    };
    credential.build_credential_offer_msg(offer_info.clone(), comment.map(|s| s.to_string())).await?;
    ISSUER_CREDENTIAL_MAP.insert(handle, credential)
}

pub async fn build_credential_offer_msg_v2(handle: u32,
                                           cred_def_handle: u32,
                                           rev_reg_handle: u32,
                                           credential_json: &str,
                                           comment: Option<&str>) -> VcxResult<()> {
    if credential_def::has_pending_revocations_primitives_to_be_published(cred_def_handle)? {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot issue credential of specified credential definition because its revocation primitives were not published on the ledger yet.")));
    };
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    let cred_def = credential_def::CREDENTIALDEF_MAP.get_cloned(cred_def_handle)?;
    let offer_info = if cred_def.get_support_revocation() {
        let rev_reg = REV_REG_MAP.get_cloned(rev_reg_handle)?;
        OfferInfo {
            credential_json: credential_json.to_string(),
            cred_def_id: credential_def::get_cred_def_id(cred_def_handle)?,
            rev_reg_id: Some(rev_reg.get_rev_reg_id()),
            tails_file: Some(rev_reg.get_tails_file())
        }
    } else {
        OfferInfo {
            credential_json: credential_json.to_string(),
            cred_def_id: credential_def::get_cred_def_id(cred_def_handle)?,
            rev_reg_id: None,
            tails_file: None
        }

    };
    credential.build_credential_offer_msg(offer_info.clone(), comment.map(|s| s.to_string())).await?;
    ISSUER_CREDENTIAL_MAP.insert(handle, credential)
}

pub fn mark_credential_offer_msg_sent(handle: u32) -> VcxResult<()> {
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    credential.mark_credential_offer_msg_sent()?;
    ISSUER_CREDENTIAL_MAP.insert(handle, credential)
}

pub fn get_credential_offer_msg(handle: u32) -> VcxResult<A2AMessage> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| {
        Ok(credential.get_credential_offer_msg()?)
    })
}

pub async fn send_credential_offer(handle: u32,
                                   cred_def_handle: u32,
                                   connection_handle: u32,
                                   credential_json: &str,
                                   comment: Option<&str>) -> VcxResult<u32> {
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    let offer_info = OfferInfo {
        credential_json: credential_json.to_string(),
        cred_def_id: credential_def::get_cred_def_id(cred_def_handle)?,
        rev_reg_id: credential_def::get_rev_reg_id(cred_def_handle).ok(),
        tails_file: credential_def::get_tails_file(cred_def_handle)?,
    };
    credential.build_credential_offer_msg(offer_info, comment.map(|s| s.to_string())).await?;
    let send_message = connection::send_message_closure(connection_handle)?;
    credential.send_credential_offer(send_message).await?;
    ISSUER_CREDENTIAL_MAP.insert(handle, credential)?;
    Ok(error::SUCCESS.code_num)
}

pub async fn send_credential_offer_v2(handle: u32, connection_handle: u32) -> VcxResult<u32> {
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    let send_message = connection::send_message_closure(connection_handle)?;
    credential.send_credential_offer(send_message).await?;
    ISSUER_CREDENTIAL_MAP.insert(handle, credential)?;
    Ok(error::SUCCESS.code_num)
}

pub fn generate_credential_msg(_handle: u32, _my_pw_did: &str) -> VcxResult<String> {
    Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Not implemented yet")) // TODO: implement
}

pub async fn send_credential(handle: u32, connection_handle: u32) -> VcxResult<u32> {
    let mut credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    credential.send_credential(connection::send_message_closure(connection_handle)?).await?;
    ISSUER_CREDENTIAL_MAP.insert(handle, credential)?;
    Ok(error::SUCCESS.code_num)
}

pub async fn revoke_credential(handle: u32) -> VcxResult<()> {
    trace!("revoke_credential >>> handle: {}", handle);
    let credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    credential.revoke_credential(true).await.map_err(|err| err.into())
}

pub async fn revoke_credential_local(handle: u32) -> VcxResult<()> {
    let credential = ISSUER_CREDENTIAL_MAP.get_cloned(handle)?;
    credential.revoke_credential(false).await.map_err(|err| err.into())
}

pub fn convert_to_map(s: &str) -> VcxResult<serde_json::Map<String, serde_json::Value>> {
    serde_json::from_str(s)
        .map_err(|_| {
            warn!("{}", error::INVALID_ATTRIBUTES_STRUCTURE.message);
            VcxError::from_msg(VcxErrorKind::InvalidAttributesStructure, error::INVALID_ATTRIBUTES_STRUCTURE.message)
        })
}

pub fn get_credential_attributes(_handle: u32) -> VcxResult<String> {
    Err(VcxError::from(VcxErrorKind::NotReady)) // TODO: implement
}

pub fn get_rev_reg_id(handle: u32) -> VcxResult<String> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| {
        credential.get_rev_reg_id().map_err(|err| err.into())
    })
}

pub fn is_revokable(handle: u32) -> VcxResult<bool> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| {
        Ok(credential.is_revokable())
    })
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| {
        credential.get_source_id().map_err(|err| err.into())
    })
}

pub fn get_thread_id(handle: u32) -> VcxResult<String> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| {
        credential.get_thread_id().map_err(|err| err.into())
    })
}

#[cfg(test)]
#[allow(unused_imports)]
pub mod tests {
    use aries_vcx::agency_client::mocking::HttpClientMockResponse;
    use aries_vcx::libindy::utils::anoncreds::libindy_create_and_store_credential_def;
    use aries_vcx::libindy::utils::LibindyMock;
    use aries_vcx::settings;
    use aries_vcx::utils::constants::{REV_REG_ID, SCHEMAS_JSON, V3_OBJECT_SERIALIZE_VERSION};
    use aries_vcx::utils::devsetup::{SetupLibraryWallet, SetupMocks, SetupWithWalletAndAgency};
    use aries_vcx::utils::mockdata::mockdata_connection::ARIES_CONNECTION_ACK;
    use aries_vcx::utils::mockdata::mockdata_credex::ARIES_CREDENTIAL_REQUEST;

    use crate::api_lib::api_handle::connection::tests::build_test_connection_inviter_requested;
    use crate::api_lib::api_handle::credential_def::tests::{create_cred_def_fake, create_cred_def_fake_unpublished};
    use crate::api_lib::api_handle::issuer_credential;
    use crate::aries_vcx::protocols::issuance::issuer::state_machine::IssuerState;

    use super::*;

    pub async fn util_put_credential_def_in_issuer_wallet(_schema_seq_num: u32, _wallet_handle: i32) {
        let issuer_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let tag = "test_tag";
        let config = "{support_revocation: false}";

        libindy_create_and_store_credential_def(&issuer_did, SCHEMAS_JSON, tag, None, config).await.unwrap();
    }

    fn _issuer_credential_create() -> u32 {
        issuer_credential_create("1".to_string()).unwrap()
    }

    fn _cred_json() -> &'static str {
        "{\"attr\":\"value\"}"
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_issuer_credential_create_succeeds() {
        let _setup = SetupMocks::init();

        let handle = _issuer_credential_create();
        assert!(handle > 0);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_to_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = _issuer_credential_create();
        let string = to_string(handle).unwrap();
        assert!(!string.is_empty());
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_send_credential_offer() {
        let _setup = SetupMocks::init();

        let handle_conn = build_test_connection_inviter_requested().await;

        let handle_cred = _issuer_credential_create();

        assert_eq!(send_credential_offer(handle_cred, create_cred_def_fake().await, handle_conn, _cred_json(), None).await.unwrap(), error::SUCCESS.code_num);
        assert_eq!(get_state(handle_cred).unwrap(), u32::from(IssuerState::OfferSent));
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_fail_creating_cred_offer_if_revocations_were_not_published() {
        let _setup = SetupMocks::init();

        let handle_conn = build_test_connection_inviter_requested().await;

        let handle_cred = _issuer_credential_create();

        assert_eq!(send_credential_offer(handle_cred, create_cred_def_fake_unpublished().await, handle_conn, _cred_json(), None).await.unwrap(), error::SUCCESS.code_num);
    }

    #[cfg(feature = "pool_tests")]
    #[cfg(feature = "to_restore")]
    #[tokio::test]
    async fn test_generate_cred_offer() {
        let _setup = SetupWithWalletAndAgency::init().await;

        let _issuer = create_full_issuer_credential().0
            .generate_credential_offer().unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_retry_send_credential_offer() {
        let _setup = SetupMocks::init();

        let connection_handle = build_test_connection_inviter_requested().await;

        let handle = _issuer_credential_create();
        assert_eq!(get_state(handle).unwrap(), u32::from(IssuerState::Initial));

        LibindyMock::set_next_result(error::TIMEOUT_LIBINDY_ERROR.code_num);

        let res = send_credential_offer(handle, create_cred_def_fake().await, connection_handle, _cred_json(), None).await.unwrap_err();
        assert_eq!(res.kind(), VcxErrorKind::InvalidState);
        assert_eq!(get_state(handle).unwrap(), u32::from(IssuerState::Initial));

        // Can retry after initial failure
        assert_eq!(send_credential_offer(handle, create_cred_def_fake().await, connection_handle, _cred_json(), None).await.unwrap(), error::SUCCESS.code_num);
        assert_eq!(get_state(handle).unwrap(), u32::from(IssuerState::OfferSent));
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_from_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = _issuer_credential_create();

        let string = to_string(handle).unwrap();

        let value: serde_json::Value = serde_json::from_str(&string).unwrap();
        assert_eq!(value["version"], V3_OBJECT_SERIALIZE_VERSION);

        release(handle).unwrap();

        let new_handle = from_string(&string).unwrap();

        let new_string = to_string(new_handle).unwrap();
        assert_eq!(new_string, string);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_update_state_with_message() {
        let _setup = SetupMocks::init();

        let handle_conn = build_test_connection_inviter_requested().await;
        let handle_cred = _issuer_credential_create();

        assert_eq!(send_credential_offer(handle_cred, create_cred_def_fake().await, handle_conn, _cred_json(), None).await.unwrap(), error::SUCCESS.code_num);
        assert_eq!(get_state(handle_cred).unwrap(), u32::from(IssuerState::OfferSent));

        issuer_credential::update_state(handle_cred, Some(ARIES_CREDENTIAL_REQUEST), handle_conn).await.unwrap();
        assert_eq!(get_state(handle_cred).unwrap(), u32::from(IssuerState::RequestReceived));
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_update_state_with_bad_message() {
        let _setup = SetupMocks::init();

        let handle_conn = build_test_connection_inviter_requested().await;
        let handle_cred = _issuer_credential_create();

        assert_eq!(send_credential_offer(handle_cred, create_cred_def_fake().await, handle_conn, _cred_json(), None).await.unwrap(), error::SUCCESS.code_num);
        assert_eq!(get_state(handle_cred).unwrap(), u32::from(IssuerState::OfferSent));

        // try to update state with nonsense message
        let result = issuer_credential::update_state(handle_cred, Some(ARIES_CONNECTION_ACK), handle_conn).await;
        assert!(result.is_ok()); // todo: maybe we should rather return error if update_state doesn't progress state
        assert_eq!(get_state(handle_cred).unwrap(), u32::from(IssuerState::OfferSent));
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_release_all() {
        let _setup = SetupMocks::init();

        let h1 = _issuer_credential_create();
        let h2 = _issuer_credential_create();
        let h3 = _issuer_credential_create();
        let h4 = _issuer_credential_create();
        let h5 = _issuer_credential_create();
        release_all();
        assert_eq!(release(h1).unwrap_err().kind(), VcxErrorKind::InvalidIssuerCredentialHandle);
        assert_eq!(release(h2).unwrap_err().kind(), VcxErrorKind::InvalidIssuerCredentialHandle);
        assert_eq!(release(h3).unwrap_err().kind(), VcxErrorKind::InvalidIssuerCredentialHandle);
        assert_eq!(release(h4).unwrap_err().kind(), VcxErrorKind::InvalidIssuerCredentialHandle);
        assert_eq!(release(h5).unwrap_err().kind(), VcxErrorKind::InvalidIssuerCredentialHandle);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_errors() {
        let _setup = SetupLibraryWallet::init();

        assert_eq!(to_string(0).unwrap_err().kind(), VcxErrorKind::InvalidHandle);
        assert_eq!(release(0).unwrap_err().kind(), VcxErrorKind::InvalidIssuerCredentialHandle);
    }

    // todo: Write test which will use use credetial definition supporting revocation, then actually revoke credential
}
