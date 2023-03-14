use aries_vcx::{
    common::primitives::revocation_registry::{RevocationRegistry, RevocationRegistryDefinition},
    global::settings::CONFIG_INSTITUTION_DID,
};

use crate::{
    api_vcx::{
        api_global::{
            profile::{get_main_profile, get_main_profile_optional_pool},
            settings::get_config_value,
        },
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
    let profile = get_main_profile_optional_pool(); // do not throw if pool is not open
    let rev_reg = RevocationRegistry::create(&profile, &issuer_did, &cred_def_id, &tails_dir, max_creds, tag).await?;
    let handle = REV_REG_MAP.add(rev_reg)?;
    Ok(handle)
}

pub async fn publish(handle: u32, tails_url: &str) -> LibvcxResult<u32> {
    let mut rev_reg = REV_REG_MAP.get_cloned(handle)?;
    let profile = get_main_profile()?;
    rev_reg.publish_revocation_primitives(&profile, tails_url).await?;
    REV_REG_MAP.insert(handle, rev_reg)?;
    Ok(handle)
}

pub async fn publish_revocations(handle: u32) -> LibvcxResult<()> {
    let submitter_did = get_config_value(CONFIG_INSTITUTION_DID)?;

    let rev_reg = REV_REG_MAP.get_cloned(handle)?;
    let rev_reg_id = rev_reg.get_rev_reg_id();

    // TODO: Check result
    let profile = get_main_profile()?;
    let anoncreds = profile.inject_anoncreds();
    anoncreds.publish_local_revocations(&submitter_did, &rev_reg_id).await?;
    Ok(())
}

pub fn get_rev_reg_id(handle: u32) -> LibvcxResult<String> {
    REV_REG_MAP.get(handle, |rev_reg| Ok(rev_reg.rev_reg_id.clone()))
}

pub fn to_string(handle: u32) -> LibvcxResult<String> {
    REV_REG_MAP.get(handle, |rev_reg| rev_reg.to_string().map_err(|err| err.into()))
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
    REV_REG_MAP.get(handle, |rev_reg| Ok(rev_reg.get_rev_reg_def().value.tails_hash))
}

pub fn get_rev_reg_def(handle: u32) -> LibvcxResult<RevocationRegistryDefinition> {
    REV_REG_MAP.get(handle, |rev_reg| Ok(rev_reg.get_rev_reg_def()))
}
