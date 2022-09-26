use vdrtools_sys::{PoolHandle, WalletHandle};
use vdrtools::blob_storage;

use crate::error::{VcxError, VcxErrorKind, VcxResult};
use crate::global::settings;
use crate::indy::primitives::credential_definition::PublicEntityStateType;
use crate::indy::anoncreds;
use crate::indy::ledger::transactions::{build_rev_reg_delta_request, build_rev_reg_request, check_response, sign_and_submit_to_ledger};
use crate::indy::wallet_non_secrets::{clear_rev_reg_delta, get_rev_reg_delta, set_rev_reg_delta};
use crate::utils::constants::REV_REG_ID;

pub const BLOB_STORAGE_TYPE: &str = "default";
pub const REVOCATION_REGISTRY_TYPE: &str = "ISSUANCE_BY_DEFAULT";

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq)]
pub struct RevocationRegistry {
    cred_def_id: String,
    issuer_did: String,
    pub rev_reg_id: String,
    pub(in crate::indy) rev_reg_def: RevocationRegistryDefinition,
    pub(in crate::indy) rev_reg_entry: String,
    pub(in crate::indy) tails_dir: String,
    pub(in crate::indy) max_creds: u32,
    pub(in crate::indy) tag: u32,
    rev_reg_def_state: PublicEntityStateType,
    rev_reg_delta_state: PublicEntityStateType,
}

impl RevocationRegistry {
    pub async fn create(
        wallet_handle: WalletHandle,
        issuer_did: &str,
        cred_def_id: &str,
        tails_dir: &str,
        max_creds: u32,
        tag: u32,
    ) -> VcxResult<RevocationRegistry> {
        trace!(
            "RevocationRegistry::create >>> issuer_did: {}, cred_def_id: {}, tails_dir: {}, max_creds: {}, tag_no: {}",
            issuer_did,
            cred_def_id,
            tails_dir,
            max_creds,
            tag
        );
        let (rev_reg_id, rev_reg_def, rev_reg_entry) = generate_rev_reg(
            wallet_handle,
            issuer_did,
            cred_def_id,
            tails_dir,
            max_creds,
            &format!("tag{}", tag),
        )
        .await
        .map_err(|err| err.map(VcxErrorKind::CreateRevRegDef, "Cannot create Revocation Registry"))?;
        Ok(RevocationRegistry {
            cred_def_id: cred_def_id.to_string(),
            issuer_did: issuer_did.to_string(),
            rev_reg_id,
            rev_reg_def,
            rev_reg_entry,
            tails_dir: tails_dir.to_string(),
            max_creds,
            tag,
            rev_reg_def_state: PublicEntityStateType::Built,
            rev_reg_delta_state: PublicEntityStateType::Built,
        })
    }

    pub fn get_rev_reg_id(&self) -> String {
        self.rev_reg_id.clone()
    }

    pub fn get_rev_reg_def(&self) -> RevocationRegistryDefinition {
        self.rev_reg_def.clone()
    }

    pub fn get_tails_dir(&self) -> String {
        self.tails_dir.clone()
    }

    pub fn was_rev_reg_def_published(&self) -> bool {
        self.rev_reg_def_state == PublicEntityStateType::Published
    }

    pub fn was_rev_reg_delta_published(&self) -> bool {
        self.rev_reg_delta_state == PublicEntityStateType::Published
    }

    pub async fn publish_rev_reg_def(
        &mut self,
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        issuer_did: &str,
        tails_url: &str,
    ) -> VcxResult<()> {
        trace!(
            "RevocationRegistry::publish_rev_reg_def >>> issuer_did:{}, rev_reg_id: {}, rev_reg_def:{:?}",
            issuer_did,
            &self.rev_reg_id,
            &self.rev_reg_def
        );
        self.rev_reg_def.value.tails_location = String::from(tails_url);
        publish_rev_reg_def(wallet_handle, pool_handle, issuer_did, &self.rev_reg_def)
            .await
            .map_err(|err| {
                err.map(
                    VcxErrorKind::InvalidState,
                    "Cannot publish revocation registry definition",
                )
            })?;
        self.rev_reg_def_state = PublicEntityStateType::Published;
        Ok(())
    }

    pub async fn publish_rev_reg_delta(&mut self, wallet_handle: WalletHandle, pool_handle: PoolHandle, issuer_did: &str) -> VcxResult<()> {
        trace!(
            "RevocationRegistry::publish_rev_reg_delta >>> issuer_did:{}, rev_reg_id: {}",
            issuer_did,
            self.rev_reg_id
        );
        publish_rev_reg_delta(wallet_handle, pool_handle, issuer_did, &self.rev_reg_id, &self.rev_reg_entry)
            .await
            .map_err(|err| err.map(VcxErrorKind::InvalidRevocationEntry, "Cannot post RevocationEntry"))?;
        self.rev_reg_delta_state = PublicEntityStateType::Published;
        Ok(())
    }

    pub async fn publish_revocation_primitives(
        &mut self,
        wallet_handle: WalletHandle,
        pool_handle: PoolHandle,
        tails_url: &str,
    ) -> VcxResult<()> {
        trace!(
            "RevocationRegistry::publish_revocation_primitives >>> tails_url: {}",
            tails_url
        );
        self.publish_built_rev_reg_def(wallet_handle, pool_handle, tails_url).await?;
        self.publish_built_rev_reg_delta(wallet_handle, pool_handle).await
    }

    async fn publish_built_rev_reg_delta(&mut self, wallet_handle: WalletHandle, pool_handle: PoolHandle) -> VcxResult<()> {
        let issuer_did = &self.issuer_did.clone();
        if self.was_rev_reg_delta_published() {
            info!("No unpublished revocation registry delta found, nothing to publish")
        } else {
            self.publish_rev_reg_delta(wallet_handle, pool_handle, issuer_did).await?;
        }
        Ok(())
    }

    async fn publish_built_rev_reg_def(&mut self, wallet_handle: WalletHandle, pool_handle: PoolHandle, tails_url: &str) -> VcxResult<()> {
        let issuer_did = &self.issuer_did.clone();
        if self.was_rev_reg_def_published() {
            info!("No unpublished revocation registry definition found, nothing to publish")
        } else {
            self.publish_rev_reg_def(wallet_handle, pool_handle, issuer_did, tails_url).await?;
        }
        Ok(())
    }

    pub fn to_string(&self) -> VcxResult<String> {
        serde_json::to_string(&self).map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::SerializationError,
                format!("Cannot serialize revocation registry: {:?}", err),
            )
        })
    }

    pub fn from_string(rev_reg_data: &str) -> VcxResult<Self> {
        serde_json::from_str(rev_reg_data).map_err(|err| {
            VcxError::from_msg(
                VcxErrorKind::InvalidJson,
                format!("Cannot deserialize revocation registry: {:?}", err),
            )
        })
    }
}

pub async fn libindy_create_and_store_revoc_reg(
    wallet_handle: WalletHandle,
    issuer_did: &str,
    cred_def_id: &str,
    tails_dir: &str,
    max_creds: u32,
    tag: &str,
) -> VcxResult<(String, String, String)> {
    trace!("creating revocation: {}, {}, {}", cred_def_id, tails_dir, max_creds);

    let tails_config = json!({"base_dir": tails_dir,"uri_pattern": ""}).to_string();

    let writer = blob_storage::open_writer(BLOB_STORAGE_TYPE, &tails_config).await?;

    let revoc_config = json!({"max_cred_num": max_creds, "issuance_type": REVOCATION_REGISTRY_TYPE}).to_string();

    vdrtools::anoncreds::issuer_create_and_store_revoc_reg(
        wallet_handle,
        issuer_did,
        None,
        tag,
        cred_def_id,
        &revoc_config,
        writer,
    )
        .await
        .map_err(VcxError::from)
}

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinitionValue {
    pub issuance_type: String,
    pub max_cred_num: u32,
    pub public_keys: serde_json::Value,
    pub tails_hash: String,
    pub tails_location: String,
}

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct RevocationRegistryDefinition {
    pub id: String,
    pub revoc_def_type: String,
    pub tag: String,
    pub cred_def_id: String,
    pub value: RevocationRegistryDefinitionValue,
    pub ver: String,
}

pub async fn libindy_issuer_revoke_credential(
    wallet_handle: WalletHandle,
    tails_file: &str,
    rev_reg_id: &str,
    cred_rev_id: &str,
) -> VcxResult<String> {
    let blob_handle = anoncreds::blob_storage_open_reader(tails_file).await?;

    vdrtools::anoncreds::issuer_revoke_credential(wallet_handle, blob_handle, rev_reg_id, cred_rev_id)
        .await
        .map_err(VcxError::from)
}

pub async fn libindy_issuer_merge_revocation_registry_deltas(old_delta: &str, new_delta: &str) -> VcxResult<String> {
    vdrtools::anoncreds::issuer_merge_revocation_registry_deltas(old_delta, new_delta)
        .await
        .map_err(VcxError::from)
}

pub async fn generate_rev_reg(
    wallet_handle: WalletHandle,
    issuer_did: &str,
    cred_def_id: &str,
    tails_dir: &str,
    max_creds: u32,
    tag: &str,
) -> VcxResult<(String, RevocationRegistryDefinition, String)> {
    trace!(
        "generate_rev_reg >>> issuer_did: {}, cred_def_id: {}, tails_file: {}, max_creds: {}, tag: {}",
        issuer_did,
        cred_def_id,
        tails_dir,
        max_creds,
        tag
    );
    if settings::indy_mocks_enabled() {
        debug!("generate_rev_reg >>> returning mocked value");
        return Ok((
            REV_REG_ID.to_string(),
            RevocationRegistryDefinition::default(),
            "".to_string(),
        ));
    }

    let (rev_reg_id, rev_reg_def_json, rev_reg_entry_json) =
        libindy_create_and_store_revoc_reg(wallet_handle, issuer_did, cred_def_id, tails_dir, max_creds, tag).await?;

    let rev_reg_def: RevocationRegistryDefinition = serde_json::from_str(&rev_reg_def_json).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!(
                "Failed to deserialize rev_reg_def: {:?}, error: {:?}",
                rev_reg_def_json, err
            ),
        )
    })?;

    Ok((rev_reg_id, rev_reg_def, rev_reg_entry_json))
}


pub async fn publish_rev_reg_def(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    issuer_did: &str,
    rev_reg_def: &RevocationRegistryDefinition,
) -> VcxResult<()> {
    trace!("publish_rev_reg_def >>> issuer_did: {}, rev_reg_def: ...", issuer_did);
    if settings::indy_mocks_enabled() {
        debug!("publish_rev_reg_def >>> mocked success");
        return Ok(());
    }

    let rev_reg_def_json = serde_json::to_string(&rev_reg_def).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::SerializationError,
            format!("Failed to serialize rev_reg_def: {:?}, error: {:?}", rev_reg_def, err),
        )
    })?;
    let rev_reg_def_req = build_rev_reg_request(issuer_did, &rev_reg_def_json).await?;
    let response = sign_and_submit_to_ledger(wallet_handle, pool_handle, issuer_did, &rev_reg_def_req).await?;
    check_response(&response)
}

pub async fn publish_rev_reg_delta(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    issuer_did: &str,
    rev_reg_id: &str,
    revoc_reg_delta_json: &str,
) -> VcxResult<String> {
    trace!(
        "publish_rev_reg_delta >>> issuer_did: {}, rev_reg_id: {}, revoc_reg_delta_json: {}",
        issuer_did,
        rev_reg_id,
        revoc_reg_delta_json
    );
    let request = build_rev_reg_delta_request(issuer_did, rev_reg_id, revoc_reg_delta_json).await?;
    let response = sign_and_submit_to_ledger(wallet_handle, pool_handle, issuer_did, &request).await?;
    check_response(&response)?;
    Ok(response)
}

pub async fn revoke_credential_local(
    wallet_handle: WalletHandle,
    tails_file: &str,
    rev_reg_id: &str,
    cred_rev_id: &str,
) -> VcxResult<()> {
    if settings::indy_mocks_enabled() {
        return Ok(());
    }
    let mut new_delta_json = libindy_issuer_revoke_credential(wallet_handle, tails_file, rev_reg_id, cred_rev_id).await?;
    debug!("revoke_credential_local >>> new_delta_json: {}", new_delta_json);
    if let Some(old_delta_json) = get_rev_reg_delta(wallet_handle, rev_reg_id).await {
        debug!("revoke_credential_local >>> old_delta_json: {}", old_delta_json);
        new_delta_json = libindy_issuer_merge_revocation_registry_deltas(old_delta_json.as_str(), new_delta_json.as_str()).await?;
        debug!("revoke_credential_local >>> merged_delta_json: {}", new_delta_json);
    }
    set_rev_reg_delta(wallet_handle, rev_reg_id, &new_delta_json).await
}

pub async fn publish_local_revocations(wallet_handle: WalletHandle, pool_handle: PoolHandle, submitter_did: &str, rev_reg_id: &str) -> VcxResult<()> {
    if let Some(delta) = get_rev_reg_delta(wallet_handle, rev_reg_id).await {
        publish_rev_reg_delta(wallet_handle, pool_handle, &submitter_did, rev_reg_id, &delta).await?;
        info!("publish_local_revocations >>> rev_reg_delta published for rev_reg_id {}", rev_reg_id);
        match clear_rev_reg_delta(wallet_handle, rev_reg_id).await {
            Ok(_) => {
                info!("publish_local_revocations >>> rev_reg_delta storage cleared for rev_reg_id {}", rev_reg_id);
                Ok(())
            },
            Err(err) => {
                error!("publish_local_revocations >>> failed to clear revocation delta storage for rev_reg_id: {}, error: {}", rev_reg_id, err);
                Err(VcxError::from(VcxErrorKind::RevDeltaFailedToClear))
            }
        }
    } else {
        Err(VcxError::from(VcxErrorKind::RevDeltaNotFound))
    }
}
