use serde_json;

use aries_vcx::handlers::issuance::credential_def::CredentialDef;
use aries_vcx::handlers::issuance::credential_def::PublicEntityStateType;
use aries_vcx::libindy::utils::anoncreds;
use aries_vcx::libindy::utils::cache::update_rev_reg_ids_cache;

use crate::api_lib::api_handle::object_cache::ObjectCache;
use crate::error::prelude::*;

lazy_static! {
    static ref CREDENTIALDEF_MAP: ObjectCache<CredentialDef> = ObjectCache::<CredentialDef>::new("credential-defs-cache");
}

pub fn create_and_publish_credentialdef(source_id: String,
                                        name: String,
                                        issuer_did: String,
                                        schema_id: String,
                                        tag: String,
                                        revocation_details: String) -> VcxResult<u32> {
    let cred_def = CredentialDef::create(source_id, name, issuer_did, schema_id, tag, revocation_details)?;
    let handle = CREDENTIALDEF_MAP.add(cred_def)?;
    Ok(handle)
}

pub fn publish_revocations(handle: u32) -> VcxResult<()> {
    CREDENTIALDEF_MAP.get(handle, |cd| {
        if let Some(rev_reg_id) = cd.get_rev_reg_id() {
            anoncreds::publish_local_revocations(rev_reg_id.as_str())?;
            Ok(())
        } else {
            Err(VcxError::from(VcxErrorKind::InvalidCredDefHandle))
        }
    })
}

pub fn is_valid_handle(handle: u32) -> bool {
    CREDENTIALDEF_MAP.has_handle(handle)
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |cd| {
        cd.to_string().map_err(|err| err.into())
    })
}

pub fn from_string(data: &str) -> VcxResult<u32> {
    let cred_def: CredentialDef = CredentialDef::from_str(data)?;
    CREDENTIALDEF_MAP.add(cred_def)
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        Ok(c.get_source_id().clone())
    })
}

pub fn get_cred_def_id(handle: u32) -> VcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        Ok(c.get_cred_def_id())
    })
}

pub fn get_rev_reg_id(handle: u32) -> VcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        c.get_rev_reg_id().ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "No revocation registry found - does this credential definiton support revocation?"))
    })
}

pub fn get_tails_file(handle: u32) -> VcxResult<Option<String>> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        Ok(c.get_tails_file())
    })
}

pub fn get_rev_reg_def(handle: u32) -> VcxResult<Option<String>> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        Ok(c.get_rev_reg_def())
    })
}

pub fn release(handle: u32) -> VcxResult<()> {
    CREDENTIALDEF_MAP.release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidCredDefHandle)))
}

pub fn release_all() {
    CREDENTIALDEF_MAP.drain().ok();
}

pub fn update_state(handle: u32) -> VcxResult<u32> {
    CREDENTIALDEF_MAP.get_mut(handle, |s| {
        s.update_state().map_err(|err| err.into())
    })
}

pub fn get_state(handle: u32) -> VcxResult<u32> {
    CREDENTIALDEF_MAP.get_mut(handle, |s| {
        Ok(s.get_state())
    })
}

pub fn check_is_published(handle: u32) -> VcxResult<bool> {
    CREDENTIALDEF_MAP.get_mut(handle, |s| {
        Ok(PublicEntityStateType::Published == s.state)
    })
}

pub fn rotate_rev_reg_def(handle: u32, revocation_details: &str) -> VcxResult<String> {
    CREDENTIALDEF_MAP.get_mut(handle, |s| {
        match &s.issuer_did {
            Some(_) => {
                let new_rev_reg = s.rotate_rev_reg(revocation_details)?;
                match update_rev_reg_ids_cache(&s.id, &new_rev_reg.rev_reg_id) {
                    Ok(()) => s.to_string().map_err(|err| err.into()),
                    Err(err) => Err(err.into())
                }
            }
            // TODO: Better error
            None => Err(VcxError::from(VcxErrorKind::InvalidCredentialHandle))
        }
    })
}

pub fn get_tails_hash(handle: u32) -> VcxResult<String> {
    CREDENTIALDEF_MAP.get_mut(handle, |s| {
        match &s.get_rev_reg_def() {
            Some(rev_reg_def) => {
                let rev_reg_def: aries_vcx::handlers::issuance::credential_def::RevocationRegistryDefinition = serde_json::from_str(&rev_reg_def)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to deserialize current rev_reg_def: {:?}, error: {:?}", rev_reg_def, err)))?;
                Ok(rev_reg_def.value.tails_hash)
            }
            None => Err(VcxError::from(VcxErrorKind::InvalidCredentialHandle))
        }
    })
}

#[cfg(test)]
#[allow(unused_imports)]
pub mod tests {
    use std::{
        thread::sleep,
        time::Duration,
    };

    use aries_vcx::libindy::utils::anoncreds::get_cred_def_json;
    use aries_vcx::libindy::utils::anoncreds::test_utils::create_and_write_test_schema;
    use aries_vcx::settings;
    use aries_vcx::utils;
    use aries_vcx::utils::{
        constants::SCHEMA_ID,
        get_temp_dir_path,
    };
    use aries_vcx::utils::devsetup::{SetupWithWalletAndAgency, SetupMocks};

    use crate::api_lib::api_handle::schema;

    use super::*;

    static CREDENTIAL_DEF_NAME: &str = "Test Credential Definition";
    static ISSUER_DID: &str = "4fUDR9R7fjwELRvH9JT6HH";

    pub fn revocation_details(revoc: bool) -> serde_json::Value {
        let mut revocation_details = json!({"support_revocation":revoc});
        if revoc {
            revocation_details["tails_file"] = json!(get_temp_dir_path("tails_file.txt").to_str().unwrap());
            revocation_details["max_creds"] = json!(10);
        }
        revocation_details
    }

    pub fn prepare_create_cred_def_data(revoc: bool) -> (u32, String, String, serde_json::Value) {
        let schema_handle = schema::tests::create_schema_real();
        sleep(Duration::from_secs(2));
        let schema_id = schema::get_schema_id(schema_handle).unwrap();
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let revocation_details = revocation_details(revoc);
        (schema_handle, schema_id, did, revocation_details)
    }

    pub fn create_cred_def_real(revoc: bool) -> (u32, u32) {
        let (schema_handle, schema_id, did, revocation_details) = prepare_create_cred_def_data(revoc);
        sleep(Duration::from_secs(2));
        let cred_def_handle = create_and_publish_credentialdef("1".to_string(),
                                                               CREDENTIAL_DEF_NAME.to_string(),
                                                               did,
                                                               schema_id,
                                                               "tag_1".to_string(),
                                                               revocation_details.to_string()).unwrap();

        (schema_handle, cred_def_handle)
    }

    pub fn create_cred_def_fake() -> u32 {
        let rev_details = json!({"support_revocation": true, "tails_file": utils::constants::TEST_TAILS_FILE, "max_creds": 2, "tails_url": utils::constants::TEST_TAILS_URL}).to_string();

        create_and_publish_credentialdef("SourceId".to_string(),
                                         CREDENTIAL_DEF_NAME.to_string(),
                                         ISSUER_DID.to_string(),
                                         SCHEMA_ID.to_string(),
                                         "tag".to_string(),
                                         rev_details).unwrap()
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_cred_def_without_rev_will_have_no_rev_id() {
        let _setup = SetupWithWalletAndAgency::init();

        let (_, handle) = create_cred_def_real(false);
        let rev_reg_id = get_rev_reg_id(handle).ok();
        assert!(rev_reg_id.is_none());

        let (_, handle) = create_cred_def_real(true);
        let rev_reg_id = get_rev_reg_id(handle).ok();
        assert!(rev_reg_id.is_some());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_cred_def() {
        let _setup = SetupMocks::init();

        let (_, _) = create_cred_def_real(false);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_revocable_fails_with_no_tails_file() {
        let _setup = SetupWithWalletAndAgency::init();

        let (schema_id, _) = create_and_write_test_schema(utils::constants::DEFAULT_SCHEMA_ATTRS);
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        let rc = create_and_publish_credentialdef("1".to_string(),
                                                  "test_create_revocable_fails_with_no_tails_file".to_string(),
                                                  did,
                                                  schema_id,
                                                  "tag_1".to_string(),
                                                  r#"{"support_revocation":true}"#.to_string());
        assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::InvalidRevocationDetails);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_tails_url_written_to_ledger() {
        let _setup = SetupWithWalletAndAgency::init();

        let (schema_id, _) = create_and_write_test_schema(utils::constants::DEFAULT_SCHEMA_ATTRS);
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        let revocation_details = json!({"support_revocation": true, "tails_file": get_temp_dir_path("tails.txt").to_str().unwrap(), "max_creds": 2, "tails_url": utils::constants::TEST_TAILS_URL.to_string()}).to_string();
        let handle = create_and_publish_credentialdef("1".to_string(),
                                                      "test_tails_url_written_to_ledger".to_string(),
                                                      did,
                                                      schema_id,
                                                      "tag1".to_string(),
                                                      revocation_details).unwrap();
        let rev_reg_def = get_rev_reg_def(handle).unwrap().unwrap();
        let rev_reg_def: serde_json::Value = serde_json::from_str(&rev_reg_def).unwrap();
        let _rev_reg_id = get_rev_reg_id(handle).unwrap();
        assert_eq!(rev_reg_def["value"]["tailsLocation"], utils::constants::TEST_TAILS_URL.to_string());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_tails_base_url_written_to_ledger() {
        let _setup = SetupWithWalletAndAgency::init();
        let tails_url = utils::constants::TEST_TAILS_URL.to_string();

        let (schema_id, _) = create_and_write_test_schema(utils::constants::DEFAULT_SCHEMA_ATTRS);
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        let revocation_details = json!({"support_revocation": true, "tails_file": get_temp_dir_path("tails.txt").to_str().unwrap(), "max_creds": 2, "tails_base_url": tails_url}).to_string();
        let handle = create_and_publish_credentialdef("1".to_string(),
                                                      "test_tails_url_written_to_ledger".to_string(),
                                                      did,
                                                      schema_id,
                                                      "tag1".to_string(),
                                                      revocation_details).unwrap();
        let rev_reg_def = get_rev_reg_def(handle).unwrap().unwrap();
        let rev_reg_def: serde_json::Value = serde_json::from_str(&rev_reg_def).unwrap();
        let tails_hash = get_tails_hash(handle).unwrap();
        assert_eq!(rev_reg_def["value"]["tailsLocation"], vec![tails_url, tails_hash].join("/"));
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_revocable_cred_def() {
        let _setup = SetupWithWalletAndAgency::init();

        let (schema_id, _) = create_and_write_test_schema(utils::constants::DEFAULT_SCHEMA_ATTRS);
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        let revocation_details = json!({"support_revocation": true, "tails_file": get_temp_dir_path("tails.txt").to_str().unwrap(), "max_creds": 2}).to_string();
        let handle = create_and_publish_credentialdef("1".to_string(),
                                                      "test_create_revocable_cred_def".to_string(),
                                                      did,
                                                      schema_id,
                                                      "tag_1".to_string(),
                                                      revocation_details).unwrap();

        assert!(get_rev_reg_def(handle).unwrap().is_some());
        assert!(get_rev_reg_id(handle).ok().is_some());
        let cred_id = get_cred_def_id(handle).unwrap();
        get_cred_def_json(&cred_id).unwrap();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_credential_def_real() {
        let _setup = SetupWithWalletAndAgency::init();

        let (_, handle) = create_cred_def_real(false);

        let _source_id = get_source_id(handle).unwrap();
        let _cred_def_id = get_cred_def_id(handle).unwrap();
        let _schema_json = to_string(handle).unwrap();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_credential_works_twice() {
        let _setup = SetupWithWalletAndAgency::init();

        let (_, schema_id, did, revocation_details) = prepare_create_cred_def_data(false);
        create_and_publish_credentialdef("1".to_string(),
                                         "name".to_string(),
                                         did.clone(),
                                         schema_id.clone(),
                                         "tag_1".to_string(),
                                         revocation_details.to_string()).unwrap();

        sleep(Duration::from_secs(1));
        let err = create_and_publish_credentialdef("1".to_string(),
                                                   "name".to_string(),
                                                   did.clone(),
                                                   schema_id.clone(),
                                                   "tag_1".to_string(),
                                                   revocation_details.to_string()).unwrap_err();

        assert_eq!(err.kind(), VcxErrorKind::CreateCredDef);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_to_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = create_cred_def_fake();

        let credential_string = to_string(handle).unwrap();
        let credential_values: serde_json::Value = serde_json::from_str(&credential_string).unwrap();
        assert_eq!(credential_values["version"].clone(), "1.0");
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_from_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = create_cred_def_fake();
        let credentialdef_data = to_string(handle).unwrap();
        assert!(!credentialdef_data.is_empty());
        release(handle).unwrap();

        let new_handle = from_string(&credentialdef_data).unwrap();
        let new_credentialdef_data = to_string(new_handle).unwrap();

        let credentialdef1: CredentialDef = CredentialDef::from_str(&credentialdef_data).unwrap();
        let credentialdef2: CredentialDef = CredentialDef::from_str(&new_credentialdef_data).unwrap();

        assert_eq!(credentialdef1, credentialdef2);
        assert_eq!(CredentialDef::from_str("{}").unwrap_err().kind(), aries_vcx::error::VcxErrorKind::CreateCredDef);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_release_all() {
        let _setup = SetupMocks::init();

        let h1 = create_and_publish_credentialdef("SourceId".to_string(), CREDENTIAL_DEF_NAME.to_string(), ISSUER_DID.to_string(), SCHEMA_ID.to_string(), "tag".to_string(), "{}".to_string()).unwrap();
        let h2 = create_and_publish_credentialdef("SourceId".to_string(), CREDENTIAL_DEF_NAME.to_string(), ISSUER_DID.to_string(), SCHEMA_ID.to_string(), "tag".to_string(), "{}".to_string()).unwrap();
        let h3 = create_and_publish_credentialdef("SourceId".to_string(), CREDENTIAL_DEF_NAME.to_string(), ISSUER_DID.to_string(), SCHEMA_ID.to_string(), "tag".to_string(), "{}".to_string()).unwrap();
        let h4 = create_and_publish_credentialdef("SourceId".to_string(), CREDENTIAL_DEF_NAME.to_string(), ISSUER_DID.to_string(), SCHEMA_ID.to_string(), "tag".to_string(), "{}".to_string()).unwrap();
        let h5 = create_and_publish_credentialdef("SourceId".to_string(), CREDENTIAL_DEF_NAME.to_string(), ISSUER_DID.to_string(), SCHEMA_ID.to_string(), "tag".to_string(), "{}".to_string()).unwrap();
        release_all();
        assert_eq!(release(h1).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(release(h2).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(release(h3).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(release(h4).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(release(h5).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
    }
}
