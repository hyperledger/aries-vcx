use std::fmt;

use serde_json;

use crate::error::prelude::*;
use crate::libindy::credential_def::revocation_registry::RevocationRegistry;
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
    rev_reg: Option<RevocationRegistry>,
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
        Err(err) => Err(VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("Failed to check presence of credential definition id {} on the ledger\nError: {}", cred_def_id, err)))
    }
}

impl CredentialDef {
    pub async fn create(source_id: String, config: CredentialDefConfig, support_revocation: bool) -> VcxResult<Self>{
        trace!("CredentialDef::create >>> source_id: {}, config: {:?}", source_id, config);
        let CredentialDefConfig { issuer_did, schema_id, tag } = config;
        let (_, schema_json) = anoncreds::get_schema_json(&schema_id).await?;
        let (cred_def_id, cred_def_json) = anoncreds::generate_cred_def(&issuer_did,
                                                                        &schema_json,
                                                                        &tag,
                                                                        None,
                                                                        Some(support_revocation)).await?;
        Ok(
            Self {
                source_id,
                tag,
                cred_def_id,
                cred_def_json,
                issuer_did,
                rev_reg: None,
                support_revocation,
                state: PublicEntityStateType::Built,
            }
        )
    }

    pub async fn create_and_store(source_id: String, config: CredentialDefConfig, revocation_details: RevocationDetails) -> VcxResult<Self> {
        // unimplemented!("Use create()+publish_cred_def() instead")
        trace!("CredentialDef::create_and_store >>> source_id: {}, config: {:?}, revocation_details: {:?}", source_id, config, revocation_details);
        let CredentialDefConfig { issuer_did, schema_id, tag } = config;
        let (_, schema_json) = anoncreds::get_schema_json(&schema_id).await?;
        let (cred_def_id, cred_def_json) = anoncreds::generate_cred_def(&issuer_did,
                                                                        &schema_json,
                                                                        &tag,
                                                                        None,
                                                                        revocation_details.support_revocation.clone()).await?;

        let rev_reg = if revocation_details.support_revocation.unwrap_or(false) {
            let tails_dir = revocation_details
                .tails_dir
                .ok_or(VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails, "Invalid RevocationDetails: `tails_dir` field not found"))?;

            let max_creds = revocation_details
                .max_creds
                .ok_or(VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails, "Invalid RevocationDetails: `max_creds` field not found"))?;

            Some(RevocationRegistry::create(&issuer_did, &cred_def_id, &tails_dir, max_creds, 1).await?)
        } else {
            None
        };
        Ok(
            Self {
                source_id,
                tag,
                cred_def_id,
                cred_def_json,
                issuer_did,
                support_revocation: revocation_details.support_revocation.unwrap_or(false),
                rev_reg,
                state: PublicEntityStateType::Built,
            }
        )
    }

    pub fn was_published(&self) -> bool {
        self.state == PublicEntityStateType::Published
    }

    pub fn get_support_revocation(&self) -> bool {
        self.support_revocation
    }

    pub async fn publish_cred_def(self) -> VcxResult<Self> {
        trace!("publish_cred_def >>> issuer_did: {}, cred_def_id: {}", self.issuer_did, self.cred_def_id);
        if let Some(ledger_cred_def_json) = _try_get_cred_def_from_ledger(&self.issuer_did, &self.cred_def_id).await? {
            return Err(VcxError::from_msg(VcxErrorKind::CredDefAlreadyCreated, format!("Credential definition with id {} already exists on the ledger: {}", self.cred_def_id, ledger_cred_def_json)));
        }
        anoncreds::publish_cred_def(&self.issuer_did, &self.cred_def_json).await?;
        Ok(
            Self {
                state: PublicEntityStateType::Published,
                ..self
            }
        )
    }

    pub async fn rotate_rev_reg(&mut self, revocation_details: RevocationDetails) -> VcxResult<()> {
        // unimplemented!("Just create a new revocation registry bro")
        trace!("CredentialDef::rotate_rev_reg >>> revocation_details: {:?}", revocation_details);
        let (tails_dir, max_creds) = (
            revocation_details.tails_dir.or(self.get_tails_dir()),
            revocation_details.max_creds.or(self.get_max_creds())
        );

        self.rev_reg = match (&self.rev_reg, &tails_dir, &max_creds) {
            (Some(rev_reg), Some(tails_dir), Some(max_creds)) => {
                let tag = rev_reg.tag + 1;
                Some(RevocationRegistry::create(&self.issuer_did, &self.cred_def_id, tails_dir, max_creds.clone(), tag).await?)
            }
            _ => return {
                Err(VcxError::from_msg(VcxErrorKind::RevRegDefNotFound,
                                       "No revocation registry definitions associated with this credential definition",
                ))
            }
        };
        trace!("rotate_rev_reg_def <<< new_rev_reg_def: {:?}", self.rev_reg);
        Ok(())
    }

    pub async fn publish_revocation_primitives(&mut self, tails_url: &str) -> VcxResult<()> {
        // unimplemented!("Just create and publish new revocation registry bro")
        warn!("publish_revocation_primitives >>> tails_url: {}", tails_url);
        match &mut self.rev_reg {
            Some(rev_reg) => rev_reg.publish_revocation_primitives(tails_url).await,
            None => {
                Err(VcxError::from_msg(VcxErrorKind::NotReady,
                                       "Tried to publish revocation primitives, but this credential definition is not revocable",
                ))
            }
        }
    }

    pub fn has_pending_revocations_primitives_to_be_published(&self) -> bool {
        match &self.rev_reg {
            None => false,
            Some(rev_reg) => {
                !rev_reg.was_rev_reg_def_published() || !rev_reg.was_rev_reg_delta_published()
            }
        }
    }

    pub fn from_string(data: &str) -> VcxResult<Self> {
        ObjectWithVersion::deserialize(data)
            .map(|obj: ObjectWithVersion<Self>| obj.data)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::CreateCredDef, format!("Cannot deserialize CredentialDefinition: {}", err)))
    }

    pub fn to_string(&self) -> VcxResult<String> {
        ObjectWithVersion::new(DEFAULT_SERIALIZE_VERSION, self.to_owned())
            .serialize()
            .map_err(|err| err.into())
            .map_err(|err: VcxError| err.extend("Cannot serialize CredentialDefinition"))
    }

    pub fn get_source_id(&self) -> &String { &self.source_id }

    pub fn get_rev_reg_id(&self) -> Option<String> {
        match &self.rev_reg {
            Some(rev_reg) => Some(rev_reg.rev_reg_id.clone()),
            None => None
        }
    }

    pub fn get_tails_dir(&self) -> Option<String> {
        match &self.rev_reg {
            Some(rev_reg) => Some(rev_reg.tails_dir.clone()),
            None => None
        }
    }

    pub fn get_max_creds(&self) -> Option<u32> {
        match &self.rev_reg {
            Some(rev_reg) => Some(rev_reg.max_creds.clone()),
            None => None
        }
    }

    pub fn get_rev_reg_def(&self) -> VcxResult<Option<String>> {
        match &self.rev_reg {
            Some(rev_reg) => {
                let rev_reg_def_json = serde_json::to_string(&rev_reg.rev_reg_def)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to serialize rev_reg_def: {:?}, error: {:?}", rev_reg.rev_reg_def, err)))?;
                Ok(Some(rev_reg_def_json))
            }
            None => Ok(None)
        }
    }

    pub fn get_cred_def_id(&self) -> String { self.cred_def_id.clone() }

    pub fn set_source_id(&mut self, source_id: String) { self.source_id = source_id.clone(); }

    pub async fn update_state(&mut self) -> VcxResult<u32> {
        if let Some(ref rev_reg_id) = self.get_rev_reg_id() {
            if let (Ok(_), Ok(_), Ok(_)) = (anoncreds::get_cred_def_json(&self.cred_def_id).await,
                                            anoncreds::get_rev_reg_def_json(rev_reg_id).await,
                                            anoncreds::get_rev_reg(rev_reg_id, time::get_time().sec as u64).await) {
                self.state = PublicEntityStateType::Published
            }
        } else {
            if let Ok(_) = anoncreds::get_cred_def_json(&self.cred_def_id).await {
                self.state = PublicEntityStateType::Published
            }
        }

        Ok(self.state as u32)
    }

    pub fn get_state(&self) -> u32 { self.state as u32 }
}
