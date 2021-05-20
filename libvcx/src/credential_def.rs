use serde_json;

use agency_client::ObjectWithVersion;

use crate::api::PublicEntityStateType;
use crate::error::prelude::*;
use crate::libindy::utils::{anoncreds, ledger};
use crate::libindy::utils::cache::update_rev_reg_ids_cache;
use crate::libindy::utils::payments::PaymentTxn;
use crate::utils::constants::DEFAULT_SERIALIZE_VERSION;
use crate::utils::object_cache::ObjectCache;

lazy_static! {
    static ref CREDENTIALDEF_MAP: ObjectCache<CredentialDef> = ObjectCache::<CredentialDef>::new("credential-defs-cache");
}

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq)]
struct RevocationRegistry {
    rev_reg_id: String,
    rev_reg_def: String,
    rev_reg_entry: String,
    tails_file: String,
    max_creds: u32,
    tag: u32,
    rev_reg_def_payment_txn: Option<PaymentTxn>,
    rev_reg_delta_payment_txn: Option<PaymentTxn>,
}

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq)]
pub struct CredentialDef {
    id: String,
    tag: String,
    name: String,
    source_id: String,
    issuer_did: Option<String>,
    cred_def_payment_txn: Option<PaymentTxn>,
    rev_reg: Option<RevocationRegistry>,
    #[serde(default)]
    state: PublicEntityStateType,
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
    pub fn from_str(data: &str) -> VcxResult<CredentialDef> {
        ObjectWithVersion::deserialize(data)
            .map(|obj: ObjectWithVersion<CredentialDef>| obj.data)
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

    pub fn get_rev_reg_id(&self) -> Option<&String> {
        match &self.rev_reg {
            Some(rev_reg) => Some(&rev_reg.rev_reg_id),
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

    pub fn get_rev_reg_def(&self) -> Option<&String> {
        match &self.rev_reg {
            Some(rev_reg) => Some(&rev_reg.rev_reg_def),
            None => None
        }
    }

    pub fn get_cred_def_id(&self) -> &String { &self.id }

    pub fn set_name(&mut self, name: String) { self.name = name.clone(); }

    pub fn set_source_id(&mut self, source_id: String) { self.source_id = source_id.clone(); }

    fn get_rev_reg_def_payment_txn(&self) -> Option<PaymentTxn> {
        match &self.rev_reg {
            Some(rev_reg) => rev_reg.rev_reg_def_payment_txn.clone(),
            None => None
        }
        // self.rev_reg_def_payment_txn.clone();
    }

    fn get_rev_reg_delta_payment_txn(&self) -> Option<PaymentTxn> {
        match &self.rev_reg {
            Some(rev_reg) => rev_reg.rev_reg_delta_payment_txn.clone(),
            None => None
        }
        // self.rev_reg_delta_payment_txn.clone();
    }

    fn update_state(&mut self) -> VcxResult<u32> {
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

    fn get_state(&self) -> u32 { self.state as u32 }

    fn rotate_rev_reg(&mut self, revocation_details: &str) -> VcxResult<RevocationRegistry> {
        debug!("rotate_rev_reg >>> revocation_details: {}", revocation_details);
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

fn _parse_revocation_details(revocation_details: &str) -> VcxResult<RevocationDetails> {
    let revoc_details = serde_json::from_str::<RevocationDetails>(&revocation_details)
        .to_vcx(VcxErrorKind::InvalidRevocationDetails, "Cannot deserialize RevocationDetails")?;

    match revoc_details.tails_url.is_some() && revoc_details.tails_base_url.is_some() {
       true => Err(VcxError::from_msg(VcxErrorKind::InvalidOption, "It is allowed to specify either tails_location or tails_base_location, but not both")),
       false => Ok(revoc_details)
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

pub fn create_and_publish_credentialdef(source_id: String,
                                        name: String,
                                        issuer_did: String,
                                        schema_id: String,
                                        tag: String,
                                        revocation_details: String) -> VcxResult<u32> {
    trace!("create_and_publish_credentialdef >>> source_id: {}, name: {}, issuer_did: {}, schema_id: {}, revocation_details: {}",
           source_id, name, issuer_did, schema_id, revocation_details);

    let revocation_details: RevocationDetails = _parse_revocation_details(&revocation_details)?;

    let (cred_def_id, cred_def_json, rev_reg_id, rev_reg_def, rev_reg_entry) = _create_credentialdef(&issuer_did, &schema_id, &tag, &revocation_details)?;

    let (rev_def_payment, rev_delta_payment, cred_def_payment_txn) = match _try_get_cred_def_from_ledger(&issuer_did, &cred_def_id) {
        Ok(Some(ledger_cred_def_json)) => {
            return Err(VcxError::from_msg(VcxErrorKind::CreateCredDef, format!("Credential definition with id {} already exists on the ledger: {}", cred_def_id, ledger_cred_def_json)))
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

    let cred_def = CredentialDef {
        source_id,
        name,
        tag,
        id: cred_def_id,
        issuer_did: Some(issuer_did),
        cred_def_payment_txn,
        rev_reg,
        state: PublicEntityStateType::Published,
    };

    let handle = CREDENTIALDEF_MAP.add(cred_def).or(Err(VcxError::from(VcxErrorKind::CreateCredDef)))?;

    Ok(handle)
}

pub fn publish_revocations(handle: u32) -> VcxResult<()> {
    CREDENTIALDEF_MAP.get(handle, |cd| {
        if let Some(rev_reg_id) = cd.get_rev_reg_id() {
            anoncreds::publish_local_revocations(rev_reg_id.as_str())?;
            Ok(())
        } else {
            Err(VcxError::from(VcxErrorKind::InvalidCredDefHandle))
        }
    })
}

pub fn is_valid_handle(handle: u32) -> bool {
    CREDENTIALDEF_MAP.has_handle(handle)
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |cd| {
        cd.to_string()
    })
}

pub fn from_string(data: &str) -> VcxResult<u32> {
    let cred_def: CredentialDef = CredentialDef::from_str(data)?;
    CREDENTIALDEF_MAP.add(cred_def)
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        Ok(c.get_source_id().clone())
    })
}

pub fn get_cred_def_id(handle: u32) -> VcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        Ok(c.get_cred_def_id().clone())
    })
}

pub fn get_rev_reg_id(handle: u32) -> VcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        c.get_rev_reg_id().cloned().ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, "No revocation registry found - does this credential definiton support revocation?"))
    })
}

pub fn get_tails_file(handle: u32) -> VcxResult<Option<String>> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        Ok(c.get_tails_file())
    })
}

pub fn get_rev_reg_def(handle: u32) -> VcxResult<Option<String>> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        Ok(c.get_rev_reg_def().cloned())
    })
}

pub fn get_rev_reg_def_payment_txn(handle: u32) -> VcxResult<Option<PaymentTxn>> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        Ok(c.get_rev_reg_def_payment_txn())
    })
}


pub fn get_rev_reg_delta_payment_txn(handle: u32) -> VcxResult<Option<PaymentTxn>> {
    CREDENTIALDEF_MAP.get(handle, |c| {
        Ok(c.get_rev_reg_delta_payment_txn())
    })
}

pub fn release(handle: u32) -> VcxResult<()> {
    CREDENTIALDEF_MAP.release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidCredDefHandle)))
}

pub fn release_all() {
    CREDENTIALDEF_MAP.drain().ok();
}

pub fn update_state(handle: u32) -> VcxResult<u32> {
    CREDENTIALDEF_MAP.get_mut(handle, |s| {
        s.update_state()
    })
}

pub fn get_state(handle: u32) -> VcxResult<u32> {
    CREDENTIALDEF_MAP.get_mut(handle, |s| {
        Ok(s.get_state())
    })
}

pub fn check_is_published(handle: u32) -> VcxResult<bool> {
    CREDENTIALDEF_MAP.get_mut(handle, |s| {
        Ok(PublicEntityStateType::Published == s.state)
    })
}

pub fn rotate_rev_reg_def(handle: u32, revocation_details: &str) -> VcxResult<String> {
    CREDENTIALDEF_MAP.get_mut(handle, |s| {
        match &s.issuer_did {
            Some(_) => {
                let new_rev_reg = s.rotate_rev_reg(revocation_details)?;
                match update_rev_reg_ids_cache(&s.id, &new_rev_reg.rev_reg_id) {
                    Ok(()) => s.to_string(),
                    Err(err) => Err(err)
                }
            }
            // TODO: Better error
            None => Err(VcxError::from(VcxErrorKind::InvalidCredentialHandle))
        }
    })
}

pub fn get_tails_hash(handle: u32) -> VcxResult<String> {
    CREDENTIALDEF_MAP.get_mut(handle, |s| {
        match &s.get_rev_reg_def() {
            Some(rev_reg_def) => {
                let rev_reg_def: RevocationRegistryDefinition = serde_json::from_str(&rev_reg_def)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError, format!("Failed to deserialize current rev_reg_def: {:?}, error: {:?}", rev_reg_def, err)))?;
                Ok(rev_reg_def.value.tails_hash)
            }
            None => Err(VcxError::from(VcxErrorKind::InvalidCredentialHandle))
        }
    })
}

#[cfg(test)]
pub mod tests {
    use std::{
        thread::sleep,
        time::Duration,
    };

    use crate::{libindy, schema, settings, utils};
    #[cfg(feature = "pool_tests")]
    use crate::libindy::utils::payments::add_new_did;
    use crate::utils::{
        constants::SCHEMA_ID,
        get_temp_dir_path,
    };
    use crate::utils::devsetup::*;

    use super::*;

    static CREDENTIAL_DEF_NAME: &str = "Test Credential Definition";
    static ISSUER_DID: &str = "4fUDR9R7fjwELRvH9JT6HH";

    pub fn revocation_details(revoc: bool) -> serde_json::Value {
        let mut revocation_details = json!({"support_revocation":revoc});
        if revoc {
            revocation_details["tails_file"] = json!(get_temp_dir_path("tails_file.txt").to_str().unwrap());
            revocation_details["max_creds"] = json!(10);
        }
        revocation_details
    }

    pub fn prepare_create_cred_def_data(revoc: bool) -> (u32, String, String, serde_json::Value) {
        let schema_handle = schema::tests::create_schema_real();
        sleep(Duration::from_secs(2));
        let schema_id = schema::get_schema_id(schema_handle).unwrap();
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let revocation_details = revocation_details(revoc);
        (schema_handle, schema_id, did, revocation_details)
    }

    pub fn create_cred_def_real(revoc: bool) -> (u32, u32) {
        let (schema_handle, schema_id, did, revocation_details) = prepare_create_cred_def_data(revoc);
        sleep(Duration::from_secs(2));
        let cred_def_handle = create_and_publish_credentialdef("1".to_string(),
                                                               CREDENTIAL_DEF_NAME.to_string(),
                                                               did,
                                                               schema_id,
                                                               "tag_1".to_string(),
                                                               revocation_details.to_string()).unwrap();

        (schema_handle, cred_def_handle)
    }

    pub fn create_cred_def_fake() -> u32 {
        let rev_details = json!({"support_revocation": true, "tails_file": utils::constants::TEST_TAILS_FILE, "max_creds": 2, "tails_url": utils::constants::TEST_TAILS_URL}).to_string();

        create_and_publish_credentialdef("SourceId".to_string(),
                                         CREDENTIAL_DEF_NAME.to_string(),
                                         ISSUER_DID.to_string(),
                                         SCHEMA_ID.to_string(),
                                         "tag".to_string(),
                                         rev_details).unwrap()
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_cred_def_without_rev_will_have_no_rev_id() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (_, handle) = create_cred_def_real(false);
        let rev_reg_id = get_rev_reg_id(handle).ok();
        assert!(rev_reg_id.is_none());

        let (_, handle) = create_cred_def_real(true);
        let rev_reg_id = get_rev_reg_id(handle).ok();
        assert!(rev_reg_id.is_some());
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_create_cred_def() {
        let _setup = SetupMocks::init();

        let (_, handle) = create_cred_def_real(false);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_get_credential_def() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();
        let (_, _, cred_def_id, cred_def_json, _, _) = libindy::utils::anoncreds::tests::create_and_store_credential_def(utils::constants::DEFAULT_SCHEMA_ATTRS, false);

        let (id, r_cred_def_json) = libindy::utils::anoncreds::get_cred_def_json(&cred_def_id).unwrap();

        assert_eq!(id, cred_def_id);
        let def1: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let def2: serde_json::Value = serde_json::from_str(&r_cred_def_json).unwrap();
        assert_eq!(def1, def2);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_revocable_fails_with_no_tails_file() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (schema_id, _) = libindy::utils::anoncreds::tests::create_and_write_test_schema(utils::constants::DEFAULT_SCHEMA_ATTRS);
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        let rc = create_and_publish_credentialdef("1".to_string(),
                                                  "test_create_revocable_fails_with_no_tails_file".to_string(),
                                                  did,
                                                  schema_id,
                                                  "tag_1".to_string(),
                                                  r#"{"support_revocation":true}"#.to_string());
        assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::InvalidRevocationDetails);
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_tails_url_written_to_ledger() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (schema_id, _) = libindy::utils::anoncreds::tests::create_and_write_test_schema(utils::constants::DEFAULT_SCHEMA_ATTRS);
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        let revocation_details = json!({"support_revocation": true, "tails_file": get_temp_dir_path("tails.txt").to_str().unwrap(), "max_creds": 2, "tails_url": utils::constants::TEST_TAILS_URL.to_string()}).to_string();
        let handle = create_and_publish_credentialdef("1".to_string(),
                                                      "test_tails_url_written_to_ledger".to_string(),
                                                      did,
                                                      schema_id,
                                                      "tag1".to_string(),
                                                      revocation_details).unwrap();
        let rev_reg_def = get_rev_reg_def(handle).unwrap().unwrap();
        let rev_reg_def: serde_json::Value = serde_json::from_str(&rev_reg_def).unwrap();
        let _rev_reg_id = get_rev_reg_id(handle).unwrap();
        assert_eq!(rev_reg_def["value"]["tailsLocation"], utils::constants::TEST_TAILS_URL.to_string());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_tails_base_url_written_to_ledger() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();
        let tails_url = utils::constants::TEST_TAILS_URL.to_string();

        let (schema_id, _) = libindy::utils::anoncreds::tests::create_and_write_test_schema(utils::constants::DEFAULT_SCHEMA_ATTRS);
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        let revocation_details = json!({"support_revocation": true, "tails_file": get_temp_dir_path("tails.txt").to_str().unwrap(), "max_creds": 2, "tails_base_url": tails_url}).to_string();
        let handle = create_and_publish_credentialdef("1".to_string(),
                                                      "test_tails_url_written_to_ledger".to_string(),
                                                      did,
                                                      schema_id,
                                                      "tag1".to_string(),
                                                      revocation_details).unwrap();
        let rev_reg_def = get_rev_reg_def(handle).unwrap().unwrap();
        let rev_reg_def: serde_json::Value = serde_json::from_str(&rev_reg_def).unwrap();
        let tails_hash = get_tails_hash(handle).unwrap();
        assert_eq!(rev_reg_def["value"]["tailsLocation"], vec![tails_url, tails_hash].join("/"));
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_revocable_cred_def_with_payments() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (schema_id, _) = libindy::utils::anoncreds::tests::create_and_write_test_schema(utils::constants::DEFAULT_SCHEMA_ATTRS);
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        let revocation_details = json!({"support_revocation": true, "tails_file": get_temp_dir_path("tails.txt").to_str().unwrap(), "max_creds": 2}).to_string();
        let handle = create_and_publish_credentialdef("1".to_string(),
                                                      "test_create_revocable_cred_def".to_string(),
                                                      did,
                                                      schema_id,
                                                      "tag_1".to_string(),
                                                      revocation_details).unwrap();

        assert!(get_rev_reg_def(handle).unwrap().is_some());
        assert!(get_rev_reg_id(handle).ok().is_some());
        assert!(get_rev_reg_def_payment_txn(handle).unwrap().is_some());
        assert!(get_rev_reg_delta_payment_txn(handle).unwrap().is_some());
        let cred_id = get_cred_def_id(handle).unwrap();
        libindy::utils::anoncreds::get_cred_def_json(&cred_id).unwrap();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_credential_def_real() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (_, handle) = create_cred_def_real(false);

        let _source_id = get_source_id(handle).unwrap();
        let _cred_def_id = get_cred_def_id(handle).unwrap();
        let _schema_json = to_string(handle).unwrap();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_credential_def_no_fees_real() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (_, handle) = create_cred_def_real(false);

        let _source_id = get_source_id(handle).unwrap();
        let _cred_def_id = get_cred_def_id(handle).unwrap();
        let _schema_json = to_string(handle).unwrap();
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_create_credential_works_twice() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        let (_, schema_id, did, revocation_details) = prepare_create_cred_def_data(false);
        create_and_publish_credentialdef("1".to_string(),
                                         "name".to_string(),
                                         did.clone(),
                                         schema_id.clone(),
                                         "tag_1".to_string(),
                                         revocation_details.to_string()).unwrap();

        let err = create_and_publish_credentialdef("1".to_string(),
                                                   "name".to_string(),
                                                   did.clone(),
                                                   schema_id.clone(),
                                                   "tag_1".to_string(),
                                                   revocation_details.to_string()).unwrap_err();

        assert_eq!(err.kind(), VcxErrorKind::CreateCredDef);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_to_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = create_cred_def_fake();

        let credential_string = to_string(handle).unwrap();
        let credential_values: serde_json::Value = serde_json::from_str(&credential_string).unwrap();
        assert_eq!(credential_values["version"].clone(), "1.0");
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_from_string_succeeds() {
        let _setup = SetupMocks::init();

        let handle = create_cred_def_fake();
        let credentialdef_data = to_string(handle).unwrap();
        assert!(!credentialdef_data.is_empty());
        release(handle).unwrap();

        let new_handle = from_string(&credentialdef_data).unwrap();
        let new_credentialdef_data = to_string(new_handle).unwrap();

        let credentialdef1: CredentialDef = CredentialDef::from_str(&credentialdef_data).unwrap();
        let credentialdef2: CredentialDef = CredentialDef::from_str(&new_credentialdef_data).unwrap();

        assert_eq!(credentialdef1, credentialdef2);
        assert_eq!(CredentialDef::from_str("{}").unwrap_err().kind(), VcxErrorKind::CreateCredDef);
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_release_all() {
        let _setup = SetupMocks::init();

        let h1 = create_and_publish_credentialdef("SourceId".to_string(), CREDENTIAL_DEF_NAME.to_string(), ISSUER_DID.to_string(), SCHEMA_ID.to_string(), "tag".to_string(), "{}".to_string()).unwrap();
        let h2 = create_and_publish_credentialdef("SourceId".to_string(), CREDENTIAL_DEF_NAME.to_string(), ISSUER_DID.to_string(), SCHEMA_ID.to_string(), "tag".to_string(), "{}".to_string()).unwrap();
        let h3 = create_and_publish_credentialdef("SourceId".to_string(), CREDENTIAL_DEF_NAME.to_string(), ISSUER_DID.to_string(), SCHEMA_ID.to_string(), "tag".to_string(), "{}".to_string()).unwrap();
        let h4 = create_and_publish_credentialdef("SourceId".to_string(), CREDENTIAL_DEF_NAME.to_string(), ISSUER_DID.to_string(), SCHEMA_ID.to_string(), "tag".to_string(), "{}".to_string()).unwrap();
        let h5 = create_and_publish_credentialdef("SourceId".to_string(), CREDENTIAL_DEF_NAME.to_string(), ISSUER_DID.to_string(), SCHEMA_ID.to_string(), "tag".to_string(), "{}".to_string()).unwrap();
        release_all();
        assert_eq!(release(h1).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(release(h2).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(release(h3).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(release(h4).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
        assert_eq!(release(h5).unwrap_err().kind(), VcxErrorKind::InvalidCredDefHandle);
    }
}
