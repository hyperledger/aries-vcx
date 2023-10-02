use aries_vcx::common::primitives::credential_definition::{
    CredentialDef, CredentialDefConfigBuilder, PublicEntityStateType,
};

use crate::{
    api_vcx::{
        api_global::profile::{
            get_main_anoncreds, get_main_anoncreds_ledger_read, get_main_anoncreds_ledger_write,
        },
        api_handle::object_cache::ObjectCache,
    },
    errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
};

lazy_static! {
    pub static ref CREDENTIALDEF_MAP: ObjectCache<CredentialDef> =
        ObjectCache::<CredentialDef>::new("credential-defs-cache");
}

pub async fn create(
    issuer_did: String,
    source_id: String,
    schema_id: String,
    tag: String,
    support_revocation: bool,
) -> LibvcxResult<u32> {
    let config = CredentialDefConfigBuilder::default()
        .issuer_did(issuer_did)
        .schema_id(schema_id)
        .tag(tag)
        .build()
        .map_err(|err| {
            LibvcxError::from_msg(
                LibvcxErrorKind::InvalidConfiguration,
                format!(
                    "Failed build credential config using provided parameters: {:?}",
                    err
                ),
            )
        })?;
    let cred_def = CredentialDef::create(
        &get_main_anoncreds_ledger_read()?,
        &get_main_anoncreds()?,
        source_id,
        config,
        support_revocation,
    )
    .await?;
    let handle = CREDENTIALDEF_MAP.add(cred_def)?;
    Ok(handle)
}

pub async fn publish(handle: u32) -> LibvcxResult<()> {
    let mut cd = CREDENTIALDEF_MAP.get_cloned(handle)?;
    if !cd.was_published() {
        cd = cd
            .publish_cred_def(
                &get_main_anoncreds_ledger_read()?,
                &get_main_anoncreds_ledger_write()?,
            )
            .await?;
    } else {
        info!("publish >>> Credential definition was already published")
    }
    CREDENTIALDEF_MAP.insert(handle, cd)
}

pub fn is_valid_handle(handle: u32) -> bool {
    CREDENTIALDEF_MAP.has_handle(handle)
}

pub fn to_string(handle: u32) -> LibvcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |cd| cd.to_string().map_err(|err| err.into()))
}

pub fn from_string(data: &str) -> LibvcxResult<u32> {
    let cred_def: CredentialDef = CredentialDef::from_string(data)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::CreateCredDef, e.to_string()))?;
    CREDENTIALDEF_MAP.add(cred_def)
}

pub fn get_source_id(handle: u32) -> LibvcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |c| Ok(c.get_source_id().clone()))
}

pub fn get_cred_def_id(handle: u32) -> LibvcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |c| Ok(c.get_cred_def_id()))
}

pub fn release(handle: u32) -> LibvcxResult<()> {
    CREDENTIALDEF_MAP
        .release(handle)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::InvalidCredDefHandle, e.to_string()))
}

pub fn release_all() {
    CREDENTIALDEF_MAP.drain().ok();
}

pub async fn update_state(handle: u32) -> LibvcxResult<u32> {
    let mut cd = CREDENTIALDEF_MAP.get_cloned(handle)?;
    let res = cd.update_state(&get_main_anoncreds_ledger_read()?).await?;
    CREDENTIALDEF_MAP.insert(handle, cd)?;
    Ok(res)
}

pub fn get_state(handle: u32) -> LibvcxResult<u32> {
    CREDENTIALDEF_MAP.get(handle, |s| Ok(s.get_state()))
}

pub fn check_is_published(handle: u32) -> LibvcxResult<bool> {
    CREDENTIALDEF_MAP.get(handle, |s| Ok(PublicEntityStateType::Published == s.state))
}

#[cfg(test)]
pub mod tests {
    use std::{thread::sleep, time::Duration};

    use aries_vcx::{
        aries_vcx_core::ledger::indy::pool::test_utils::get_temp_dir_path,
        common::test_utils::create_and_write_test_schema,
        global::settings::DEFAULT_DID,
        utils,
        utils::{constants::SCHEMA_ID, devsetup::SetupMocks},
    };

    use super::*;
    use crate::api_vcx::{
        api_handle::{revocation_registry, revocation_registry::RevocationRegistryConfig, schema},
        utils::devsetup::SetupGlobalsWalletPoolAgency,
    };

    #[tokio::test]
    async fn test_vcx_credentialdef_release() {
        let _setup = SetupMocks::init();
        let schema_handle = schema::test_utils::create_schema_real().await;
        sleep(Duration::from_secs(1));

        let schema_id = schema::get_schema_id(schema_handle).unwrap();

        let cred_def_handle = create(
            DEFAULT_DID.to_owned(),
            "1".to_string(),
            schema_id,
            "tag_1".to_string(),
            false,
        )
        .await
        .unwrap();
        release(cred_def_handle).unwrap();
        assert_eq!(
            to_string(cred_def_handle).unwrap_err().kind,
            LibvcxErrorKind::InvalidHandle
        )
    }

    pub async fn create_and_publish_nonrevocable_creddef() -> (u32, u32) {
        let schema_handle = schema::test_utils::create_schema_real().await;
        sleep(Duration::from_secs(1));

        let schema_id = schema::get_schema_id(schema_handle).unwrap();
        let cred_def_handle = create(
            DEFAULT_DID.to_owned(),
            "1".to_string(),
            schema_id,
            "tag_1".to_string(),
            false,
        )
        .await
        .unwrap();

        publish(cred_def_handle).await.unwrap();
        (schema_handle, cred_def_handle)
    }

    #[tokio::test]
    async fn test_create_cred_def() {
        let _setup = SetupMocks::init();
        let (_, _) = create_and_publish_nonrevocable_creddef().await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_credential_def_real() {
        SetupGlobalsWalletPoolAgency::run(|_setup| async move {
            let (_, handle) = create_and_publish_nonrevocable_creddef().await;

            let _source_id = get_source_id(handle).unwrap();
            let _cred_def_id = get_cred_def_id(handle).unwrap();
            let _schema_json = to_string(handle).unwrap();
        })
        .await;
    }

    #[tokio::test]
    async fn test_to_string_succeeds() {
        let _setup = SetupMocks::init();

        let (_, cred_def_handle) = create_and_publish_nonrevocable_creddef().await;

        let credential_string = to_string(cred_def_handle).unwrap();
        let credential_values: serde_json::Value =
            serde_json::from_str(&credential_string).unwrap();
        assert_eq!(credential_values["version"].clone(), "1.0");
    }

    #[tokio::test]
    async fn test_from_string_succeeds() {
        let _setup = SetupMocks::init();

        let (_, cred_def_handle) = create_and_publish_nonrevocable_creddef().await;
        let credentialdef_data = to_string(cred_def_handle).unwrap();
        assert!(!credentialdef_data.is_empty());
        release(cred_def_handle).unwrap();

        let new_handle = from_string(&credentialdef_data).unwrap();
        let new_credentialdef_data = to_string(new_handle).unwrap();

        let credentialdef1: CredentialDef =
            CredentialDef::from_string(&credentialdef_data).unwrap();
        let credentialdef2: CredentialDef =
            CredentialDef::from_string(&new_credentialdef_data).unwrap();

        assert_eq!(credentialdef1, credentialdef2);
        assert_eq!(
            from_string("{}").unwrap_err().kind(),
            LibvcxErrorKind::CreateCredDef
        );
    }

    #[tokio::test]
    async fn test_release_all() {
        let _setup = SetupMocks::init();

        let issuer_did = String::from("4fUDR9R7fjwELRvH9JT6HH");
        let h1 = create(
            issuer_did.clone(),
            "SourceId".to_string(),
            SCHEMA_ID.to_string(),
            "tag".to_string(),
            false,
        )
        .await
        .unwrap();

        let h2 = create(
            issuer_did,
            "SourceId".to_string(),
            SCHEMA_ID.to_string(),
            "tag".to_string(),
            false,
        )
        .await
        .unwrap();

        release_all();

        assert!(!is_valid_handle(h1));
        assert!(!is_valid_handle(h2));
    }
}
