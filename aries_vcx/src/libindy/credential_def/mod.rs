use indy_sys::WalletHandle;
use std::fmt;

use crate::error::prelude::*;
use crate::libindy::utils::anoncreds;
use crate::utils::constants::DEFAULT_SERIALIZE_VERSION;
use crate::utils::serialization::ObjectWithVersion;

pub mod revocation_registry;

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq, Default)]
pub struct CredentialDef {
    pub cred_def_id: String,
    tag: String,
    source_id: String,
    issuer_did: String,
    cred_def_json: String,
    support_revocation: bool,
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

impl Default for PublicEntityStateType {
    fn default() -> Self {
        PublicEntityStateType::Published
    }
}

async fn _try_get_cred_def_from_ledger(issuer_did: &str, cred_def_id: &str) -> VcxResult<Option<String>> {
    match anoncreds::get_cred_def(Some(issuer_did), cred_def_id).await {
        Ok((_, cred_def)) => Ok(Some(cred_def)),
        Err(err) if err.kind() == VcxErrorKind::LibndyError(309) => Ok(None),
        Err(err) => Err(VcxError::from_msg(
            VcxErrorKind::InvalidLedgerResponse,
            format!(
                "Failed to check presence of credential definition id {} on the ledger\nError: {}",
                cred_def_id, err
            ),
        )),
    }
}

impl CredentialDef {
    pub async fn create(
        wallet_handle: WalletHandle,
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
        let pool_handle = crate::global::pool::get_main_pool_handle()?;
        let (_, schema_json) = anoncreds::get_schema_json(wallet_handle, pool_handle, &schema_id).await?;
        let (cred_def_id, cred_def_json) = anoncreds::generate_cred_def(
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
            cred_def_id,
            cred_def_json,
            issuer_did,
            support_revocation,
            state: PublicEntityStateType::Built,
        })
    }

    pub fn was_published(&self) -> bool {
        self.state == PublicEntityStateType::Published
    }

    pub fn get_support_revocation(&self) -> bool {
        self.support_revocation
    }

    pub async fn publish_cred_def(self, wallet_handle: WalletHandle) -> VcxResult<Self> {
        trace!(
            "publish_cred_def >>> issuer_did: {}, cred_def_id: {}",
            self.issuer_did,
            self.cred_def_id
        );
        if let Some(ledger_cred_def_json) = _try_get_cred_def_from_ledger(&self.issuer_did, &self.cred_def_id).await? {
            return Err(VcxError::from_msg(
                VcxErrorKind::CredDefAlreadyCreated,
                format!(
                    "Credential definition with id {} already exists on the ledger: {}",
                    self.cred_def_id, ledger_cred_def_json
                ),
            ));
        }
        anoncreds::publish_cred_def(wallet_handle, &self.issuer_did, &self.cred_def_json).await?;
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

    pub fn get_source_id(&self) -> &String {
        &self.source_id
    }

    pub fn get_cred_def_id(&self) -> String {
        self.cred_def_id.clone()
    }

    pub fn set_source_id(&mut self, source_id: String) {
        self.source_id = source_id;
    }

    pub async fn update_state(&mut self, wallet_handle: WalletHandle) -> VcxResult<u32> {
        let pool_handle = crate::global::pool::get_main_pool_handle()?;
        if (anoncreds::get_cred_def_json(wallet_handle, pool_handle, &self.cred_def_id).await).is_ok() {
            self.state = PublicEntityStateType::Published
        }
        Ok(self.state as u32)
    }

    pub fn get_state(&self) -> u32 {
        self.state as u32
    }
}
