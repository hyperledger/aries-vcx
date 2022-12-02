use aries_vcx::error::{VcxError, VcxErrorKind, VcxResult};
use aries_vcx::common::primitives::credential_definition::PublicEntityStateType;
use aries_vcx::common::primitives::credential_definition::CredentialDefConfigBuilder;
use aries_vcx::common::primitives::credential_definition::CredentialDef;

use crate::api_lib::api_handle::object_cache::ObjectCache;
use crate::api_lib::global::profile::get_main_profile;

lazy_static! {
    pub static ref CREDENTIALDEF_MAP: ObjectCache<CredentialDef> =
        ObjectCache::<CredentialDef>::new("credential-defs-cache");
}

pub async fn create(
    source_id: String,
    schema_id: String,
    issuer_did: String,
    tag: String,
    support_revocation: bool,
) -> VcxResult<u32> {
    let config = CredentialDefConfigBuilder::default()
        .issuer_did(issuer_did)
        .schema_id(schema_id)
        .tag(tag)
        .build()
        .map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::InvalidConfiguration,
                format!("Failed build credential config using provided parameters: {:?}", err),
            )
        })?;
    let profile = get_main_profile()?;
    let cred_def = CredentialDef::create(&profile, source_id, config, support_revocation).await?;
    let handle = CREDENTIALDEF_MAP.add(cred_def)?;
    Ok(handle)
}

pub async fn publish(handle: u32) -> VcxResult<()> {
    let mut cd = CREDENTIALDEF_MAP.get_cloned(handle)?;
    if !cd.was_published() {
        let profile = get_main_profile()?;
        cd = cd.publish_cred_def(&profile).await?;
    } else {
        info!("publish >>> Credential definition was already published")
    }
    CREDENTIALDEF_MAP.insert(handle, cd)
}

pub fn is_valid_handle(handle: u32) -> bool {
    CREDENTIALDEF_MAP.has_handle(handle)
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |cd| cd.to_string().map_err(|err| err.into()))
}

pub fn from_string(data: &str) -> VcxResult<u32> {
    let cred_def: CredentialDef = CredentialDef::from_string(data)?;
    CREDENTIALDEF_MAP.add(cred_def)
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |c| Ok(c.get_source_id().clone()))
}

pub fn get_cred_def_id(handle: u32) -> VcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |c| Ok(c.get_cred_def_id()))
}

pub fn release(handle: u32) -> VcxResult<()> {
    CREDENTIALDEF_MAP
        .release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidCredDefHandle)))
}

pub fn release_all() {
    CREDENTIALDEF_MAP.drain().ok();
}

pub async fn update_state(handle: u32) -> VcxResult<u32> {
    let mut cd = CREDENTIALDEF_MAP.get_cloned(handle)?;
    let profile = get_main_profile()?;
    let res = cd.update_state(&profile).await?;
    CREDENTIALDEF_MAP.insert(handle, cd)?;
    Ok(res)
}

pub fn get_state(handle: u32) -> VcxResult<u32> {
    CREDENTIALDEF_MAP.get(handle, |s| Ok(s.get_state()))
}

pub fn check_is_published(handle: u32) -> VcxResult<bool> {
    CREDENTIALDEF_MAP.get(handle, |s| Ok(PublicEntityStateType::Published == s.state))
}

#[cfg(test)]
pub mod tests {
    use std::{thread::sleep, time::Duration};

    use aries_vcx::global::settings;
    #[cfg(feature = "pool_tests")]
    use aries_vcx::common::primitives::credential_definition::RevocationDetailsBuilder;
    #[cfg(feature = "pool_tests")]
    use aries_vcx::common::test_utils::create_and_write_test_schema;
    #[cfg(feature = "pool_tests")]
    use aries_vcx::utils;
    use aries_vcx::utils::devsetup::SetupMocks;
    use aries_vcx::utils::constants::SCHEMA_ID;

    #[cfg(feature = "pool_tests")]
    use aries_vcx::utils::get_temp_dir_path;

    #[cfg(feature = "pool_tests")]
    use crate::api_lib::api_handle::revocation_registry::RevocationRegistryConfig;
    #[cfg(feature = "pool_tests")]

    use crate::api_lib::api_handle::revocation_registry;
    use crate::api_lib::api_handle::schema;

    #[cfg(feature = "pool_tests")]
    use crate::api_lib::utils::devsetup::SetupGlobalsWalletPoolAgency;

    use super::*;

    pub async fn create_and_publish_nonrevocable_creddef() -> (u32, u32) {
        let schema_handle = schema::tests::create_schema_real().await;
        sleep(Duration::from_secs(1));

        let schema_id = schema::get_schema_id(schema_handle).unwrap();
        let issuer_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let cred_def_handle = create(
            "1".to_string(),
            schema_id,
            issuer_did.clone(),
            "tag_1".to_string(),
            false,
        )
        .await
        .unwrap();

        publish(cred_def_handle).await.unwrap();
        (schema_handle, cred_def_handle)
    }

    #[cfg(feature = "general_test")]
    #[tokio::test]
    async fn test_create_cred_def() {
        let _setup = SetupMocks::init();
        let (_, _) = create_and_publish_nonrevocable_creddef().await;
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn create_revocable_cred_def_and_check_tails_location() {
        SetupGlobalsWalletPoolAgency::run(|setup| async move {

        let profile = get_main_profile().unwrap();
        let (schema_id, _) =
            create_and_write_test_schema(&profile, &setup.setup.institution_did, utils::constants::DEFAULT_SCHEMA_ATTRS).await;
        let issuer_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        let revocation_details = RevocationDetailsBuilder::default()
            .support_revocation(true)
            .tails_dir(get_temp_dir_path("tails.txt").to_str().unwrap())
            .max_creds(2 as u32)
            .build()
            .unwrap();
        let _revocation_details = serde_json::to_string(&revocation_details).unwrap();
        let handle_cred_def = create("1".to_string(), schema_id, issuer_did.clone(), "tag1".to_string(), true)
            .await
            .unwrap();
        publish(handle_cred_def).await.unwrap();

        let rev_reg_config = RevocationRegistryConfig {
            issuer_did,
            cred_def_id: get_cred_def_id(handle_cred_def).unwrap(),
            tag: 1,
            tails_dir: String::from(get_temp_dir_path("tails.txt").to_str().unwrap()),
            max_creds: 2,
        };
        let handle_rev_reg = revocation_registry::create(rev_reg_config).await.unwrap();
        let tails_url = utils::constants::TEST_TAILS_URL;

        revocation_registry::publish(handle_rev_reg, tails_url).await.unwrap();
        let rev_reg_def = revocation_registry::get_rev_reg_def(handle_rev_reg).unwrap();
        assert_eq!(rev_reg_def.value.tails_location, tails_url);
        }).await;
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_create_credential_def_real() {
        SetupGlobalsWalletPoolAgency::run(|_setup| async move {

        let (_, handle) = create_and_publish_nonrevocable_creddef().await;

        let _source_id = get_source_id(handle).unwrap();
        let _cred_def_id = get_cred_def_id(handle).unwrap();
        let _schema_json = to_string(handle).unwrap();
        }).await;
    }

    #[cfg(feature = "general_test")]
    #[tokio::test]
    async fn test_to_string_succeeds() {
        let _setup = SetupMocks::init();

        let (_, cred_def_handle) = create_and_publish_nonrevocable_creddef().await;

        let credential_string = to_string(cred_def_handle).unwrap();
        let credential_values: serde_json::Value = serde_json::from_str(&credential_string).unwrap();
        assert_eq!(credential_values["version"].clone(), "1.0");
    }

    #[cfg(feature = "general_test")]
    #[tokio::test]
    async fn test_from_string_succeeds() {
        let _setup = SetupMocks::init();

        let (_, cred_def_handle) = create_and_publish_nonrevocable_creddef().await;
        let credentialdef_data = to_string(cred_def_handle).unwrap();
        assert!(!credentialdef_data.is_empty());
        release(cred_def_handle).unwrap();

        let new_handle = from_string(&credentialdef_data).unwrap();
        let new_credentialdef_data = to_string(new_handle).unwrap();

        let credentialdef1: CredentialDef = CredentialDef::from_string(&credentialdef_data).unwrap();
        let credentialdef2: CredentialDef = CredentialDef::from_string(&new_credentialdef_data).unwrap();

        assert_eq!(credentialdef1, credentialdef2);
        assert_eq!(
            CredentialDef::from_string("{}").unwrap_err().kind(),
            aries_vcx::error::VcxErrorKind::CreateCredDef
        );
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_release_all() {
        let _setup = SetupMocks::init();

        let issuer_did = String::from("4fUDR9R7fjwELRvH9JT6HH");
        let h1 = create(
            "SourceId".to_string(),
            SCHEMA_ID.to_string(),
            issuer_did.clone(),
            "tag".to_string(),
            false,
        )
        .await
        .unwrap();
        let h2 = create(
            "SourceId".to_string(),
            SCHEMA_ID.to_string(),
            issuer_did.clone(),
            "tag".to_string(),
            false,
        )
        .await
        .unwrap();
        let h3 = create(
            "SourceId".to_string(),
            SCHEMA_ID.to_string(),
            issuer_did.clone(),
            "tag".to_string(),
            false,
        )
        .await
        .unwrap();
        let h4 = create(
            "SourceId".to_string(),
            SCHEMA_ID.to_string(),
            issuer_did.clone(),
            "tag".to_string(),
            false,
        )
        .await
        .unwrap();
        let h5 = create(
            "SourceId".to_string(),
            SCHEMA_ID.to_string(),
            issuer_did.clone(),
            "tag".to_string(),
            false,
        )
        .await
        .unwrap();
        release_all();
        assert_eq!(release(h1).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(release(h2).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(release(h3).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(release(h4).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(release(h5).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
    }
}
