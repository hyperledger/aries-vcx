use std::string::ToString;
use std::sync::Arc;

use serde_json;

use aries_vcx::error::{VcxError, VcxErrorKind, VcxResult};
use aries_vcx::vdrtools::{PoolHandle, WalletHandle};
use aries_vcx::xyz::primitives::credential_schema::Schema;

use crate::api_lib::api_handle::object_cache::ObjectCache;
use crate::api_lib::global::profile::{get_main_profile, indy_handles_to_profile};

lazy_static! {
    static ref SCHEMA_MAP: ObjectCache<Schema> = ObjectCache::<Schema>::new("schemas-cache");
}

pub async fn create_and_publish_schema(
    source_id: &str,
    issuer_did: String,
    name: String,
    version: String,
    data: String,
) -> VcxResult<u32> {
    trace!(
        "create_new_schema >>> source_id: {}, issuer_did: {}, name: {}, version: {}, data: {}",
        source_id,
        issuer_did,
        name,
        version,
        data
    );
    debug!(
        "creating schema with source_id: {}, name: {}, issuer_did: {}",
        source_id, name, issuer_did
    );

    let data: Vec<String> = serde_json::from_str(&data).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!("Cannot deserialize schema data to vec: {:?}", err),
        )
    })?;
    let profile = get_main_profile()?;
    let schema = Schema::create(&profile, source_id, &issuer_did, &name, &version, &data)
        .await?
        .publish(&profile, None)
        .await?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    debug!("created schema on ledger with id: {}", schema.get_schema_id());

    SCHEMA_MAP
        .add(schema)
        .or(Err(VcxError::from(VcxErrorKind::CreateSchema)))
}

pub async fn prepare_schema_for_endorser(
    source_id: &str,
    issuer_did: String,
    name: String,
    version: String,
    data: String,
    endorser: String,
) -> VcxResult<(u32, String)> {
    trace!(
        "create_schema_for_endorser >>> source_id: {}, issuer_did: {}, name: {}, version: {}, data: {}, endorser: {}",
        source_id,
        issuer_did,
        name,
        version,
        data,
        endorser
    );
    debug!(
        "preparing schema for endorser with source_id: {}, name: {}, issuer_did: {}",
        source_id, name, issuer_did
    );

    let data: Vec<String> = serde_json::from_str(&data).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!("Cannot deserialize schema data to vec: {:?}", err),
        )
    })?;
    let profile = get_main_profile()?;
    let schema = Schema::create(&profile, source_id, &issuer_did, &name, &version, &data).await?;
    let schema_json = schema
        .get_schema_json(&profile)
        .await?;
    let schema_id = schema.get_schema_id();
    let ledger = Arc::clone(&profile).inject_ledger();
    let schema_request = ledger.build_schema_request(&issuer_did, &schema_json).await?;
    let schema_request =
       ledger.set_endorser(&issuer_did, &schema_request, &endorser).await?;

    debug!("prepared schema for endorser with id: {}", schema_id);

    let schema_handle = SCHEMA_MAP
        .add(schema)
        .or(Err(VcxError::from(VcxErrorKind::CreateSchema)))?;

    Ok((schema_handle, schema_request))
}

pub async fn get_schema_attrs(source_id: String, schema_id: String) -> VcxResult<(u32, String)> {
    trace!(
        "get_schema_attrs >>> source_id: {}, schema_id: {}",
        source_id,
        schema_id
    );
    let profile = get_main_profile()?;
    let schema = Schema::create_from_ledger_json(
        &profile,
        &source_id,
        &schema_id,
    )
    .await
    .map_err(|err| err.map(VcxErrorKind::CreateSchema, "Create schema from ledger failed"))?;
    let schema_json = schema.to_string()?;

    let handle = SCHEMA_MAP
        .add(schema)
        .or(Err(VcxError::from(VcxErrorKind::CreateSchema)))?;

    Ok((handle, schema_json))
}

pub fn is_valid_handle(handle: u32) -> bool {
    SCHEMA_MAP.has_handle(handle)
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    SCHEMA_MAP.get(handle, |s| s.to_string().map_err(|err| err.into()))
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    SCHEMA_MAP.get(handle, |s| Ok(s.get_source_id().to_string()))
}

pub fn get_schema_id(handle: u32) -> VcxResult<String> {
    SCHEMA_MAP.get(handle, |s| Ok(s.get_schema_id().to_string()))
}

pub fn from_string(schema_data: &str) -> VcxResult<u32> {
    let schema: Schema = Schema::from_str(schema_data)?;
    SCHEMA_MAP.add(schema)
}

pub fn release(handle: u32) -> VcxResult<()> {
    SCHEMA_MAP
        .release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidSchemaHandle)))
}

pub fn release_all() {
    SCHEMA_MAP.drain().ok();
}

pub async fn update_state(wallet_handle: WalletHandle, pool_handle: PoolHandle, schema_handle: u32) -> VcxResult<u32> {
    let mut schema = SCHEMA_MAP.get_cloned(schema_handle)?;
    let profile = indy_handles_to_profile(wallet_handle, pool_handle);
    let res = schema.update_state(&profile).await?;
    SCHEMA_MAP.insert(schema_handle, schema)?;
    Ok(res)
}

pub fn get_state(handle: u32) -> VcxResult<u32> {
    SCHEMA_MAP.get(handle, |s| Ok(s.get_state()))
}

#[cfg(test)]
pub mod tests {
    use rand::Rng;

    use aries_vcx::global::settings;
    #[cfg(feature = "pool_tests")]
    use aries_vcx::xyz::ledger::transactions::add_new_did;
    #[cfg(feature = "pool_tests")]
    use aries_vcx::xyz::test_utils::create_and_write_test_schema;
    #[cfg(feature = "pool_tests")]
    use aries_vcx::utils::constants;
    use aries_vcx::utils::constants::SCHEMA_ID;
    use aries_vcx::utils::devsetup::{SetupDefaults, SetupEmpty, SetupMocks};
    use crate::api_lib::global::wallet::get_main_wallet_handle;

    #[cfg(feature = "pool_tests")]
    use crate::api_lib::api_handle::schema;
    #[cfg(feature = "pool_tests")]
    use crate::api_lib::utils::devsetup::SetupGlobalsWalletPoolAgency;

    use super::*;

    fn data() -> Vec<String> {
        vec![
            "address1".to_string(),
            "address2".to_string(),
            "zip".to_string(),
            "city".to_string(),
            "state".to_string(),
        ]
    }

    pub fn prepare_schema_data() -> (String, String, String, String) {
        let data = json!(data()).to_string();
        let schema_name: String = aries_vcx::utils::random::generate_random_schema_name();
        let schema_version: String = format!(
            "{}.{}",
            rand::thread_rng().gen::<u32>().to_string(),
            rand::thread_rng().gen::<u32>().to_string()
        );
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        (did, schema_name, schema_version, data)
    }

    pub async fn create_schema_real() -> u32 {
        let (did, schema_name, schema_version, data) = prepare_schema_data();
        create_and_publish_schema("id", did, schema_name, schema_version, data)
            .await
            .unwrap()
    }

    fn check_schema(schema_handle: u32, schema_json: &str, schema_id: &str, data: &str) {
        let schema: Schema = Schema::from_str(schema_json).unwrap();
        assert_eq!(schema.schema_id, schema_id.to_string());
        assert_eq!(schema.data.clone().sort(), vec!(data).sort());
        assert!(schema_handle > 0);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_create_schema_to_string() {
        let _setup = SetupMocks::init();

        let (did, schema_name, schema_version, data) = prepare_schema_data();
        let handle = create_and_publish_schema(
            "test_create_schema_success",
            did,
            schema_name,
            schema_version,
            data.clone(),
        )
        .await
        .unwrap();

        let schema_id = get_schema_id(handle).unwrap();
        let create_schema_json = to_string(handle).unwrap();

        let value: serde_json::Value = serde_json::from_str(&create_schema_json).unwrap();
        assert_eq!(value["version"], "1.0");
        assert!(value["data"].is_object());

        let handle = from_string(&create_schema_json).unwrap();

        assert_eq!(
            get_source_id(handle).unwrap(),
            String::from("test_create_schema_success")
        );
        check_schema(handle, &create_schema_json, &schema_id, &data);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_create_schema_success() {
        let _setup = SetupMocks::init();

        let (did, schema_name, schema_version, data) = prepare_schema_data();
        create_and_publish_schema("test_create_schema_success", did, schema_name, schema_version, data)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_prepare_schema_success() {
        let _setup = SetupMocks::init();

        let (did, schema_name, schema_version, data) = prepare_schema_data();
        prepare_schema_for_endorser(
            "test_create_schema_success",
            did,
            schema_name,
            schema_version,
            data,
            "V4SGRU86Z58d6TV7PBUe6f".to_string(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_get_schema_attrs_success() {
        let _setup = SetupMocks::init();

        let (handle, schema_json) = get_schema_attrs("Check For Success".to_string(), SCHEMA_ID.to_string())
            .await
            .unwrap();

        check_schema(handle, &schema_json, SCHEMA_ID, r#"["name","age","height","sex"]"#);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_create_schema_fails() {
        let _setup = SetupDefaults::init();

        let err = create_and_publish_schema(
            "1",
            "VsKV7grR1BUE29mG2Fm2kX".to_string(),
            "name".to_string(),
            "1.0".to_string(),
            "".to_string(),
        )
        .await
        .unwrap_err();
        assert_eq!(err.kind(), VcxErrorKind::SerializationError)
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_get_schema_attrs_from_ledger() {
        SetupGlobalsWalletPoolAgency::run(|setup| async move {

        let profile = get_main_profile().unwrap();

        let (schema_id, _) = create_and_write_test_schema(
            &profile,
            &setup.setup.institution_did,
            constants::DEFAULT_SCHEMA_ATTRS,
        )
        .await;

        let (schema_handle, schema_attrs) = get_schema_attrs("id".to_string(), schema_id.clone()).await.unwrap();

        check_schema(
            schema_handle,
            &schema_attrs,
            &schema_id,
            constants::DEFAULT_SCHEMA_ATTRS,
        );
        }).await;
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_create_schema_with_pool() {
        SetupGlobalsWalletPoolAgency::run(|_setup| async move {

        let handle = create_schema_real().await;

        let _source_id = get_source_id(handle).unwrap();
        let _schema_id = get_schema_id(handle).unwrap();
        let _schema_json = to_string(handle).unwrap();
        }).await;
    }

    #[tokio::test]
    #[cfg(feature = "pool_tests")]
    async fn test_create_duplicate_fails() {
        SetupGlobalsWalletPoolAgency::run(|_setup| async move {

        let (did, schema_name, schema_version, data) = prepare_schema_data();

        create_and_publish_schema(
            "id",
            did.clone(),
            schema_name.clone(),
            schema_version.clone(),
            data.clone(),
        )
        .await
        .unwrap();

        let err = create_and_publish_schema("id_2", did, schema_name, schema_version, data)
            .await
            .unwrap_err();

        assert_eq!(err.kind(), VcxErrorKind::DuplicationSchema);
        }).await;
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_release_all() {
        let _setup = SetupMocks::init();

        let (did, schema_name, version, data) = prepare_schema_data();

        let h1 = create_and_publish_schema("1", did.clone(), schema_name.clone(), version.clone(), data.clone())
            .await
            .unwrap();
        let h2 = create_and_publish_schema("2", did.clone(), schema_name.clone(), version.clone(), data.clone())
            .await
            .unwrap();
        let h3 = create_and_publish_schema("3", did.clone(), schema_name.clone(), version.clone(), data.clone())
            .await
            .unwrap();
        let h4 = create_and_publish_schema("4", did.clone(), schema_name.clone(), version.clone(), data.clone())
            .await
            .unwrap();
        let h5 = create_and_publish_schema("5", did.clone(), schema_name.clone(), version.clone(), data.clone())
            .await
            .unwrap();

        release_all();

        assert_eq!(release(h1).unwrap_err().kind(), VcxErrorKind::InvalidSchemaHandle);
        assert_eq!(release(h2).unwrap_err().kind(), VcxErrorKind::InvalidSchemaHandle);
        assert_eq!(release(h3).unwrap_err().kind(), VcxErrorKind::InvalidSchemaHandle);
        assert_eq!(release(h4).unwrap_err().kind(), VcxErrorKind::InvalidSchemaHandle);
        assert_eq!(release(h5).unwrap_err().kind(), VcxErrorKind::InvalidSchemaHandle);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_handle_errors() {
        let _setup = SetupEmpty::init();

        assert_eq!(to_string(13435178).unwrap_err().kind(), VcxErrorKind::InvalidHandle);
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_vcx_endorse_schema() {
        SetupGlobalsWalletPoolAgency::run(|setup| async move {

        let (did, schema_name, schema_version, data) = prepare_schema_data();

        let profile = get_main_profile().unwrap();

        let (endorser_did, _) = add_new_did(
            &profile,
            &setup.setup.institution_did,
            Some("ENDORSER"),
        )
        .await.unwrap();

        let (schema_handle, schema_request) = prepare_schema_for_endorser(
            "test_vcx_schema_update_state_with_ledger",
            did,
            schema_name,
            schema_version,
            data,
            endorser_did.clone(),
        )
        .await
        .unwrap();
        assert_eq!(0, get_state(schema_handle).unwrap());
        assert_eq!(
            0,
            update_state(get_main_wallet_handle(), setup.setup.pool_handle, schema_handle)
                .await
                .unwrap()
        );

        let ledger = profile.inject_ledger();

        ledger.endorse_transaction(
            &endorser_did,
            &schema_request,
        )
        .await
        .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(1000));

        assert_eq!(
            1,
            update_state(get_main_wallet_handle(), setup.setup.pool_handle, schema_handle)
                .await
                .unwrap()
        );
        assert_eq!(1, get_state(schema_handle).unwrap());
        }).await;
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_vcx_schema_get_state_with_ledger() {
        SetupGlobalsWalletPoolAgency::run(|_setup| async move {

        let handle = create_schema_real().await;
        assert_eq!(1, get_state(handle).unwrap());
        }).await;
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_vcx_create_schema_with_pool() {
        SetupGlobalsWalletPoolAgency::run(|_setup| async move {

        let (issuer_did, schema_name, schema_version, schema_data) = prepare_schema_data();
        let _schema_handle =
            schema::create_and_publish_schema("source_id", issuer_did, schema_name, schema_version, schema_data)
                .await
                .unwrap();
        }).await;
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_vcx_schema_serialize_contains_version() {
        SetupGlobalsWalletPoolAgency::run(|_setup| async move {

        let (issuer_did, schema_name, schema_version, schema_data) = prepare_schema_data();
        let schema_handle =
            schema::create_and_publish_schema("source_id", issuer_did, schema_name, schema_version, schema_data)
                .await
                .unwrap();

        let schema_json = schema::to_string(schema_handle).unwrap();

        let j: serde_json::Value = serde_json::from_str(&schema_json).unwrap();
        let _schema: Schema = serde_json::from_value(j["data"].clone()).unwrap();
        assert_eq!(j["version"], "1.0");
        }).await;
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_vcx_schema_get_attrs_with_pool() {
        SetupGlobalsWalletPoolAgency::run(|_setup| async move {

        let (issuer_did, schema_name, schema_version, schema_data) = prepare_schema_data();
        let schema_handle =
            schema::create_and_publish_schema("source_id", issuer_did, schema_name, schema_version, schema_data)
                .await
                .unwrap();
        let _schema_json_1 = schema::to_string(schema_handle).unwrap();
        let schema_id = schema::get_schema_id(schema_handle).unwrap();

        let (_schema_handle, schema_json_2) = schema::get_schema_attrs("source_id".into(), schema_id).await.unwrap();
        let j: serde_json::Value = serde_json::from_str(&schema_json_2).unwrap();
        let _schema: Schema = serde_json::from_value(j["data"].clone()).unwrap();
        assert_eq!(j["version"], "1.0");
        }).await;
    }
}
