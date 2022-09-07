use indy_sys::{WalletHandle, PoolHandle};

use crate::error::{VcxError, VcxErrorKind, VcxResult};
use crate::libindy::credential_def::PublicEntityStateType;
use crate::libindy::utils::anoncreds;
use crate::libindy::utils::anoncreds::RevocationRegistryDefinition;

#[derive(Clone, Deserialize, Debug, Serialize, PartialEq)]
pub struct RevocationRegistry {
    cred_def_id: String,
    issuer_did: String,
    pub rev_reg_id: String,
    pub(super) rev_reg_def: RevocationRegistryDefinition,
    pub(super) rev_reg_entry: String,
    pub(super) tails_dir: String,
    pub(super) max_creds: u32,
    pub(super) tag: u32,
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
        let (rev_reg_id, rev_reg_def, rev_reg_entry) = anoncreds::generate_rev_reg(
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
        anoncreds::publish_rev_reg_def(wallet_handle, pool_handle, issuer_did, &self.rev_reg_def)
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

    pub async fn publish_rev_reg_delta(&mut self, wallet_handle: WalletHandle, issuer_did: &str) -> VcxResult<()> {
        trace!(
            "RevocationRegistry::publish_rev_reg_delta >>> issuer_did:{}, rev_reg_id: {}",
            issuer_did,
            self.rev_reg_id
        );
        anoncreds::publish_rev_reg_delta(wallet_handle, issuer_did, &self.rev_reg_id, &self.rev_reg_entry)
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
        self.publish_built_rev_reg_delta(wallet_handle).await
    }

    async fn publish_built_rev_reg_delta(&mut self, wallet_handle: WalletHandle) -> VcxResult<()> {
        let issuer_did = &self.issuer_did.clone();
        if self.was_rev_reg_delta_published() {
            info!("No unpublished revocation registry delta found, nothing to publish")
        } else {
            self.publish_rev_reg_delta(wallet_handle, issuer_did).await?;
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
