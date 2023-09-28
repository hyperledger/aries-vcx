use std::string::ToString;

use aries_vcx::{
    common::primitives::credential_schema::Schema, global::settings::CONFIG_INSTITUTION_DID,
};
use serde_json;

use crate::{
    api_vcx::{
        api_global::{
            profile::{
                get_main_anoncreds, get_main_anoncreds_ledger_read, get_main_anoncreds_ledger_write,
            },
            settings::get_config_value,
        },
        api_handle::object_cache::ObjectCache,
    },
    errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
};

lazy_static! {
    static ref SCHEMA_MAP: ObjectCache<Schema> = ObjectCache::<Schema>::new("schemas-cache");
}

pub async fn create_and_publish_schema(
    source_id: &str,
    name: String,
    version: String,
    data: String,
) -> LibvcxResult<u32> {
    let issuer_did = get_config_value(CONFIG_INSTITUTION_DID)?;

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
        LibvcxError::from_msg(
            LibvcxErrorKind::SerializationError,
            format!("Cannot deserialize schema data to vec: {:?}", err),
        )
    })?;
    let schema = Schema::create(
        &get_main_anoncreds()?,
        source_id,
        &issuer_did,
        &name,
        &version,
        &data,
    )
    .await?
    .publish(&get_main_anoncreds_ledger_write()?, None)
    .await?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    debug!(
        "created schema on ledger with id: {}",
        schema.get_schema_id()
    );

    SCHEMA_MAP
        .add(schema)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::CreateSchema, e.to_string()))
}

pub async fn get_schema_attrs(source_id: String, schema_id: String) -> LibvcxResult<(u32, String)> {
    trace!(
        "get_schema_attrs >>> source_id: {}, schema_id: {}",
        source_id,
        schema_id
    );
    let schema_ledger_data_json = get_main_anoncreds_ledger_read()?
        .get_schema(&schema_id, None)
        .await?;
    let schema = Schema::create_from_ledger_json(&schema_ledger_data_json, &source_id, &schema_id)?;
    let schema_json = schema.to_string_versioned()?;

    let handle = SCHEMA_MAP
        .add(schema)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::CreateSchema, e.to_string()))?;

    Ok((handle, schema_json))
}

pub fn is_valid_handle(handle: u32) -> bool {
    SCHEMA_MAP.has_handle(handle)
}

pub fn to_string(handle: u32) -> LibvcxResult<String> {
    SCHEMA_MAP.get(handle, |s| {
        s.to_string_versioned().map_err(|err| err.into())
    })
}

pub fn get_source_id(handle: u32) -> LibvcxResult<String> {
    SCHEMA_MAP.get(handle, |s| Ok(s.get_source_id()))
}

pub fn get_schema_id(handle: u32) -> LibvcxResult<String> {
    SCHEMA_MAP.get(handle, |s| Ok(s.get_schema_id()))
}

pub fn from_string(schema_data: &str) -> LibvcxResult<u32> {
    let schema: Schema = Schema::from_string_versioned(schema_data)?;
    SCHEMA_MAP.add(schema)
}

pub fn release(handle: u32) -> LibvcxResult<()> {
    SCHEMA_MAP
        .release(handle)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::InvalidSchemaHandle, e.to_string()))
}

pub fn release_all() {
    SCHEMA_MAP.drain().ok();
}

pub async fn update_state(schema_handle: u32) -> LibvcxResult<u32> {
    let mut schema = SCHEMA_MAP.get_cloned(schema_handle)?;
    let res = schema
        .update_state(&get_main_anoncreds_ledger_read()?)
        .await?;
    SCHEMA_MAP.insert(schema_handle, schema)?;
    Ok(res)
}

pub fn get_state(handle: u32) -> LibvcxResult<u32> {
    SCHEMA_MAP.get(handle, |s| Ok(s.get_state()))
}

#[allow(clippy::unwrap_used)]
pub mod test_utils {
    use rand::Rng;

    use super::*;
    use crate::api_vcx::api_global::settings::get_config_value;

    pub fn prepare_schema_data() -> (String, String, String, String) {
        let data = json!(data()).to_string();
        let schema_name: String = aries_vcx::utils::random::generate_random_schema_name();
        let schema_version: String = format!(
            "{}.{}",
            rand::thread_rng().gen::<u32>(),
            rand::thread_rng().gen::<u32>()
        );
        let did = get_config_value(CONFIG_INSTITUTION_DID).unwrap();

        (did, schema_name, schema_version, data)
    }

    // TODO: Reuse test utils code and data
    pub async fn create_schema_real() -> u32 {
        let (_did, schema_name, schema_version, data) = prepare_schema_data();
        create_and_publish_schema("id", schema_name, schema_version, data)
            .await
            .unwrap()
    }

    pub fn check_schema(schema_handle: u32, schema_json: &str, schema_id: &str, data: &str) {
        let schema: Schema = Schema::from_string_versioned(schema_json).unwrap();
        info!("schema: {:?}", schema);
        assert_eq!(schema.schema_id, schema_id.to_string());

        let mut schema_data = schema.data;
        schema_data.sort();
        let mut vec_data: Vec<String> = serde_json::from_str(data).unwrap();
        vec_data.sort();
        assert_eq!(schema_data, vec_data);

        assert!(schema_handle > 0);
    }

    fn data() -> Vec<String> {
        vec![
            "address1".to_string(),
            "address2".to_string(),
            "zip".to_string(),
            "city".to_string(),
            "state".to_string(),
        ]
    }
}

#[cfg(test)]
pub mod tests {
    use aries_vcx::{
        common::test_utils::create_and_write_test_schema,
        global::settings::{set_config_value, DEFAULT_DID},
        utils::{
            constants,
            constants::SCHEMA_ID,
            devsetup::{SetupDefaults, SetupEmpty, SetupMocks},
        },
    };

    use super::*;
    use crate::api_vcx::{
        api_handle::{
            schema,
            schema::test_utils::{check_schema, create_schema_real, prepare_schema_data},
        },
        utils::devsetup::SetupGlobalsWalletPoolAgency,
    };

    #[tokio::test]
    async fn test_vcx_schema_release() {
        let _setup = SetupMocks::init();

        let (_did, schema_name, schema_version, data) = prepare_schema_data();
        let handle = create_and_publish_schema(
            "test_create_schema_success",
            schema_name,
            schema_version,
            data.clone(),
        )
        .await
        .unwrap();
        release(handle).unwrap();
        assert_eq!(
            to_string(handle).unwrap_err().kind,
            LibvcxErrorKind::InvalidHandle
        )
    }

    #[tokio::test]
    async fn test_create_schema_success() {
        let _setup = SetupMocks::init();

        let (_did, schema_name, schema_version, data) = prepare_schema_data();
        create_and_publish_schema(
            "test_create_schema_success",
            schema_name,
            schema_version,
            data,
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_get_schema_attrs_success() {
        let _setup = SetupMocks::init();

        let (handle, schema_json) =
            get_schema_attrs("Check For Success".to_string(), SCHEMA_ID.to_string())
                .await
                .unwrap();

        check_schema(
            handle,
            &schema_json,
            SCHEMA_ID,
            r#"["name","age","height","sex"]"#,
        );
    }

    #[tokio::test]
    async fn test_create_schema_fails() {
        let _setup = SetupDefaults::init();
        set_config_value(CONFIG_INSTITUTION_DID, DEFAULT_DID).unwrap();
        let err =
            create_and_publish_schema("1", "name".to_string(), "1.0".to_string(), "".to_string())
                .await
                .unwrap_err();
        assert_eq!(err.kind(), LibvcxErrorKind::SerializationError)
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_schema_attrs_from_ledger() {
        SetupGlobalsWalletPoolAgency::run(|setup| async move {
            let schema = create_and_write_test_schema(
                &get_main_anoncreds().unwrap(),
                &get_main_anoncreds_ledger_write().unwrap(),
                &setup.institution_did,
                constants::DEFAULT_SCHEMA_ATTRS,
            )
            .await;

            let (schema_handle, schema_attrs) =
                get_schema_attrs("id".to_string(), schema.schema_id.clone())
                    .await
                    .unwrap();

            check_schema(
                schema_handle,
                &schema_attrs,
                &schema.schema_id,
                constants::DEFAULT_SCHEMA_ATTRS,
            );
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_schema_with_pool() {
        SetupGlobalsWalletPoolAgency::run(|_setup| async move {
            let handle = create_schema_real().await;

            let _source_id = get_source_id(handle).unwrap();
            let _schema_id = get_schema_id(handle).unwrap();
            let _schema_json = to_string(handle).unwrap();
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_duplicate_fails() {
        SetupGlobalsWalletPoolAgency::run(|_setup| async move {
            let (_did, schema_name, schema_version, data) = prepare_schema_data();

            create_and_publish_schema(
                "id",
                schema_name.clone(),
                schema_version.clone(),
                data.clone(),
            )
            .await
            .unwrap();

            let err = create_and_publish_schema("id_2", schema_name, schema_version, data)
                .await
                .unwrap_err();
            error!("err: {:?}", err);
            // .unwrap_err();

            assert_eq!(err.kind(), LibvcxErrorKind::DuplicationSchema);
        })
        .await;
    }

    #[tokio::test]
    async fn test_release_all() {
        let _setup = SetupMocks::init();

        let (_did, schema_name, version, data) = prepare_schema_data();

        let h1 = create_and_publish_schema("1", schema_name.clone(), version.clone(), data.clone())
            .await
            .unwrap();
        let h2 = create_and_publish_schema("2", schema_name.clone(), version.clone(), data.clone())
            .await
            .unwrap();
        let h3 = create_and_publish_schema("3", schema_name.clone(), version.clone(), data.clone())
            .await
            .unwrap();

        release_all();

        assert!(!is_valid_handle(h1));
        assert!(!is_valid_handle(h2));
        assert!(!is_valid_handle(h3));
    }

    #[test]
    fn test_handle_errors() {
        let _setup = SetupEmpty::init();

        assert_eq!(
            to_string(13435178).unwrap_err().kind(),
            LibvcxErrorKind::InvalidHandle
        );
    }

    #[tokio::test]
    #[ignore]
    async fn test_vcx_schema_get_state_with_ledger() {
        SetupGlobalsWalletPoolAgency::run(|_setup| async move {
            let handle = create_schema_real().await;
            assert_eq!(1, get_state(handle).unwrap());
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_vcx_create_schema_with_pool() {
        SetupGlobalsWalletPoolAgency::run(|_setup| async move {
            let (_issuer_did, schema_name, schema_version, schema_data) = prepare_schema_data();
            let _schema_handle = schema::create_and_publish_schema(
                "source_id",
                schema_name,
                schema_version,
                schema_data,
            )
            .await
            .unwrap();
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_vcx_schema_serialize_contains_version() {
        SetupGlobalsWalletPoolAgency::run(|_setup| async move {
            let (_issuer_did, schema_name, schema_version, schema_data) = prepare_schema_data();
            let schema_handle = schema::create_and_publish_schema(
                "source_id",
                schema_name,
                schema_version,
                schema_data,
            )
            .await
            .unwrap();

            let schema_json = schema::to_string(schema_handle).unwrap();

            let j: serde_json::Value = serde_json::from_str(&schema_json).unwrap();
            let _schema: Schema = serde_json::from_value(j["data"].clone()).unwrap();
            assert_eq!(j["version"], "1.0");
        })
        .await;
    }

    #[tokio::test]
    #[ignore]
    async fn test_vcx_schema_get_attrs_with_pool() {
        SetupGlobalsWalletPoolAgency::run(|_setup| async move {
            let (_issuer_did, schema_name, schema_version, schema_data) = prepare_schema_data();
            let schema_handle = schema::create_and_publish_schema(
                "source_id",
                schema_name,
                schema_version,
                schema_data,
            )
            .await
            .unwrap();
            let _schema_json_1 = schema::to_string(schema_handle).unwrap();
            let schema_id = schema::get_schema_id(schema_handle).unwrap();

            let (_schema_handle, schema_json_2) =
                schema::get_schema_attrs("source_id".into(), schema_id)
                    .await
                    .unwrap();
            let j: serde_json::Value = serde_json::from_str(&schema_json_2).unwrap();
            let _schema: Schema = serde_json::from_value(j["data"].clone()).unwrap();
            assert_eq!(j["version"], "1.0");
        })
        .await;
    }
}
