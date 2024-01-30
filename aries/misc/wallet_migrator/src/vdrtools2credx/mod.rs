pub mod conv;

use aries_vcx_core::wallet::base_wallet::record::Record;
use log::trace;

use crate::error::MigrationResult;

pub(crate) const INDY_DID: &str = "Indy::Did";
pub(crate) const INDY_KEY: &str = "Indy::Key";
pub(crate) const INDY_MASTER_SECRET: &str = "Indy::MasterSecret";
pub(crate) const INDY_CRED: &str = "Indy::Credential";
pub(crate) const INDY_CRED_DEF: &str = "Indy::CredentialDefinition";
pub(crate) const INDY_CRED_DEF_PRIV: &str = "Indy::CredentialDefinitionPrivateKey";
pub(crate) const INDY_CRED_DEF_CR_PROOF: &str = "Indy::CredentialDefinitionCorrectnessProof";
pub(crate) const INDY_SCHEMA: &str = "Indy::Schema";
pub(crate) const INDY_SCHEMA_ID: &str = "Indy::SchemaId";
pub(crate) const INDY_REV_REG: &str = "Indy::RevocationRegistry";
pub(crate) const INDY_REV_REG_DELTA: &str = "cache"; // very intuitive, indy devs
pub(crate) const INDY_REV_REG_INFO: &str = "Indy::RevocationRegistryInfo";
pub(crate) const INDY_REV_REG_DEF: &str = "Indy::RevocationRegistryDefinition";
pub(crate) const INDY_REV_REG_DEF_PRIV: &str = "Indy::RevocationRegistryDefinitionPrivate";

/// Contains the logic for record mapping and migration.
pub fn migrate_any_record(record: Record) -> MigrationResult<Option<Record>> {
    trace!("Migrating wallet record {record:?}");

    let record = match record.category().to_string().as_str() {
        // Indy wallet records - to be left alone!
        INDY_DID | INDY_KEY => Ok(Some(record)),
        // Master secret
        INDY_MASTER_SECRET => Ok(Some(Record::try_from_indy_record(
            conv::convert_master_secret(record.into())?,
        )?)),
        // Credential
        INDY_CRED => Ok(Some(Record::try_from_indy_record(conv::convert_cred(
            record.into(),
        )?)?)),
        INDY_CRED_DEF => Ok(Some(Record::try_from_indy_record(conv::convert_cred_def(
            record.into(),
        )?)?)),
        INDY_CRED_DEF_PRIV => Ok(Some(Record::try_from_indy_record(
            conv::convert_cred_def_priv_key(record.into())?,
        )?)),
        INDY_CRED_DEF_CR_PROOF => Ok(Some(Record::try_from_indy_record(
            conv::convert_cred_def_correctness_proof(record.into())?,
        )?)),
        // Schema
        INDY_SCHEMA => Ok(Some(Record::try_from_indy_record(conv::convert_schema(
            record.into(),
        )?)?)),
        INDY_SCHEMA_ID => Ok(Some(Record::try_from_indy_record(
            conv::convert_schema_id(record.into())?,
        )?)),
        // Revocation registry
        INDY_REV_REG => Ok(Some(Record::try_from_indy_record(conv::convert_rev_reg(
            record.into(),
        )?)?)),
        INDY_REV_REG_DELTA => Ok(Some(Record::try_from_indy_record(
            conv::convert_rev_reg_delta(record.into())?,
        )?)),
        INDY_REV_REG_INFO => Ok(Some(Record::try_from_indy_record(
            conv::convert_rev_reg_info(record.into())?,
        )?)),
        INDY_REV_REG_DEF => Ok(Some(Record::try_from_indy_record(
            conv::convert_rev_reg_def(record.into())?,
        )?)),
        INDY_REV_REG_DEF_PRIV => Ok(Some(Record::try_from_indy_record(
            conv::convert_rev_reg_def_priv(record.into())?,
        )?)),
        _ => Ok(None), // Ignore unknown/uninteresting records
    };

    trace!("Converted wallet record to {record:?}");
    record
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use aries_vcx_core::{
        anoncreds::credx_anoncreds::RevocationRegistryInfo,
        wallet::{base_wallet::record_category::RecordCategory, indy::IndySdkWallet},
    };
    use credx::{
        anoncreds_clsignatures::{bn::BigNumber, LinkSecret as ClLinkSecret},
        types::LinkSecret,
    };
    use serde_json::json;
    use vdrtools::{
        types::domain::wallet::{Config, Credentials, IndyRecord, KeyDerivationMethod},
        Locator, WalletHandle,
    };

    use super::*;
    use crate::migrate_wallet;

    const WALLET_KEY: &str = "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY";

    #[tokio::test]
    async fn test_sqlite_migration() {
        let (credentials, config) = make_wallet_reqs("wallet_test_migration".to_owned());
        let (new_credentials, new_config) = make_wallet_reqs("new_better_wallet".to_owned());

        test_migration(credentials, config, new_credentials, new_config).await;
    }

    #[tokio::test]
    async fn test_mysql_migration() {
        let wallet_name = "wallet_test_migration";
        let new_wallet_name = "new_better_wallet";

        let (mut credentials, mut config) = make_wallet_reqs(wallet_name.to_owned());
        let (mut new_credentials, mut new_config) = make_wallet_reqs(new_wallet_name.to_owned());

        config.storage_type = Some("mysql".to_owned());
        new_config.storage_type = Some("mysql".to_owned());

        let storage_config = json!({
            "read_host": "localhost",
            "write_host": "localhost",
            "port": 3306,
            "db_name": wallet_name,
            "default_connection_limit": 50
        });

        let new_storage_config = json!({
            "read_host": "localhost",
            "write_host": "localhost",
            "port": 3306,
            "db_name": new_wallet_name,
            "default_connection_limit": 50
        });

        let storage_credentials = json!({
            "user": "root",
            "pass": "mysecretpassword"
        });

        config.storage_config = Some(storage_config);
        credentials.storage_credentials = Some(storage_credentials.clone());

        new_config.storage_config = Some(new_storage_config);
        new_credentials.storage_credentials = Some(storage_credentials);

        test_migration(credentials, config, new_credentials, new_config).await;
    }

    macro_rules! add_wallet_item {
        ($wh:expr, $category:expr, $val:expr) => {
            Locator::instance()
                .non_secret_controller
                .add_record(
                    $wh,
                    $category.to_owned(),
                    "test_id".to_owned(),
                    serde_json::to_string(&$val).unwrap(),
                    None,
                )
                .await
                .unwrap();
        };
    }

    macro_rules! get_wallet_item {
        ($wh:expr, $category:expr, $res:ty) => {{
            let val = get_wallet_item_raw($wh, $category).await;
            serde_json::from_str::<$res>(&val).unwrap()
        }};
    }

    async fn test_migration(
        credentials: Credentials,
        config: Config,
        new_credentials: Credentials,
        new_config: Config,
    ) {
        // Removes old wallet if it already exists
        Locator::instance()
            .wallet_controller
            .delete(config.clone(), credentials.clone())
            .await
            .ok();

        // Create and open the old wallet
        // where we'll store old indy anoncreds types
        Locator::instance()
            .wallet_controller
            .create(config.clone(), credentials.clone())
            .await
            .unwrap();

        let src_wallet_handle = Locator::instance()
            .wallet_controller
            .open(config.clone(), credentials.clone())
            .await
            .unwrap();

        // Construct and add legacy indy records
        // These are dummy records with dummy values
        // and are NOT expected to be functional
        //
        // ################# Ingestion start #################

        // Master secret
        add_wallet_item!(
            src_wallet_handle,
            INDY_MASTER_SECRET,
            make_dummy_master_secret()
        );

        // Credential
        add_wallet_item!(src_wallet_handle, INDY_CRED, make_dummy_cred());
        add_wallet_item!(src_wallet_handle, INDY_CRED_DEF, make_dummy_cred_def());
        add_wallet_item!(
            src_wallet_handle,
            INDY_CRED_DEF_PRIV,
            make_dummy_cred_def_priv_key()
        );
        add_wallet_item!(
            src_wallet_handle,
            INDY_CRED_DEF_CR_PROOF,
            make_dummy_cred_def_correctness_proof()
        );

        // Schema
        add_wallet_item!(src_wallet_handle, INDY_SCHEMA, make_dummy_schema());
        add_wallet_item!(src_wallet_handle, INDY_SCHEMA_ID, make_dummy_schema_id());

        // Revocation registry
        add_wallet_item!(src_wallet_handle, INDY_REV_REG, make_dummy_rev_reg());
        add_wallet_item!(
            src_wallet_handle,
            INDY_REV_REG_DELTA,
            make_dummy_rev_reg_delta()
        );
        add_wallet_item!(
            src_wallet_handle,
            INDY_REV_REG_INFO,
            make_dummy_rev_reg_info()
        );
        add_wallet_item!(
            src_wallet_handle,
            INDY_REV_REG_DEF,
            make_dummy_rev_reg_def()
        );
        add_wallet_item!(
            src_wallet_handle,
            INDY_REV_REG_DEF_PRIV,
            make_dummy_rev_reg_def_priv()
        );

        // ################# Ingestion end #################

        // Remove new wallet if it already exists
        Locator::instance()
            .wallet_controller
            .delete(new_config.clone(), new_credentials.clone())
            .await
            .ok();

        Locator::instance()
            .wallet_controller
            .create(new_config.clone(), new_credentials.clone())
            .await
            .unwrap();

        let dest_wallet_handle = Locator::instance()
            .wallet_controller
            .open(new_config.clone(), new_credentials.clone())
            .await
            .unwrap();

        let src_wallet = IndySdkWallet::new(src_wallet_handle);
        let dest_wallet = IndySdkWallet::new(dest_wallet_handle);

        // Migrate the records
        migrate_wallet(&src_wallet, &dest_wallet, migrate_any_record)
            .await
            .unwrap();

        // Old wallet cleanup
        Locator::instance()
            .wallet_controller
            .close(src_wallet_handle)
            .await
            .unwrap();

        Locator::instance()
            .wallet_controller
            .delete(config, credentials)
            .await
            .unwrap();

        // ################# Retrieval start #################

        // Master secret
        get_master_secret(dest_wallet_handle).await;

        // Credential
        get_wallet_item!(
            dest_wallet_handle,
            &RecordCategory::Cred.to_string(),
            credx::types::Credential
        );
        get_wallet_item!(
            dest_wallet_handle,
            &RecordCategory::CredDef.to_string(),
            credx::types::CredentialDefinition
        );
        get_wallet_item!(
            dest_wallet_handle,
            &RecordCategory::CredDefPriv.to_string(),
            credx::types::CredentialDefinitionPrivate
        );
        get_wallet_item!(
            dest_wallet_handle,
            &RecordCategory::CredKeyCorrectnessProof.to_string(),
            credx::types::CredentialKeyCorrectnessProof
        );

        // Schema
        get_wallet_item!(
            dest_wallet_handle,
            &RecordCategory::CredSchema.to_string(),
            credx::types::Schema
        );
        get_wallet_item!(
            dest_wallet_handle,
            &RecordCategory::CredMapSchemaId.to_string(),
            credx::types::SchemaId
        );

        // Revocation registry
        get_wallet_item!(
            dest_wallet_handle,
            &RecordCategory::RevReg.to_string(),
            credx::types::RevocationRegistry
        );
        get_wallet_item!(
            dest_wallet_handle,
            &RecordCategory::RevRegDelta.to_string(),
            credx::types::RevocationRegistryDelta
        );
        get_wallet_item!(
            dest_wallet_handle,
            &RecordCategory::RevRegInfo.to_string(),
            RevocationRegistryInfo
        );
        get_wallet_item!(
            dest_wallet_handle,
            &RecordCategory::RevRegDef.to_string(),
            credx::types::RevocationRegistryDefinition
        );
        get_wallet_item!(
            dest_wallet_handle,
            &RecordCategory::RevRegDefPriv.to_string(),
            credx::types::RevocationRegistryDefinitionPrivate
        );

        // ################# Retrieval end #################

        // New wallet cleanup
        Locator::instance()
            .wallet_controller
            .close(dest_wallet_handle)
            .await
            .unwrap();

        Locator::instance()
            .wallet_controller
            .delete(new_config, new_credentials)
            .await
            .unwrap();
    }

    fn make_wallet_reqs(wallet_name: String) -> (Credentials, Config) {
        let credentials = Credentials {
            key: WALLET_KEY.to_owned(),
            key_derivation_method: KeyDerivationMethod::RAW,
            rekey: None,
            rekey_derivation_method: KeyDerivationMethod::ARGON2I_MOD,
            storage_credentials: None,
        };

        let config = Config {
            id: wallet_name,
            storage_type: None,
            storage_config: None,
            cache: None,
        };

        (credentials, config)
    }

    async fn get_wallet_item_raw(wallet_handle: WalletHandle, category: &str) -> String {
        let options = r#"{"retrieveType": true, "retrieveValue": true, "retrieveTags": true}"#;

        let record_str = Locator::instance()
            .non_secret_controller
            .get_record(
                wallet_handle,
                category.to_owned(),
                "test_id".to_owned(),
                options.to_owned(),
            )
            .await
            .unwrap();

        let record: IndyRecord = serde_json::from_str(&record_str).unwrap();
        record.value
    }

    // MasterSecret needs special processing
    async fn get_master_secret(wallet_handle: WalletHandle) {
        let ms_decimal =
            get_wallet_item_raw(wallet_handle, &RecordCategory::LinkSecret.to_string()).await;
        let ms_bn = BigNumber::from_dec(&ms_decimal).unwrap();

        let ursa_ms: ClLinkSecret = serde_json::from_value(json!({ "ms": ms_bn })).unwrap();
        let _ = LinkSecret { value: ursa_ms };
    }

    fn make_dummy_master_secret() -> vdrtools::MasterSecret {
        let ms_str = json!({
            "value": {
                "ms": "1234567890"
            }
        })
        .to_string();

        serde_json::from_str(&ms_str).unwrap()
    }

    fn make_dummy_cred() -> vdrtools::Credential {
        let cred_sig_str = json!({
            "p_credential": {
                "m_2": "1234567890",
                "a": "1234567890",
                "e": "1234567890",
                "v": "1234567890"
            },
            "r_credential": null
        })
        .to_string();

        let sig_cor_proof_str = json!({
            "se": "1234567890",
            "c": "1234567890"
        })
        .to_string();

        vdrtools::Credential {
            schema_id: vdrtools::SchemaId("test_schema_id".to_owned()),
            cred_def_id: vdrtools::CredentialDefinitionId("test_cred_def_id".to_owned()),
            rev_reg_id: Some(vdrtools::RevocationRegistryId("test_rev_reg_id".to_owned())),
            values: vdrtools::CredentialValues(HashMap::new()),
            signature: serde_json::from_str(&cred_sig_str).unwrap(),
            signature_correctness_proof: serde_json::from_str(&sig_cor_proof_str).unwrap(),
            rev_reg: None,
            witness: None,
        }
    }

    fn make_dummy_cred_def() -> vdrtools::CredentialDefinition {
        let primary = json!({
            "n": "1234567890",
            "s": "1234567890",
            "r": {},
            "rctxt": "1234567890",
            "z": "1234567890",
        })
        .to_string();

        vdrtools::CredentialDefinition::CredentialDefinitionV1(vdrtools::CredentialDefinitionV1 {
            id: vdrtools::CredentialDefinitionId("test_cred_def_id".to_owned()),
            schema_id: vdrtools::SchemaId("test_schema_id".to_owned()),
            signature_type: vdrtools::SignatureType::CL,
            tag: "{}".to_owned(),
            value: vdrtools::CredentialDefinitionData {
                primary: serde_json::from_str(&primary).unwrap(),
                revocation: None,
            },
        })
    }

    fn make_dummy_cred_def_priv_key() -> vdrtools::CredentialDefinitionPrivateKey {
        let priv_key = json!({
            "p_key": {
                "p": "1234567890",
                "q": "1234567890"
            }
        })
        .to_string();

        vdrtools::CredentialDefinitionPrivateKey {
            value: serde_json::from_str(&priv_key).unwrap(),
        }
    }

    fn make_dummy_cred_def_correctness_proof() -> vdrtools::CredentialDefinitionCorrectnessProof {
        let cor_proof = json!({
            "c": "1234567890",
            "xz_cap": "1234567890",
            "xr_cap": []
        })
        .to_string();

        vdrtools::CredentialDefinitionCorrectnessProof {
            value: serde_json::from_str(&cor_proof).unwrap(),
        }
    }

    fn make_dummy_schema() -> vdrtools::Schema {
        vdrtools::Schema::SchemaV1(vdrtools::SchemaV1 {
            id: vdrtools::SchemaId("test_schema_id".to_owned()),
            name: "test_schema_name".to_owned(),
            version: "test_schema_version".to_owned(),
            attr_names: vdrtools::AttributeNames(HashSet::new()),
            seq_no: None,
        })
    }

    fn make_dummy_schema_id() -> vdrtools::SchemaId {
        vdrtools::SchemaId("test_schema_id".to_owned())
    }

    fn make_dummy_rev_reg() -> vdrtools::RevocationRegistry {
        let rev_reg = json!({
            "accum": "21 11ED98357F9B9B3077E633D35A72CECEF107F85DA7BBFBF2873E2EE7E0F27D326 21 1371CDA6174D6F01A39157428768D328B4B80088EB14AA0AAB7F046B645E1A235 6 65BBFAC37012790BB8B283F164BE3C0585AB60CD7B72123E4DC43DDA7A6A4E6D 4 3BB64FAF922865095CD5AA4349C0437D04EA30FB7592D932531732F2DCB83DB8 6 77039B80A78AB4A2476373C6F8ECC5E2D94B8F37F924549AFA247E2D6EE86DEE 4 24E94FB6B5233B22BDF47745AA821A1797BC6504BC11D5B825B4F8137F1E307F"
        }).to_string();

        vdrtools::RevocationRegistry::RevocationRegistryV1(vdrtools::RevocationRegistryV1 {
            value: serde_json::from_str(&rev_reg).unwrap(),
        })
    }

    fn make_dummy_rev_reg_delta() -> String {
        let rev_reg = json!({
            "prevAccum": "21 11ED98357F9B9B3077E633D35A72CECEF107F85DA7BBFBF2873E2EE7E0F27D326 21 1371CDA6174D6F01A39157428768D328B4B80088EB14AA0AAB7F046B645E1A235 6 65BBFAC37012790BB8B283F164BE3C0585AB60CD7B72123E4DC43DDA7A6A4E6D 4 3BB64FAF922865095CD5AA4349C0437D04EA30FB7592D932531732F2DCB83DB8 6 77039B80A78AB4A2476373C6F8ECC5E2D94B8F37F924549AFA247E2D6EE86DEE 4 24E94FB6B5233B22BDF47745AA821A1797BC6504BC11D5B825B4F8137F1E307F",
            "accum": "21 11ED98357F9B9B3077E633D35A72CECEF107F85DA7BBFBF2873E2EE7E0F27D326 21 1371CDA6174D6F01A39157428768D328B4B80088EB14AA0AAB7F046B645E1A235 6 65BBFAC37012790BB8B283F164BE3C0585AB60CD7B72123E4DC43DDA7A6A4E6D 4 3BB64FAF922865095CD5AA4349C0437D04EA30FB7592D932531732F2DCB83DB8 6 77039B80A78AB4A2476373C6F8ECC5E2D94B8F37F924549AFA247E2D6EE86DEE 4 24E94FB6B5233B22BDF47745AA821A1797BC6504BC11D5B825B4F8137F1E307F",
            "issued": [],
            "revoked": []
        }).to_string();

        let rev_reg_delta = vdrtools::RevocationRegistryDelta::RevocationRegistryDeltaV1(
            vdrtools::RevocationRegistryDeltaV1 {
                value: serde_json::from_str(&rev_reg).unwrap(),
            },
        );

        // Vdrtools serializes this to String.
        // Sad, I know...
        json!(rev_reg_delta).to_string()
    }

    fn make_dummy_rev_reg_info() -> vdrtools::RevocationRegistryInfo {
        vdrtools::RevocationRegistryInfo {
            id: vdrtools::RevocationRegistryId("test_rev_reg_id".to_owned()),
            curr_id: 1,
            used_ids: HashSet::new(),
        }
    }

    fn make_dummy_rev_reg_def() -> vdrtools::RevocationRegistryDefinition {
        let accum_key = json!({
            "z": "1 042CDA7AA76FFD05D0EA1C97F0F238A579AAE4348442298B7F8513277A21D671 1 04C49DDECC3731B11BC98A1495C39DF7F94A297EA6D691DADAF1493300D2977E 1 0D78B673DE9F1CE37FA98E0765B69D963BFF9973317722981943797EFEF1F628 1 1F4DFD2C1ED2BD80D9D92600AB7A1B2911180B4B44C6BC42962084AC4C042385 1 07724871AD4FFC1C30BCAEFE289FAF6F2F322203C34D8D2D3C36DFD816AF9430 1 050F4014E2AFD680A67C197B39D35CA4D03332D6C6922A4D991EC1402B7FF4E6 1 07C0DCAF303CF4B0741447A1A808C8C2BAE6CD30397AAF834428848FEE70FC3D 1 1C028C08BD426B053942A4409F71A5215B6B0B58FF651C72303F1B4C5DDB84C4 1 22DE20332A0E1B0C58F76CBADBF73D0B6875A5F3479AC0E3C4D27A605656BF6E 1 1F461563E404002F9AFE37D09FA98F34B4666D1A4424C89B3C8CE7E85DE23B8A 1 096DA55063F6ABA1B578471DEBDEACA5DE485994F99099BBBB6E326DDF8C3DD2 1 12FFCEFF31CE5781FF6BB9AB279BF8A100E97D43B0F6C31E6FCD6373227E34FD"
        }).to_string();

        vdrtools::RevocationRegistryDefinition::RevocationRegistryDefinitionV1(
            vdrtools::RevocationRegistryDefinitionV1 {
                id: vdrtools::RevocationRegistryId("test_rev_reg_id".to_owned()),
                revoc_def_type: vdrtools::RegistryType::CL_ACCUM,
                tag: "{}".to_owned(),
                cred_def_id: vdrtools::CredentialDefinitionId("test_cred_def_id".to_owned()),
                value: vdrtools::RevocationRegistryDefinitionValue {
                    issuance_type: vdrtools::IssuanceType::ISSUANCE_BY_DEFAULT,
                    max_cred_num: 10,
                    public_keys: vdrtools::RevocationRegistryDefinitionValuePublicKeys {
                        accum_key: serde_json::from_str(&accum_key).unwrap(),
                    },
                    tails_hash: "abc".to_owned(),
                    tails_location: "/dev/null".to_owned(),
                },
            },
        )
    }

    fn make_dummy_rev_reg_def_priv() -> vdrtools::RevocationRegistryDefinitionPrivate {
        let rev_key_priv = json!({
            "gamma": "12345"
        })
        .to_string();

        vdrtools::RevocationRegistryDefinitionPrivate {
            value: serde_json::from_str(&rev_key_priv).unwrap(),
        }
    }
}
