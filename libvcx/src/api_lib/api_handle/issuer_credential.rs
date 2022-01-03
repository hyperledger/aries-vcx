use serde_json;

use aries_vcx::messages::issuance::credential_offer::CredentialOffer;
use aries_vcx::utils::error;

use crate::api_lib::api_handle::connection;
use crate::api_lib::api_handle::credential_def;
use crate::api_lib::api_handle::object_cache::ObjectCache;
use crate::aries_vcx::handlers::issuance::issuer::issuer::Issuer;
use crate::aries_vcx::messages::a2a::A2AMessage;
use crate::aries_vcx::messages::issuance::credential_offer::OfferInfo;
use crate::error::prelude::*;

lazy_static! {
    static ref ISSUER_CREDENTIAL_MAP: ObjectCache<Issuer> = ObjectCache::<Issuer>::new("issuer-credentials-cache");
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "version", content = "data")]
enum IssuerCredentials {
    #[serde(rename = "2.0")]
    V3(Issuer),
}

pub fn issuer_credential_create(source_id: String) -> VcxResult<u32> {
    ISSUER_CREDENTIAL_MAP.add(Issuer::create(&source_id)?)
}

pub fn update_state(handle: u32, message: Option<&str>, connection_handle: u32) -> VcxResult<u32> {
    ISSUER_CREDENTIAL_MAP.get_mut(handle, |credential| {
        trace!("issuer_credential::update_state >>> ");
        if credential.is_terminal_state() { return Ok(credential.get_state().into()); }
        let send_message = connection::send_message_closure(connection_handle)?;

        if let Some(message) = message {
            let message: A2AMessage = serde_json::from_str(&message)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidOption, format!("Cannot update state: Message deserialization failed: {:?}", err)))?;
            credential.step(message.into(), Some(&send_message))?;
        } else {
            let messages = connection::get_messages(connection_handle)?;
            if let Some((uid, msg)) = credential.find_message_to_handle(messages) {
                credential.step(msg.into(), Some(&send_message))?;
                connection::update_message_status(connection_handle, uid)?;
            }
        }
        Ok(credential.get_state().into())
    })
}

pub fn get_state(handle: u32) -> VcxResult<u32> {
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

pub fn build_credential_offer_msg(handle: u32,
                                  cred_def_handle: u32,
                                  credential_json: String,
                                  comment: Option<String>) -> VcxResult<()> {
    ISSUER_CREDENTIAL_MAP.get_mut(handle, |credential| {
        let offer_info = OfferInfo {
            credential_json: credential_json.clone(),
            cred_def_id: credential_def::get_cred_def_id(cred_def_handle)?,
            rev_reg_id: credential_def::get_rev_reg_id(cred_def_handle).ok(),
            tails_file: credential_def::get_tails_file(cred_def_handle)?,
        };
        Ok(credential.build_credential_offer_msg(offer_info.clone(), comment.clone())?)
    })
}

pub fn mark_credential_offer_msg_sent(handle: u32) -> VcxResult<()> {
    ISSUER_CREDENTIAL_MAP.get_mut(handle, |credential| {
        Ok(credential.mark_credential_offer_msg_sent()?)
    })
}

pub fn get_credential_offer_msg(handle: u32) -> VcxResult<A2AMessage> {
    ISSUER_CREDENTIAL_MAP.get_mut(handle, |credential| {
        Ok(credential.get_credential_offer_msg()?)
    })
}

pub fn send_credential_offer(handle: u32,
                             cred_def_handle: u32,
                             connection_handle: u32,
                             credential_json: String,
                             comment: Option<String>) -> VcxResult<u32> {
    ISSUER_CREDENTIAL_MAP.get_mut(handle, |credential| {
        let offer_info = OfferInfo {
            credential_json: credential_json.clone(),
            cred_def_id: credential_def::get_cred_def_id(cred_def_handle)?,
            rev_reg_id: credential_def::get_rev_reg_id(cred_def_handle).ok(),
            tails_file: credential_def::get_tails_file(cred_def_handle)?,
        };
        credential.build_credential_offer_msg(offer_info, comment.clone())?;
        let send_message = connection::send_message_closure(connection_handle)?;
        credential.send_credential_offer(send_message)?;
        let new_credential = credential.clone();
        *credential = new_credential;
        Ok(error::SUCCESS.code_num)
    })
}

pub fn send_credential_offer_v2(credential_handle: u32,
                                connection_handle: u32,) -> VcxResult<u32> {
    ISSUER_CREDENTIAL_MAP.get_mut(credential_handle, |credential| {
        let send_message = connection::send_message_closure(connection_handle)?;
        credential.send_credential_offer(send_message)?;
        let new_credential = credential.clone();
        *credential = new_credential;
        Ok(error::SUCCESS.code_num)
    })
}

pub fn generate_credential_msg(handle: u32, _my_pw_did: &str) -> VcxResult<String> {
    ISSUER_CREDENTIAL_MAP.get_mut(handle, |_| {
        Err(VcxError::from_msg(VcxErrorKind::ActionNotSupported, "Not implemented yet")) // TODO: implement
    })
}

pub fn send_credential(handle: u32, connection_handle: u32) -> VcxResult<u32> {
    ISSUER_CREDENTIAL_MAP.get_mut(handle, |credential| {
        credential.send_credential(connection::send_message_closure(connection_handle)?)?;
        Ok(error::SUCCESS.code_num)
    })
}

pub fn revoke_credential(handle: u32) -> VcxResult<()> {
    trace!("revoke_credential >>> handle: {}", handle);
    ISSUER_CREDENTIAL_MAP.get_mut(handle, |credential| {
        credential.revoke_credential(true).map_err(|err| err.into())
    })
}

pub fn revoke_credential_local(handle: u32) -> VcxResult<()> {
    ISSUER_CREDENTIAL_MAP.get_mut(handle, |credential| {
        credential.revoke_credential(false).map_err(|err| err.into())
    })
}

pub fn convert_to_map(s: &str) -> VcxResult<serde_json::Map<String, serde_json::Value>> {
    serde_json::from_str(s)
        .map_err(|_| {
            warn!("{}", error::INVALID_ATTRIBUTES_STRUCTURE.message);
            VcxError::from_msg(VcxErrorKind::InvalidAttributesStructure, error::INVALID_ATTRIBUTES_STRUCTURE.message)
        })
}

pub fn get_credential_attributes(handle: u32) -> VcxResult<String> {
    ISSUER_CREDENTIAL_MAP.get(handle, |_| {
        Err(VcxError::from(VcxErrorKind::NotReady)) // TODO: implement
    })
}

pub fn get_rev_reg_id(handle: u32) -> VcxResult<String> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| {
        credential.get_rev_reg_id().map_err(|err| err.into())
    })
}

pub fn is_revokable(handle: u32) -> VcxResult<bool> {
    ISSUER_CREDENTIAL_MAP.get(handle, |credential| {
        credential.is_revokable().map_err(|err| err.into())
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
    use crate::api_lib::api_handle::credential_def::tests::create_cred_def_fake;
    use crate::api_lib::api_handle::issuer_credential;
    use crate::aries_vcx::handlers::issuance::issuer::issuer::IssuerState;

    use super::*;

    pub fn util_put_credential_def_in_issuer_wallet(_schema_seq_num: u32, _wallet_handle: i32) {
        let issuer_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let tag = "test_tag";
        let config = "{support_revocation: false}";

        libindy_create_and_store_credential_def(&issuer_did, SCHEMAS_JSON, tag, None, config).unwrap();
    }

    fn _issuer_credential_create() -> u32 {
        issuer_credential_create("1".to_string()).unwrap()
    }

    fn _cred_json() -> String {
        "{\"attr\":\"value\"}".to_string()
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_issuer_credential_create_succeeds() {
        let _setup = SetupMocks::init();

        let handle = _issuer_credential_create();
        assert!(handle > 0);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_to_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = _issuer_credential_create();
        let string = to_string(handle).unwrap();
        assert!(!string.is_empty());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_send_credential_offer() {
        let _setup = SetupMocks::init();

        let handle_conn = build_test_connection_inviter_requested();

        let handle_cred = _issuer_credential_create();

        assert_eq!(send_credential_offer(handle_cred, create_cred_def_fake(), handle_conn, _cred_json(), None).unwrap(), error::SUCCESS.code_num);
        assert_eq!(get_state(handle_cred).unwrap(), u32::from(IssuerState::OfferSent));
    }

    #[cfg(feature = "pool_tests")]
    #[cfg(feature = "to_restore")]
    #[test]
    fn test_generate_cred_offer() {
        let _setup = SetupWithWalletAndAgency::init();

        let _issuer = create_full_issuer_credential().0
            .generate_credential_offer().unwrap();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_retry_send_credential_offer() {
        let _setup = SetupMocks::init();

        let connection_handle = build_test_connection_inviter_requested();

        let handle = _issuer_credential_create();
        assert_eq!(get_state(handle).unwrap(), u32::from(IssuerState::Initial));

        LibindyMock::set_next_result(error::TIMEOUT_LIBINDY_ERROR.code_num);

        let res = send_credential_offer(handle, create_cred_def_fake(), connection_handle, _cred_json(), None).unwrap_err();
        assert_eq!(res.kind(), VcxErrorKind::InvalidState);
        assert_eq!(get_state(handle).unwrap(), u32::from(IssuerState::Initial));

        // Can retry after initial failure
        assert_eq!(send_credential_offer(handle, create_cred_def_fake(), connection_handle, _cred_json(), None).unwrap(), error::SUCCESS.code_num);
        assert_eq!(get_state(handle).unwrap(), u32::from(IssuerState::OfferSent));
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_from_string_succeeds() {
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

    #[test]
    #[cfg(feature = "general_test")]
    fn test_update_state_with_message() {
        let _setup = SetupMocks::init();

        let handle_conn = build_test_connection_inviter_requested();
        let handle_cred = _issuer_credential_create();

        assert_eq!(send_credential_offer(handle_cred, create_cred_def_fake(), handle_conn, _cred_json(), None).unwrap(), error::SUCCESS.code_num);
        assert_eq!(get_state(handle_cred).unwrap(), u32::from(IssuerState::OfferSent));

        issuer_credential::update_state(handle_cred, Some(ARIES_CREDENTIAL_REQUEST), handle_conn).unwrap();
        assert_eq!(get_state(handle_cred).unwrap(), u32::from(IssuerState::RequestReceived));
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_update_state_with_bad_message() {
        let _setup = SetupMocks::init();

        let handle_conn = build_test_connection_inviter_requested();
        let handle_cred = _issuer_credential_create();

        assert_eq!(send_credential_offer(handle_cred, create_cred_def_fake(), handle_conn, _cred_json(), None).unwrap(), error::SUCCESS.code_num);
        assert_eq!(get_state(handle_cred).unwrap(), u32::from(IssuerState::OfferSent));

        // try to update state with nonsense message
        let result = issuer_credential::update_state(handle_cred, Some(ARIES_CONNECTION_ACK), handle_conn);
        assert!(result.is_ok()); // todo: maybe we should rather return error if update_state doesn't progress state
        assert_eq!(get_state(handle_cred).unwrap(), u32::from(IssuerState::OfferSent));
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_release_all() {
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

    #[test]
    #[cfg(feature = "general_test")]
    fn test_errors() {
        let _setup = SetupLibraryWallet::init();

        assert_eq!(to_string(0).unwrap_err().kind(), VcxErrorKind::InvalidHandle);
        assert_eq!(release(0).unwrap_err().kind(), VcxErrorKind::InvalidIssuerCredentialHandle);
    }

    // todo: Write test which will use use credetial definition supporting revocation, then actually revoke credential
}
