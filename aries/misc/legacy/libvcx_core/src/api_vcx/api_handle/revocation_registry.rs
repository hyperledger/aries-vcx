use aries_vcx::common::primitives::revocation_registry::{
    RevocationRegistry, RevocationRegistryDefinition,
};

use crate::{
    api_vcx::{
        api_global::profile::{get_main_anoncreds, get_main_ledger_write, get_main_wallet},
        api_handle::object_cache::ObjectCache,
    },
    errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
};
lazy_static! {
    pub static ref REV_REG_MAP: ObjectCache<RevocationRegistry> =
        ObjectCache::<RevocationRegistry>::new("revocation-registry-cache");
}

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq, Eq)]
pub struct RevocationRegistryConfig {
    pub issuer_did: String,
    pub cred_def_id: String,
    pub tag: u32,
    pub tails_dir: String,
    pub max_creds: u32,
}

pub async fn create(config: RevocationRegistryConfig) -> LibvcxResult<u32> {
    let RevocationRegistryConfig {
        issuer_did,
        cred_def_id,
        tails_dir,
        max_creds,
        tag,
    } = config;
    let rev_reg = RevocationRegistry::create(
        get_main_wallet()?.as_ref(),
        get_main_anoncreds()?.as_ref(),
        &issuer_did.to_string().parse()?,
        &cred_def_id,
        &tails_dir,
        max_creds,
        tag,
    )
    .await?;
    let handle = REV_REG_MAP.add(rev_reg)?;
    Ok(handle)
}

pub async fn publish(handle: u32, tails_url: &str) -> LibvcxResult<u32> {
    let mut rev_reg = REV_REG_MAP.get_cloned(handle)?;
    rev_reg
        .publish_revocation_primitives(
            get_main_wallet()?.as_ref(),
            get_main_ledger_write()?.as_ref(),
            tails_url,
        )
        .await?;
    REV_REG_MAP.insert(handle, rev_reg)?;
    Ok(handle)
}

pub async fn publish_revocations(handle: u32, submitter_did: &str) -> LibvcxResult<()> {
    let rev_reg = REV_REG_MAP.get_cloned(handle)?;
    rev_reg
        .publish_local_revocations(
            get_main_wallet()?.as_ref(),
            get_main_anoncreds()?.as_ref(),
            get_main_ledger_write()?.as_ref(),
            &submitter_did.to_string().parse()?,
        )
        .await?;

    Ok(())
}

pub fn get_rev_reg_id(handle: u32) -> LibvcxResult<String> {
    REV_REG_MAP.get(handle, |rev_reg| Ok(rev_reg.rev_reg_id.clone()))
}

pub fn to_string(handle: u32) -> LibvcxResult<String> {
    REV_REG_MAP.get(handle, |rev_reg| {
        rev_reg.to_string().map_err(|err| err.into())
    })
}

pub fn from_string(rev_reg_data: &str) -> LibvcxResult<u32> {
    let rev_reg = RevocationRegistry::from_string(rev_reg_data)?;
    REV_REG_MAP.add(rev_reg)
}

pub fn release(handle: u32) -> LibvcxResult<()> {
    REV_REG_MAP
        .release(handle)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::InvalidHandle, e.to_string()))
}

pub fn get_tails_hash(handle: u32) -> LibvcxResult<String> {
    REV_REG_MAP.get(handle, |rev_reg| {
        Ok(rev_reg.get_rev_reg_def().value.tails_hash)
    })
}

pub fn get_rev_reg_def(handle: u32) -> LibvcxResult<RevocationRegistryDefinition> {
    REV_REG_MAP.get(handle, |rev_reg| Ok(rev_reg.get_rev_reg_def()))
}
