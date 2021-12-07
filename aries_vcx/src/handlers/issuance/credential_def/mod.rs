use std::fmt;

use serde_json;

use crate::error::prelude::*;
use crate::libindy::utils::anoncreds;
use crate::libindy::utils::anoncreds::RevocationRegistryDefinition;
use crate::libindy::utils::payments::PaymentTxn;
use crate::utils::constants::DEFAULT_SERIALIZE_VERSION;
use crate::utils::serialization::ObjectWithVersion;

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq)]
pub struct RevocationRegistry {
    pub rev_reg_id: String,
    rev_reg_def: RevocationRegistryDefinition,
    rev_reg_entry: String,
    tails_file: String,
    max_creds: u32,
    tag: u32,
    rev_reg_def_payment_txn: Option<PaymentTxn>,
    rev_reg_delta_payment_txn: Option<PaymentTxn>,
}

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq, Default)]
pub struct CredentialDef {
    pub cred_def_id: String,
    tag: String,
    source_id: String,
    issuer_did: String,
    cred_def_json: String,
    cred_def_payment_txn: Option<PaymentTxn>,
    rev_reg: Option<RevocationRegistry>,
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
    pub tails_file: Option<String>,
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

fn _create_and_store(config: &CredentialDefConfig,
                     revocation_details: &RevocationDetails) -> VcxResult<(String, String, Option<String>, Option<RevocationRegistryDefinition>, Option<String>)> {
    let CredentialDefConfig { issuer_did, schema_id, tag } = config;

    let (_, schema_json) = anoncreds::get_schema_json(&schema_id)?;

    let (cred_def_id, cred_def_json) = anoncreds::generate_cred_def(&issuer_did,
                                                                    &schema_json,
                                                                    &tag,
                                                                    None,
                                                                    revocation_details.support_revocation)?;

    let (rev_reg_id, rev_reg_def, rev_reg_entry) = match revocation_details.support_revocation {
        Some(true) => {
            let tails_file = revocation_details
                .tails_file
                .as_ref()
                .ok_or(VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails, "Invalid RevocationDetails: `tails_file` field not found"))?;

            let max_creds = revocation_details
                .max_creds
                .ok_or(VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails, "Invalid RevocationDetails: `max_creds` field not found"))?;

            let (rev_reg_id, rev_reg_def, rev_reg_entry) =
                anoncreds::generate_rev_reg(&issuer_did, &cred_def_id, &tails_file, max_creds, "tag1")
                    .map_err(|err| err.map(VcxErrorKind::CreateCredDef, "Cannot create CredentialDefinition"))?;

            (Some(rev_reg_id), Some(rev_reg_def), Some(rev_reg_entry))
        }
        _ => (None, None, None),
    };

    Ok((cred_def_id, cred_def_json, rev_reg_id, rev_reg_def, rev_reg_entry))
}

fn _try_get_cred_def_from_ledger(issuer_did: &str, cred_def_id: &str) -> VcxResult<Option<String>> {
    match anoncreds::get_cred_def(Some(issuer_did), cred_def_id) {
        Ok((_, cred_def)) => Ok(Some(cred_def)),
        Err(err) if err.kind() == VcxErrorKind::LibndyError(309) => Ok(None),
        Err(err) => Err(VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("Failed to check presence of credential definition id {} on the ledger\nError: {}", cred_def_id, err)))
    }
}

// fn _maybe_set_url(mut rev_reg_def: RevocationRegistryDefinition, revocation_details: &RevocationDetails) -> VcxResult<RevocationRegistryDefinition> {
//     if let Some(tails_url) = &revocation_details.tails_url {
//         rev_reg_def.value.tails_location = tails_url.to_string();
//     } else if let Some(tails_base_url) = &revocation_details.tails_base_url {
//         rev_reg_def.value.tails_location = vec![tails_base_url.to_string(), rev_reg_def.value.tails_hash.to_owned()].join("/")
//     }
//     Ok(rev_reg_def)
// }

pub fn parse_revocation_details(revocation_details: &str) -> VcxResult<RevocationDetails> {
    serde_json::from_str::<RevocationDetails>(&revocation_details)
        .to_vcx(VcxErrorKind::InvalidRevocationDetails, "Cannot deserialize RevocationDetails")
}

impl CredentialDef {
    pub fn create_and_store(source_id: String, config: CredentialDefConfig, revocation_details: RevocationDetails) -> VcxResult<Self> {
        trace!("CredentialDef::create_and_store >>> source_id: {}, config: {:?}, revocation_details: {:?}", source_id, config, revocation_details);

        let (cred_def_id, cred_def_json, rev_reg_id, rev_reg_def, rev_reg_entry) = _create_and_store(&config, &revocation_details)?;

        let CredentialDefConfig { issuer_did, tag, .. } = config;

        let rev_reg = match (rev_reg_id, rev_reg_def, rev_reg_entry, revocation_details.tails_file, revocation_details.max_creds) {
            (Some(rev_reg_id), Some(rev_reg_def), Some(rev_reg_entry), Some(tails_file), Some(max_creds)) => {
                Some(RevocationRegistry {
                    rev_reg_id,
                    rev_reg_def,
                    rev_reg_entry,
                    tails_file,
                    max_creds,
                    tag: 1,
                    rev_reg_def_payment_txn: None,
                    rev_reg_delta_payment_txn: None,
                })
            }
            _ => None
        };

        Ok(
            Self {
                source_id,
                tag,
                cred_def_id,
                cred_def_json,
                issuer_did,
                cred_def_payment_txn: None,
                rev_reg,
                state: PublicEntityStateType::Built,
            }
        )
    }

    pub fn publish(mut self, tails_url: Option<&str>) -> VcxResult<Self> {
        trace!("CredentialDef::publish >>>");

        let (rev_reg_def_payment_txn, rev_reg_delta_payment_txn, cred_def_payment_txn) = match _try_get_cred_def_from_ledger(&self.issuer_did, &self.cred_def_id) {
            Ok(Some(ledger_cred_def_json)) => {
                return Err(VcxError::from_msg(VcxErrorKind::CreateCredDef, format!("Credential definition with id {} already exists on the ledger: {}", self.cred_def_id, ledger_cred_def_json)));
            }
            Ok(None) => {
                let cred_def_payment_txn = anoncreds::publish_cred_def(&self.issuer_did, &self.cred_def_json)?;

                match self.rev_reg {
                    Some(ref mut rev_reg) => {
                        if let Some(tails_url) = tails_url {
                            rev_reg.rev_reg_def.value.tails_location = String::from(tails_url);
                        };

                        let rev_def_payment = anoncreds::publish_rev_reg_def(&self.issuer_did, &rev_reg.rev_reg_def)
                            .map_err(|err| err.map(VcxErrorKind::CreateCredDef, "Cannot create CredentialDefinition"))?;

                        let (rev_delta_payment, _) = anoncreds::publish_rev_reg_delta(&self.issuer_did, &rev_reg.rev_reg_id, &rev_reg.rev_reg_entry)
                            .map_err(|err| err.map(VcxErrorKind::InvalidRevocationEntry, "Cannot post RevocationEntry"))?;

                        (rev_def_payment, rev_delta_payment, cred_def_payment_txn)
                    }
                    _ => (None, None, None)
                }
            }
            Err(err) => return Err(err)
        };
        let rev_reg = match self.rev_reg {
            Some(rev_reg) => {
                Some(RevocationRegistry {
                    rev_reg_def_payment_txn,
                    rev_reg_delta_payment_txn,
                    ..rev_reg
                })
            }
            _ => None
        };

        Ok(
            Self {
                cred_def_payment_txn,
                rev_reg,
                state: PublicEntityStateType::Published,
                ..self
            }
        )
    }

    pub fn create(source_id: String, config: CredentialDefConfig, revocation_details: RevocationDetails, tails_url: Option<&str>) -> VcxResult<Self> {
        trace!("CredentialDef::create >>> source_id: {}, config: {:?}, revocation_details: {:?}", source_id, config, revocation_details);
        Self::create_and_store(source_id, config, revocation_details)?.publish(tails_url)
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

    pub fn get_tails_file(&self) -> Option<String> {
        match &self.rev_reg {
            Some(rev_reg) => Some(rev_reg.tails_file.clone()),
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

    pub fn get_rev_reg_def_payment_txn(&self) -> Option<PaymentTxn> {
        match &self.rev_reg {
            Some(rev_reg) => rev_reg.rev_reg_def_payment_txn.clone(),
            None => None
        }
        // self.rev_reg_def_payment_txn.clone();
    }

    pub fn get_rev_reg_delta_payment_txn(&self) -> Option<PaymentTxn> {
        match &self.rev_reg {
            Some(rev_reg) => rev_reg.rev_reg_delta_payment_txn.clone(),
            None => None
        }
        // self.rev_reg_delta_payment_txn.clone();
    }

    pub fn update_state(&mut self) -> VcxResult<u32> {
        if let Some(ref rev_reg_id) = self.get_rev_reg_id() {
            if let (Ok(_), Ok(_), Ok(_)) = (anoncreds::get_cred_def_json(&self.cred_def_id),
                                            anoncreds::get_rev_reg_def_json(rev_reg_id),
                                            anoncreds::get_rev_reg(rev_reg_id, time::get_time().sec as u64)) {
                self.state = PublicEntityStateType::Published
            }
        } else {
            if let Ok(_) = anoncreds::get_cred_def_json(&self.cred_def_id) {
                self.state = PublicEntityStateType::Published
            }
        }

        Ok(self.state as u32)
    }

    pub fn get_state(&self) -> u32 { self.state as u32 }

    pub fn rotate_rev_reg(&mut self, revocation_details: &str, new_tails_url: Option<&str>) -> VcxResult<RevocationRegistry> {
        debug!("CredentialDef::rotate_rev_reg >>> revocation_details: {}", revocation_details);
        let revocation_details = parse_revocation_details(revocation_details)?;
        let (tails_file, max_creds) = (
            revocation_details.clone().tails_file.or(self.get_tails_file()),
            revocation_details.max_creds.or(self.get_max_creds())
        );
        match (&mut self.rev_reg, &tails_file, &max_creds) {
            (Some(rev_reg), Some(tails_file), Some(max_creds)) => {
                let tag = format!("tag{}", rev_reg.tag + 1);
                let (rev_reg_id, mut rev_reg_def, rev_reg_entry) =
                    anoncreds::generate_rev_reg(&self.issuer_did, &self.cred_def_id, &tails_file, *max_creds, tag.as_str())
                        .map_err(|err| err.map(VcxErrorKind::CreateRevRegDef, "Cannot create revocation registry defintion"))?;

                if let Some(new_tails_url) = new_tails_url {
                    rev_reg_def.value.tails_location = String::from(new_tails_url);
                };

                let rev_reg_def_payment_txn = anoncreds::publish_rev_reg_def(&self.issuer_did, &rev_reg_def)
                    .map_err(|err| err.map(VcxErrorKind::CreateCredDef, "Cannot publish revocation registry defintion"))?;

                let (rev_reg_delta_payment_txn, _) = anoncreds::publish_rev_reg_delta(&self.issuer_did, &rev_reg_id, &rev_reg_entry)
                    .map_err(|err| err.map(VcxErrorKind::InvalidRevocationEntry, "Cannot post RevocationEntry"))?;

                let new_rev_reg = RevocationRegistry {
                    rev_reg_id,
                    rev_reg_def,
                    rev_reg_entry,
                    tails_file: tails_file.to_string(),
                    max_creds: *max_creds,
                    tag: rev_reg.tag + 1,
                    rev_reg_delta_payment_txn,
                    rev_reg_def_payment_txn,
                };
                self.rev_reg = Some(new_rev_reg.clone());

                trace!("rotate_rev_reg_def <<< new_rev_reg_def: {:?}", new_rev_reg);
                Ok(new_rev_reg)
            }
            _ => Err(VcxError::from_msg(VcxErrorKind::RevRegDefNotFound, "No revocation registry definitions associated with this credential definition"))
        }
    }
}
