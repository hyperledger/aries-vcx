use anoncreds_types::data_types::identifiers::{
    cred_def_id::CredentialDefinitionId, schema_id::SchemaId,
};
use aries_vcx::common::primitives::credential_definition::{CredentialDef, PublicEntityStateType};

use crate::{
    api_vcx::{
        api_global::profile::{
            get_main_anoncreds, get_main_ledger_read, get_main_ledger_write, get_main_wallet,
        },
        api_handle::object_cache::ObjectCache,
    },
    errors::error::{LibvcxError, LibvcxErrorKind, LibvcxResult},
};

lazy_static! {
    pub static ref CREDENTIALDEF_MAP: ObjectCache<CredentialDef> =
        ObjectCache::<CredentialDef>::new("credential-defs-cache");
}

pub async fn create(
    issuer_did: String,
    source_id: String,
    schema_id: String,
    tag: String,
    support_revocation: bool,
) -> LibvcxResult<u32> {
    let cred_def = CredentialDef::create(
        get_main_wallet()?.as_ref(),
        get_main_ledger_read()?.as_ref(),
        get_main_anoncreds()?.as_ref(),
        source_id,
        did_parser::Did::parse(issuer_did)?,
        SchemaId::new(schema_id)?,
        tag,
        support_revocation,
    )
    .await?;
    let handle = CREDENTIALDEF_MAP.add(cred_def)?;
    Ok(handle)
}

pub async fn publish(handle: u32) -> LibvcxResult<()> {
    let mut cd = CREDENTIALDEF_MAP.get_cloned(handle)?;
    if !cd.was_published() {
        cd = cd
            .publish_cred_def(
                get_main_wallet()?.as_ref(),
                get_main_ledger_read()?.as_ref(),
                get_main_ledger_write()?.as_ref(),
            )
            .await?;
    } else {
        info!("publish >>> Credential definition was already published")
    }
    CREDENTIALDEF_MAP.insert(handle, cd)
}

pub fn is_valid_handle(handle: u32) -> bool {
    CREDENTIALDEF_MAP.has_handle(handle)
}

pub fn to_string(handle: u32) -> LibvcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |cd| cd.to_string().map_err(|err| err.into()))
}

pub fn from_string(data: &str) -> LibvcxResult<u32> {
    let cred_def: CredentialDef = CredentialDef::from_string(data)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::CreateCredDef, e.to_string()))?;
    CREDENTIALDEF_MAP.add(cred_def)
}

pub fn get_source_id(handle: u32) -> LibvcxResult<String> {
    CREDENTIALDEF_MAP.get(handle, |c| Ok(c.get_source_id().clone()))
}

pub fn get_cred_def_id(handle: u32) -> LibvcxResult<CredentialDefinitionId> {
    CREDENTIALDEF_MAP.get(handle, |c| Ok(c.get_cred_def_id().to_owned()))
}

pub fn release(handle: u32) -> LibvcxResult<()> {
    CREDENTIALDEF_MAP
        .release(handle)
        .map_err(|e| LibvcxError::from_msg(LibvcxErrorKind::InvalidCredDefHandle, e.to_string()))
}

pub fn release_all() {
    CREDENTIALDEF_MAP.drain().ok();
}

pub async fn update_state(handle: u32) -> LibvcxResult<u32> {
    let mut cd = CREDENTIALDEF_MAP.get_cloned(handle)?;
    let res = cd.update_state(get_main_ledger_read()?.as_ref()).await?;
    CREDENTIALDEF_MAP.insert(handle, cd)?;
    Ok(res)
}

pub fn get_state(handle: u32) -> LibvcxResult<u32> {
    CREDENTIALDEF_MAP.get(handle, |s| Ok(s.get_state()))
}

pub fn check_is_published(handle: u32) -> LibvcxResult<bool> {
    CREDENTIALDEF_MAP.get(handle, |s| Ok(PublicEntityStateType::Published == s.state))
}
