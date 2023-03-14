use std::{fmt, sync::Arc};

use crate::{
    core::profile::profile::Profile,
    errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult},
    global::settings::{self, indy_mocks_enabled},
    indy::utils::LibindyMock,
    plugins::ledger::base_ledger::BaseLedger,
    utils::{
        constants::{CRED_DEF_ID, CRED_DEF_JSON, DEFAULT_SERIALIZE_VERSION},
        serialization::ObjectWithVersion,
    },
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

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq, Eq, Default)]
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

async fn _try_get_cred_def_from_ledger(
    ledger: &Arc<dyn BaseLedger>,
    issuer_did: &str,
    cred_def_id: &str,
) -> VcxResult<Option<String>> {
    // TODO - future - may require more customized logic. We set the rc to 309, as the mock for
    // ledger.get_cred_def will return a valid mock cred def unless it reads an rc of 309. Returning
    // a valid mock cred def will result in this method returning an error.
    if indy_mocks_enabled() {
        LibindyMock::set_next_result(309)
    }
    match ledger.get_cred_def(cred_def_id, Some(issuer_did)).await {
        Ok(cred_def) => Ok(Some(cred_def)),
        Err(err) if err.kind() == AriesVcxErrorKind::LedgerItemNotFound => Ok(None),
        Err(err) => Err(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidLedgerResponse,
            format!(
                "Failed to check presence of credential definition id {} on the ledger\nError: {}",
                cred_def_id, err
            ),
        )),
    }
}
impl CredentialDef {
    pub async fn create(
        profile: &Arc<dyn Profile>,
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
        let ledger = Arc::clone(profile).inject_ledger();
        let schema_json = ledger.get_schema(&schema_id, Some(&issuer_did)).await?;
        let (cred_def_id, cred_def_json) =
            generate_cred_def(profile, &issuer_did, &schema_json, &tag, None, Some(support_revocation)).await?;
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

    pub async fn publish_cred_def(self, profile: &Arc<dyn Profile>) -> VcxResult<Self> {
        trace!(
            "publish_cred_def >>> issuer_did: {}, cred_def_id: {}",
            self.issuer_did,
            self.id
        );
        let ledger = Arc::clone(profile).inject_ledger();
        if let Some(ledger_cred_def_json) = _try_get_cred_def_from_ledger(&ledger, &self.issuer_did, &self.id).await? {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::CredDefAlreadyCreated,
                format!(
                    "Credential definition with id {} already exists on the ledger: {}",
                    self.id, ledger_cred_def_json
                ),
            ));
        }
        ledger.publish_cred_def(&self.cred_def_json, &self.issuer_did).await?;
        Ok(Self {
            state: PublicEntityStateType::Published,
            ..self
        })
    }

    pub fn from_string(data: &str) -> VcxResult<Self> {
        ObjectWithVersion::deserialize(data)
            .map(|obj: ObjectWithVersion<Self>| obj.data)
            .map_err(|err| {
                AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidJson,
                    format!("Cannot deserialize CredentialDefinition: {}", err),
                )
            })
    }

    pub fn to_string(&self) -> VcxResult<String> {
        ObjectWithVersion::new(DEFAULT_SERIALIZE_VERSION, self.to_owned())
            .serialize()
            .map_err(|err: AriesVcxError| err.extend("Cannot serialize CredentialDefinition"))
    }

    pub fn get_data_json(&self) -> VcxResult<String> {
        serde_json::to_string(&self).map_err(|_| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::SerializationError,
                "Failed to serialize credential definition",
            )
        })
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

    pub async fn update_state(&mut self, profile: &Arc<dyn Profile>) -> VcxResult<u32> {
        let ledger = Arc::clone(profile).inject_ledger();
        if (ledger.get_cred_def(&self.id, None).await).is_ok() {
            self.state = PublicEntityStateType::Published
        }
        Ok(self.state as u32)
    }

    pub fn get_state(&self) -> u32 {
        self.state as u32
    }
}

pub async fn generate_cred_def(
    profile: &Arc<dyn Profile>,
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

    let anoncreds = Arc::clone(profile).inject_anoncreds();
    anoncreds
        .issuer_create_and_store_credential_def(issuer_did, schema_json, tag, sig_type, &config_json)
        .await
}

#[cfg(test)]
#[cfg(feature = "pool_tests")]
pub mod integration_tests {
    use std::sync::Arc;

    use crate::{
        common::{
            primitives::{credential_definition::generate_cred_def, revocation_registry::generate_rev_reg},
            test_utils::create_and_write_test_schema,
        },
        utils::{constants::DEFAULT_SCHEMA_ATTRS, devsetup::SetupProfile},
    };

    #[tokio::test]
    async fn test_create_cred_def_real() {
        SetupProfile::run_indy(|setup| async move {
            let (schema_id, _) =
                create_and_write_test_schema(&setup.profile, &setup.institution_did, DEFAULT_SCHEMA_ATTRS).await;

            let ledger = Arc::clone(&setup.profile).inject_ledger();
            let schema_json = ledger.get_schema(&schema_id, None).await.unwrap();

            let (_, cred_def_json) = generate_cred_def(
                &setup.profile,
                &setup.institution_did,
                &schema_json,
                "tag_1",
                None,
                Some(true),
            )
            .await
            .unwrap();

            ledger
                .publish_cred_def(&cred_def_json, &setup.institution_did)
                .await
                .unwrap();
        })
        .await;
    }

    #[tokio::test]
    async fn test_create_rev_reg_def() {
        SetupProfile::run_indy(|setup| async move {
            let (schema_id, _) =
                create_and_write_test_schema(&setup.profile, &setup.institution_did, DEFAULT_SCHEMA_ATTRS).await;
            let ledger = Arc::clone(&setup.profile).inject_ledger();
            let schema_json = ledger.get_schema(&schema_id, None).await.unwrap();

            let (cred_def_id, cred_def_json) = generate_cred_def(
                &setup.profile,
                &setup.institution_did,
                &schema_json,
                "tag_1",
                None,
                Some(true),
            )
            .await
            .unwrap();
            ledger
                .publish_cred_def(&cred_def_json, &setup.institution_did)
                .await
                .unwrap();

            let (rev_reg_def_id, rev_reg_def_json, rev_reg_entry_json) = generate_rev_reg(
                &setup.profile,
                &setup.institution_did,
                &cred_def_id,
                "tails.txt",
                2,
                "tag1",
            )
            .await
            .unwrap();
            ledger
                .publish_rev_reg_def(&rev_reg_def_json, &setup.institution_did)
                .await
                .unwrap();
            ledger
                .publish_rev_reg_delta(&rev_reg_def_id, &rev_reg_entry_json, &setup.institution_did)
                .await
                .unwrap();
        })
        .await;
    }
}
