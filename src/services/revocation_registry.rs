use std::path::Path;
use std::sync::Mutex;

use crate::error::*;
use crate::storage::object_cache::ObjectCache;
use aries_vcx::indy::primitives::revocation_registry::RevocationRegistry;
use aries_vcx::vdrtools_sys::{PoolHandle, WalletHandle};

pub struct ServiceRevocationRegistries {
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    issuer_did: String,
    rev_regs: ObjectCache<RevocationRegistry>,
}

impl ServiceRevocationRegistries {
    pub fn new(wallet_handle: WalletHandle, pool_handle: PoolHandle, issuer_did: String) -> Self {
        Self {
            wallet_handle,
            pool_handle,
            issuer_did,
            rev_regs: ObjectCache::new("rev-regs"),
        }
    }

    fn get_tails_hash(&self, id: &str) -> AgentResult<String> {
        let rev_reg = self.rev_regs.get(id)?;
        Ok(rev_reg.get_rev_reg_def().value.tails_hash)
    }

    pub fn get_tails_dir(&self, id: &str) -> AgentResult<String> {
        let rev_reg = self.rev_regs.get(id)?;
        Ok(rev_reg.get_tails_dir())
    }

    pub async fn create_rev_reg(&self, cred_def_id: &str, max_creds: u32) -> AgentResult<String> {
        let rev_reg = RevocationRegistry::create(
            self.wallet_handle,
            &self.issuer_did,
            cred_def_id,
            "/tmp",
            max_creds,
            1,
        )
        .await?;
        self.rev_regs.set(&rev_reg.get_rev_reg_id(), rev_reg)
    }

    pub fn tails_file_path(&self, id: &str) -> AgentResult<String> {
        Ok(Path::new(&self.get_tails_dir(id)?)
            .join(self.get_tails_hash(id)?)
            .to_str()
            .ok_or_else(|| {
                AgentError::from_msg(
                    AgentErrorKind::SerializationError,
                    "Failed to serialize tails file path",
                )
            })?
            .to_string())
    }

    pub async fn publish_rev_reg(&self, id: &str, tails_url: &str) -> AgentResult<()> {
        let mut rev_reg = self.rev_regs.get(id)?;
        rev_reg
            .publish_revocation_primitives(self.wallet_handle, self.pool_handle, tails_url)
            .await?;
        self.rev_regs.set(id, rev_reg)?;
        Ok(())
    }

    pub async fn revoke_credential_locally(&self, id: &str, cred_rev_id: &str) -> AgentResult<()> {
        let rev_reg = self.rev_regs.get(id)?;
        rev_reg
            .revoke_credential_local(self.wallet_handle, cred_rev_id)
            .await?;
        Ok(())
    }

    pub async fn publish_local_revocations(&self, id: &str) -> AgentResult<()> {
        let rev_reg = self.rev_regs.get(id)?;
        rev_reg
            .publish_local_revocations(self.wallet_handle, self.pool_handle, &self.issuer_did)
            .await?;
        Ok(())
    }

    pub fn find_by_cred_def_id(&self, cred_def_id: &str) -> AgentResult<Vec<String>> {
        let cred_def_id = cred_def_id.to_string();
        let f = |(id, m): (&String, &Mutex<RevocationRegistry>)| -> Option<String> {
            let rev_reg = m.lock().unwrap();
            if rev_reg.get_cred_def_id() == cred_def_id {
                Some(id.clone())
            } else {
                None
            }
        };
        self.rev_regs.find_by(f)
    }
}
