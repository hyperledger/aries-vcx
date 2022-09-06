use aries_vcx::error::prelude::*;
use aries_vcx::libindy::credential_def::revocation_registry::RevocationRegistry;
use aries_vcx::libindy::utils::anoncreds;
use aries_vcx::libindy::utils::anoncreds::RevocationRegistryDefinition;

use crate::api_lib::api_handle::object_cache::ObjectCache;
use crate::api_lib::global::wallet::get_main_wallet_handle;

lazy_static! {
    pub static ref REV_REG_MAP: ObjectCache<RevocationRegistry> =
        ObjectCache::<RevocationRegistry>::new("revocation-registry-cache");
}

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq)]
pub struct RevocationRegistryConfig {
    pub issuer_did: String,
    pub cred_def_id: String,
    pub tag: u32,
    pub tails_dir: String,
    pub max_creds: u32,
}

pub async fn create(config: RevocationRegistryConfig) -> VcxResult<u32> {
    let RevocationRegistryConfig {
        issuer_did,
        cred_def_id,
        tails_dir,
        max_creds,
        tag,
    } = config;
    let rev_reg = RevocationRegistry::create(
        get_main_wallet_handle(),
        &issuer_did,
        &cred_def_id,
        &tails_dir,
        max_creds,
        tag,
    )
    .await?;
    let handle = REV_REG_MAP.add(rev_reg)?;
    Ok(handle)
}

pub async fn publish(handle: u32, tails_url: &str) -> VcxResult<u32> {
    let mut rev_reg = REV_REG_MAP.get_cloned(handle)?;
    rev_reg
        .publish_revocation_primitives(get_main_wallet_handle(), tails_url)
        .await?;
    REV_REG_MAP.insert(handle, rev_reg)?;
    Ok(handle)
}

pub async fn publish_revocations(handle: u32, issuer_did: &str) -> VcxResult<()> {
    let rev_reg = REV_REG_MAP.get_cloned(handle)?;
    let rev_reg_id = rev_reg.get_rev_reg_id();
    // TODO: Check result
    anoncreds::publish_local_revocations(get_main_wallet_handle(), &rev_reg_id, issuer_did).await?;
    Ok(())
}

pub fn get_rev_reg_id(handle: u32) -> VcxResult<String> {
    REV_REG_MAP.get(handle, |rev_reg| Ok(rev_reg.rev_reg_id.clone()))
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    REV_REG_MAP.get(handle, |rev_reg| rev_reg.to_string().map_err(|err| err.into()))
}

pub fn from_string(rev_reg_data: &str) -> VcxResult<u32> {
    let rev_reg = RevocationRegistry::from_string(rev_reg_data)?;
    REV_REG_MAP.add(rev_reg).map_err(|err| err.into())
}

pub fn release(handle: u32) -> VcxResult<()> {
    REV_REG_MAP
        .release(handle)
        .or(Err(VcxError::from(VcxErrorKind::InvalidHandle)))
}

pub fn get_tails_hash(handle: u32) -> VcxResult<String> {
    REV_REG_MAP.get(handle, |rev_reg| Ok(rev_reg.get_rev_reg_def().value.tails_hash))
}

pub fn get_rev_reg_def(handle: u32) -> VcxResult<RevocationRegistryDefinition> {
    REV_REG_MAP.get(handle, |rev_reg| Ok(rev_reg.get_rev_reg_def()))
}
