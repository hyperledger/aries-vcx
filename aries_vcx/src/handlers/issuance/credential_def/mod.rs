use serde_json;
use std::fmt;

use crate::error::prelude::*;
use crate::libindy::utils::{anoncreds, ledger};
use crate::libindy::utils::cache::update_rev_reg_ids_cache;
use crate::libindy::utils::payments::PaymentTxn;
use crate::utils::constants::DEFAULT_SERIALIZE_VERSION;
use crate::utils::serialization::ObjectWithVersion;

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq)]
pub struct RevocationRegistry {
    pub rev_reg_id: String,
    rev_reg_def: String,
    rev_reg_entry: String,
    tails_file: String,
    max_creds: u32,
    tag: u32,
    rev_reg_def_payment_txn: Option<PaymentTxn>,
    rev_reg_delta_payment_txn: Option<PaymentTxn>,
}

#[derive(Default, Clone, Deserialize, Debug, Serialize, PartialEq)]
pub struct CredentialDef {
    pub id: String,
    tag: String,
    name: String,
    source_id: String,
    pub issuer_did: Option<String>,
    cred_def_payment_txn: Option<PaymentTxn>,
    rev_reg: Option<RevocationRegistry>,
    #[serde(default)]
    pub state: PublicEntityStateType,
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct RevocationDetails {
    pub support_revocation: Option<bool>,
    pub tails_file: Option<String>,
    pub tails_url: Option<String>,
    pub tails_base_url: Option<String>,
    pub max_creds: Option<u32>,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinitionValue {
    pub issuance_type: String,
    pub max_cred_num: u32,
    pub public_keys: serde_json::Value,
    pub tails_hash: String,
    pub tails_location: String,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinition {
    pub id: String,
    pub revoc_def_type: String,
    pub tag: String,
    pub cred_def_id: String,
    pub value: RevocationRegistryDefinitionValue,
    pub ver: String,
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

fn _create_credentialdef(issuer_did: &str,
                         schema_id: &str,
                         tag: &str,
                         revocation_details: &RevocationDetails) -> VcxResult<(String, String, Option<String>, Option<String>, Option<String>)> {
    let (_, schema_json) = anoncreds::get_schema_json(&schema_id)?;

    let (cred_def_id, cred_def_json) = anoncreds::generate_cred_def(issuer_did,
                                                                    &schema_json,
                                                                    tag,
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

            let rev_reg_def = _maybe_set_url(&rev_reg_def, revocation_details)?;

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

fn _maybe_set_url(rev_reg_def_json: &str, revocation_details: &RevocationDetails) -> VcxResult<String> {
    let mut rev_reg_def: RevocationRegistryDefinition = serde_json::from_str(&rev_reg_def_json)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Invalid RevocationRegistryDefinition: {:?}, err: {:?}", rev_reg_def_json, err)))?;

    if let Some(tails_url) = &revocation_details.tails_url {
        rev_reg_def.value.tails_location = tails_url.to_string();
    } else if let Some(tails_base_url) = &revocation_details.tails_base_url {
        rev_reg_def.value.tails_location = vec![tails_base_url.to_string(), rev_reg_def.value.tails_hash.to_owned()].join("/")
    }

    serde_json::to_string(&rev_reg_def)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to serialize RevocationRegistryDefinition: {:?}, err: {:?}", rev_reg_def, err)))
}

fn _parse_revocation_details(revocation_details: &str) -> VcxResult<RevocationDetails> {
    let revoc_details = serde_json::from_str::<RevocationDetails>(&revocation_details)
        .to_vcx(VcxErrorKind::InvalidRevocationDetails, "Cannot deserialize RevocationDetails")?;

    match revoc_details.tails_url.is_some() && revoc_details.tails_base_url.is_some() {
        true => Err(VcxError::from_msg(VcxErrorKind::InvalidOption, "It is allowed to specify either tails_location or tails_base_location, but not both")),
        false => Ok(revoc_details)
    }
}

fn _replace_tails_location(new_rev_reg_def: &str, revocation_details: &RevocationDetails) -> VcxResult<String> {
    trace!("_replace_tails_location >>> new_rev_reg_def: {}, revocation_details: {:?}", new_rev_reg_def, revocation_details);
    let mut new_rev_reg_def: RevocationRegistryDefinition = serde_json::from_str(new_rev_reg_def)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to deserialize new rev_reg_def: {:?}, error: {:?}", new_rev_reg_def, err)))?;

    let tails_location = match &revocation_details.tails_url {
        Some(tails_url) => tails_url.to_string(),
        None => match &revocation_details.tails_base_url {
            Some(tails_base_url) => vec![tails_base_url.to_string(), new_rev_reg_def.value.tails_hash.to_owned()].join("/"),
            None => return Err(VcxError::from_msg(VcxErrorKind::InvalidRevocationDetails, "Both tails_url and tails_base_location not found in revocation details"))
        }
    };

    new_rev_reg_def.value.tails_location = String::from(tails_location);

    serde_json::to_string(&new_rev_reg_def)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to serialize new rev_reg_def: {:?}, error: {:?}", new_rev_reg_def, err)))
}

impl CredentialDef {
    pub fn create(source_id: String, name: String, issuer_did: String, schema_id: String, tag: String, revocation_details: String) -> VcxResult<Self> {
        trace!("CredentialDef::create >>> source_id: {}, name: {}, issuer_did: {}, schema_id: {}, revocation_details: {}",
               source_id, name, issuer_did, schema_id, revocation_details);

        let revocation_details: RevocationDetails = _parse_revocation_details(&revocation_details)?;

        let (cred_def_id, cred_def_json, rev_reg_id, rev_reg_def, rev_reg_entry) = _create_credentialdef(&issuer_did, &schema_id, &tag, &revocation_details)?;

        let (rev_def_payment, rev_delta_payment, cred_def_payment_txn) = match _try_get_cred_def_from_ledger(&issuer_did, &cred_def_id) {
            Ok(Some(ledger_cred_def_json)) => {
                return Err(VcxError::from_msg(VcxErrorKind::CreateCredDef, format!("Credential definition with id {} already exists on the ledger: {}", cred_def_id, ledger_cred_def_json)));
            }
            Ok(None) => {
                let cred_def_payment_txn = anoncreds::publish_cred_def(&issuer_did, &cred_def_json)?;

                match (&rev_reg_id, &rev_reg_def, &rev_reg_entry) {
                    (Some(ref rev_reg_id), Some(ref rev_reg_def), Some(ref rev_reg_entry)) => {
                        let rev_def_payment = anoncreds::publish_rev_reg_def(&issuer_did, &rev_reg_def)
                            .map_err(|err| err.map(VcxErrorKind::CreateCredDef, "Cannot create CredentialDefinition"))?;

                        let (rev_delta_payment, _) = anoncreds::publish_rev_reg_delta(&issuer_did, &rev_reg_id, &rev_reg_entry)
                            .map_err(|err| err.map(VcxErrorKind::InvalidRevocationEntry, "Cannot post RevocationEntry"))?;

                        (rev_def_payment, rev_delta_payment, cred_def_payment_txn)
                    }
                    _ => (None, None, None)
                }
            }
            Err(err) => return Err(err)
        };

        let rev_reg = match (rev_reg_id, rev_reg_def, rev_reg_entry, revocation_details.tails_file, revocation_details.max_creds) {
            (Some(rev_reg_id), Some(rev_reg_def), Some(rev_reg_entry), Some(tails_file), Some(max_creds)) => {
                Some(RevocationRegistry {
                    rev_reg_id,
                    rev_reg_def,
                    rev_reg_entry,
                    tails_file,
                    max_creds,
                    tag: 1,
                    rev_reg_def_payment_txn: rev_def_payment,
                    rev_reg_delta_payment_txn: rev_delta_payment,
                })
            }
            _ => None
        };

        Ok(
            Self {
                source_id,
                name,
                tag,
                id: cred_def_id,
                issuer_did: Some(issuer_did),
                cred_def_payment_txn,
                rev_reg,
                state: PublicEntityStateType::Published,
            }
        )
    }

    pub fn from_str(data: &str) -> VcxResult<Self> {
        ObjectWithVersion::deserialize(data)
            .map(|obj: ObjectWithVersion<Self>| obj.data)
            .map_err(|err| err.into())
            .map_err(|err: VcxError| err.map(VcxErrorKind::CreateCredDef, "Cannot deserialize CredentialDefinition"))
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

    pub fn get_rev_reg_def(&self) -> Option<String> {
        match &self.rev_reg {
            Some(rev_reg) => Some(rev_reg.rev_reg_def.clone()),
            None => None
        }
    }

    pub fn get_cred_def_id(&self) -> String { self.id.clone() }

    pub fn set_name(&mut self, name: String) { self.name = name.clone(); }

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
            if let (Ok(_), Ok(_), Ok(_)) = (anoncreds::get_cred_def_json(&self.id),
                                            anoncreds::get_rev_reg_def_json(rev_reg_id),
                                            anoncreds::get_rev_reg(rev_reg_id, time::get_time().sec as u64)) {
                self.state = PublicEntityStateType::Published
            }
        } else {
            if let Ok(_) = anoncreds::get_cred_def_json(&self.id) {
                self.state = PublicEntityStateType::Published
            }
        }

        Ok(self.state as u32)
    }

    pub fn get_state(&self) -> u32 { self.state as u32 }

    pub fn rotate_rev_reg(&mut self, revocation_details: &str) -> VcxResult<RevocationRegistry> {
        debug!("CredentialDef::rotate_rev_reg >>> revocation_details: {}", revocation_details);
        let revocation_details = _parse_revocation_details(revocation_details)?;
        let (tails_file, max_creds, issuer_did) = (
            revocation_details.clone().tails_file.or(self.get_tails_file()),
            revocation_details.max_creds.or(self.get_max_creds()),
            self.issuer_did.as_ref()
        );
        match (&mut self.rev_reg, &tails_file, &max_creds, &issuer_did) {
            (Some(rev_reg), Some(tails_file), Some(max_creds), Some(issuer_did)) => {
                let tag = format!("tag{}", rev_reg.tag + 1);
                let (rev_reg_id, rev_reg_def, rev_reg_entry) =
                    anoncreds::generate_rev_reg(&issuer_did, &self.id, &tails_file, *max_creds, tag.as_str())
                        .map_err(|err| err.map(VcxErrorKind::CreateRevRegDef, "Cannot create revocation registry defintion"))?;

                let new_rev_reg_def = _replace_tails_location(&rev_reg_def, &revocation_details)?;

                let rev_reg_def_payment_txn = anoncreds::publish_rev_reg_def(&issuer_did, &new_rev_reg_def)
                    .map_err(|err| err.map(VcxErrorKind::CreateCredDef, "Cannot publish revocation registry defintion"))?;

                let (rev_reg_delta_payment_txn, _) = anoncreds::publish_rev_reg_delta(&issuer_did, &rev_reg_id, &rev_reg_entry)
                    .map_err(|err| err.map(VcxErrorKind::InvalidRevocationEntry, "Cannot post RevocationEntry"))?;

                let new_rev_reg = RevocationRegistry {
                    rev_reg_id,
                    rev_reg_def: new_rev_reg_def,
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
