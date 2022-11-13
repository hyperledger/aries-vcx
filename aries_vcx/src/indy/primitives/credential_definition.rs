use vdrtools_sys::{PoolHandle, WalletHandle};
use crate::error::{VcxError, VcxErrorKind, VcxResult};
use crate::utils::constants::{CRED_DEF_ID, CRED_DEF_JSON, DEFAULT_SERIALIZE_VERSION};
use crate::utils::serialization::ObjectWithVersion;

use std::fmt;
use crate::global::settings;
use crate::indy::ledger::transactions::{
    build_cred_def_request, check_response, get_cred_def,
    get_cred_def_json, get_schema_json, sign_and_submit_to_ledger
};

macro_rules! enum_number {
    ($name:ident { $($variant:ident = $value:expr, )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            $($variant = $value,)*
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: ::serde::Serializer
            {
                // Serialize the enum as a u64.
                serializer.serialize_u64(*self as u64)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: ::serde::Deserializer<'de>
            {
                struct Visitor;

                impl<'de> ::serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("positive integer")
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<$name, E>
                        where E: ::serde::de::Error
                    {
                        // Rust does not come with a simple way of converting a
                        // number to an enum, so use a big `match`.
                        match value {
                            $( $value => Ok($name::$variant), )*
                            _ => Err(E::custom(
                                format!("unknown {} value: {}",
                                stringify!($name), value))),
                        }
                    }
                }

                // Deserialize the enum from a u64.
                deserializer.deserialize_u64(Visitor)
            }
        }
    }
}

enum_number!(PublicEntityStateType
{
    Built = 0,
    Published = 1,
});


#[derive(Clone, Deserialize, Debug, Serialize, PartialEq, Default)]
pub struct CredentialDef {
    #[serde(alias = "cred_def_id")]
    id: String,
    tag: String,
    source_id: String,
    issuer_did: String,
    cred_def_json: String,
    support_revocation: bool,
    #[serde(default)]
    schema_id: String,
    #[serde(default)]
    pub state: PublicEntityStateType,
}

#[derive(Clone, Debug, Deserialize, Serialize, Builder, Default)]
#[builder(setter(into), default)]
pub struct CredentialDefConfig {
    issuer_did: String,
    schema_id: String,
    tag: String,
}

#[derive(Clone, Deserialize, Debug, Serialize, Builder, Default)]
#[builder(setter(into, strip_option), default)]
pub struct RevocationDetails {
    pub support_revocation: Option<bool>,
    pub tails_dir: Option<String>,
    pub max_creds: Option<u32>,
}

impl Default for PublicEntityStateType {
    fn default() -> Self {
        PublicEntityStateType::Published
    }
}

async fn _try_get_cred_def_from_ledger(pool_handle: PoolHandle, issuer_did: &str, id: &str) -> VcxResult<Option<String>> {
    match get_cred_def(pool_handle, Some(issuer_did), id).await {
        Ok((_, cred_def)) => Ok(Some(cred_def)),
        Err(err) if err.kind() == VcxErrorKind::LibndyError(309) => Ok(None),
        Err(err) => Err(VcxError::from_msg(
            VcxErrorKind::InvalidLedgerResponse,
            format!(
                "Failed to check presence of credential definition id {} on the ledger\nError: {}",
                id, err
            ),
        )),
    }
}

impl CredentialDef {
    pub async fn create(
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        source_id: String,
        config: CredentialDefConfig,
        support_revocation: bool,
    ) -> VcxResult<Self> {
        trace!(
            "CredentialDef::create >>> source_id: {}, config: {:?}",
            source_id,
            config
        );
        let CredentialDefConfig {
            issuer_did,
            schema_id,
            tag,
        } = config;
        let (_, schema_json) = get_schema_json(wallet_handle, pool_handle, &schema_id).await?;
        let (cred_def_id, cred_def_json) = generate_cred_def(
            wallet_handle,
            &issuer_did,
            &schema_json,
            &tag,
            None,
            Some(support_revocation),
        )
        .await?;
        Ok(Self {
            source_id,
            tag,
            id: cred_def_id,
            cred_def_json,
            issuer_did,
            support_revocation,
            schema_id,
            state: PublicEntityStateType::Built,
        })
    }

    pub fn was_published(&self) -> bool {
        self.state == PublicEntityStateType::Published
    }

    pub fn get_support_revocation(&self) -> bool {
        self.support_revocation
    }

    pub async fn publish_cred_def(self, wallet_handle: WalletHandle, pool_handle: PoolHandle) -> VcxResult<Self> {
        trace!(
            "publish_cred_def >>> issuer_did: {}, cred_def_id: {}",
            self.issuer_did,
            self.id
        );
        if let Some(ledger_cred_def_json) = _try_get_cred_def_from_ledger(pool_handle, &self.issuer_did, &self.id).await? {
            return Err(VcxError::from_msg(
                VcxErrorKind::CredDefAlreadyCreated,
                format!(
                    "Credential definition with id {} already exists on the ledger: {}",
                    self.id, ledger_cred_def_json
                ),
            ));
        }
        publish_cred_def(wallet_handle, pool_handle, &self.issuer_did, &self.cred_def_json).await?;
        Ok(Self {
            state: PublicEntityStateType::Published,
            ..self
        })
    }

    pub fn from_string(data: &str) -> VcxResult<Self> {
        ObjectWithVersion::deserialize(data)
            .map(|obj: ObjectWithVersion<Self>| obj.data)
            .map_err(|err| {
                VcxError::from_msg(
                    VcxErrorKind::CreateCredDef,
                    format!("Cannot deserialize CredentialDefinition: {}", err),
                )
            })
    }

    pub fn to_string(&self) -> VcxResult<String> {
        ObjectWithVersion::new(DEFAULT_SERIALIZE_VERSION, self.to_owned())
            .serialize()
            .map_err(|err| err)
            .map_err(|err: VcxError| err.extend("Cannot serialize CredentialDefinition"))
    }

    pub fn get_data_json(&self) -> VcxResult<String> {
        serde_json::to_string(&self)
            .map_err(|_| VcxError::from_msg(VcxErrorKind::SerializationError, "Failed to serialize credential definition"))
    }

    pub fn get_source_id(&self) -> &String {
        &self.source_id
    }

    pub fn get_cred_def_id(&self) -> String {
        self.id.clone()
    }

    pub fn get_schema_id(&self) -> String {
        self.schema_id.clone()
    }

    pub fn set_source_id(&mut self, source_id: String) {
        self.source_id = source_id;
    }

    pub async fn update_state(&mut self, wallet_handle: WalletHandle, pool_handle: PoolHandle) -> VcxResult<u32> {
        if (get_cred_def_json(wallet_handle, pool_handle, &self.id).await).is_ok() {
            self.state = PublicEntityStateType::Published
        }
        Ok(self.state as u32)
    }

    pub fn get_state(&self) -> u32 {
        self.state as u32
    }
}

pub async fn publish_cred_def(wallet_handle: WalletHandle, pool_handle: PoolHandle, issuer_did: &str, cred_def_json: &str) -> VcxResult<()> {
    trace!(
        "publish_cred_def >>> issuer_did: {}, cred_def_json: {}",
        issuer_did,
        cred_def_json
    );
    if settings::indy_mocks_enabled() {
        debug!("publish_cred_def >>> mocked success");
        return Ok(());
    }
    let cred_def_req = build_cred_def_request(issuer_did, cred_def_json).await?;
    let response = sign_and_submit_to_ledger(wallet_handle, pool_handle, issuer_did, &cred_def_req).await?;
    check_response(&response)
}

pub async fn libindy_create_and_store_credential_def(
    wallet_handle: WalletHandle,
    issuer_did: &str,
    schema_json: &str,
    tag: &str,
    sig_type: Option<&str>,
    config_json: &str,
) -> VcxResult<(String, String)> {
    vdrtools::anoncreds::issuer_create_and_store_credential_def(
        wallet_handle,
        issuer_did,
        schema_json,
        tag,
        sig_type,
        config_json,
    )
        .await
        .map_err(VcxError::from)
}

pub async fn generate_cred_def(
    wallet_handle: WalletHandle,
    issuer_did: &str,
    schema_json: &str,
    tag: &str,
    sig_type: Option<&str>,
    support_revocation: Option<bool>,
) -> VcxResult<(String, String)> {
    trace!(
        "generate_cred_def >>> issuer_did: {}, schema_json: {}, tag: {}, sig_type: {:?}, support_revocation: {:?}",
        issuer_did,
        schema_json,
        tag,
        sig_type,
        support_revocation
    );
    if settings::indy_mocks_enabled() {
        return Ok((CRED_DEF_ID.to_string(), CRED_DEF_JSON.to_string()));
    }

    let config_json = json!({"support_revocation": support_revocation.unwrap_or(false)}).to_string();

    libindy_create_and_store_credential_def(wallet_handle, issuer_did, schema_json, tag, sig_type, &config_json).await
}


#[cfg(test)]
#[cfg(feature = "pool_tests")]
pub mod integration_tests {
    use crate::indy::test_utils::create_and_write_test_schema;
    use crate::indy::primitives::credential_definition::generate_cred_def;
    use crate::indy::ledger::transactions::get_schema_json;
    use crate::indy::primitives::credential_definition::publish_cred_def;
    use crate::indy::primitives::revocation_registry::{generate_rev_reg, publish_rev_reg_def, publish_rev_reg_delta};
    use crate::utils::constants::DEFAULT_SCHEMA_ATTRS;
    use crate::utils::devsetup::SetupWalletPool;

    #[tokio::test]
    async fn test_create_cred_def_real() {
        let setup = SetupWalletPool::init().await;

        let (schema_id, _) =
            create_and_write_test_schema(
                setup.wallet_handle,
                setup.pool_handle,
                &setup.institution_did,
                DEFAULT_SCHEMA_ATTRS)
            .await;

        let (_, schema_json) =
            get_schema_json(
                setup.wallet_handle,
                setup.pool_handle,
                &schema_id)
            .await
            .unwrap();

        let (_, cred_def_json) =
            generate_cred_def(
                setup.wallet_handle,
                &setup.institution_did,
                &schema_json,
                "tag_1", None, Some(true))
            .await
            .unwrap();

        publish_cred_def(
            setup.wallet_handle,
            setup.pool_handle,
            &setup.institution_did,
            &cred_def_json)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_create_rev_reg_def() {
        let setup = SetupWalletPool::init().await;

        let (schema_id, _) =
            create_and_write_test_schema(setup.wallet_handle, setup.pool_handle, &setup.institution_did, DEFAULT_SCHEMA_ATTRS).await;
        let (_, schema_json) = get_schema_json(setup.wallet_handle, setup.pool_handle, &schema_id).await.unwrap();

        let (cred_def_id, cred_def_json) =
            generate_cred_def(setup.wallet_handle, &setup.institution_did, &schema_json, "tag_1", None, Some(true))
                .await
                .unwrap();
        publish_cred_def(setup.wallet_handle, setup.pool_handle, &setup.institution_did, &cred_def_json)
            .await
            .unwrap();
        let (rev_reg_def_id, rev_reg_def_json, rev_reg_entry_json) =
            generate_rev_reg(setup.wallet_handle, &setup.institution_did, &cred_def_id, "tails.txt", 2, "tag1")
                .await
                .unwrap();
        publish_rev_reg_def(setup.wallet_handle, setup.pool_handle, &setup.institution_did, &rev_reg_def_json)
            .await
            .unwrap();
        publish_rev_reg_delta(setup.wallet_handle, setup.pool_handle, &setup.institution_did, &rev_reg_def_id, &rev_reg_entry_json)
            .await
            .unwrap();
    }
}
