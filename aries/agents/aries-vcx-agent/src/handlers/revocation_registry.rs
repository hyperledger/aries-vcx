use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use anoncreds_types::data_types::identifiers::cred_def_id::CredentialDefinitionId;
use aries_vcx::{common::primitives::revocation_registry::RevocationRegistry, did_parser_nom::Did};
use aries_vcx_anoncreds::anoncreds::anoncreds::Anoncreds;
use aries_vcx_ledger::ledger::indy_vdr_ledger::{DefaultIndyLedgerRead, DefaultIndyLedgerWrite};
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;

use crate::{
    error::*,
    storage::{agent_storage_inmem::AgentStorageInMem, AgentStorage},
};

pub struct ServiceRevocationRegistries<T> {
    ledger_write: Arc<DefaultIndyLedgerWrite>,
    ledger_read: Arc<DefaultIndyLedgerRead>,
    anoncreds: Anoncreds,
    wallet: Arc<T>,
    issuer_did: Did,
    rev_regs: AgentStorageInMem<RevocationRegistry>,
}

impl<T: BaseWallet> ServiceRevocationRegistries<T> {
    pub fn new(
        ledger_write: Arc<DefaultIndyLedgerWrite>,
        ledger_read: Arc<DefaultIndyLedgerRead>,
        anoncreds: Anoncreds,
        wallet: Arc<T>,
        issuer_did: String,
    ) -> Self {
        Self {
            issuer_did: Did::parse(issuer_did).unwrap(), // TODO
            rev_regs: AgentStorageInMem::new("rev-regs"),
            ledger_write,
            ledger_read,
            anoncreds,
            wallet,
        }
    }

    fn get_tails_hash(&self, thread_id: &str) -> AgentResult<String> {
        let rev_reg = self.rev_regs.get(thread_id)?;
        Ok(rev_reg.get_rev_reg_def().value.tails_hash)
    }

    pub fn get_tails_dir(&self, thread_id: &str) -> AgentResult<String> {
        let rev_reg = self.rev_regs.get(thread_id)?;
        Ok(rev_reg.get_tails_dir())
    }

    pub async fn create_rev_reg(
        &self,
        cred_def_id: &CredentialDefinitionId,
        max_creds: u32,
    ) -> AgentResult<String> {
        let rev_reg = RevocationRegistry::create(
            self.wallet.as_ref(),
            &self.anoncreds,
            &self.issuer_did,
            cred_def_id,
            "/tmp",
            max_creds,
            1,
        )
        .await?;
        self.rev_regs.insert(&rev_reg.get_rev_reg_id(), rev_reg)
    }

    pub fn tails_file_path(&self, thread_id: &str) -> AgentResult<String> {
        Ok(Path::new(&self.get_tails_dir(thread_id)?)
            .join(self.get_tails_hash(thread_id)?)
            .to_str()
            .ok_or_else(|| {
                AgentError::from_msg(
                    AgentErrorKind::SerializationError,
                    "Failed to serialize tails file path",
                )
            })?
            .to_string())
    }

    pub async fn publish_rev_reg(&self, thread_id: &str, tails_url: &str) -> AgentResult<()> {
        let mut rev_reg = self.rev_regs.get(thread_id)?;
        rev_reg
            .publish_revocation_primitives(
                self.wallet.as_ref(),
                self.ledger_write.as_ref(),
                tails_url,
            )
            .await?;
        self.rev_regs.insert(thread_id, rev_reg)?;
        Ok(())
    }

    pub async fn revoke_credential_locally(&self, id: &str, cred_rev_id: &str) -> AgentResult<()> {
        let rev_reg = self.rev_regs.get(id)?;
        rev_reg
            .revoke_credential_local(
                self.wallet.as_ref(),
                &self.anoncreds,
                self.ledger_read.as_ref(),
                cred_rev_id.parse()?,
            )
            .await?;
        Ok(())
    }

    pub async fn publish_local_revocations(&self, id: &str) -> AgentResult<()> {
        let rev_reg = self.rev_regs.get(id)?;
        rev_reg
            .publish_local_revocations(
                self.wallet.as_ref(),
                &self.anoncreds,
                self.ledger_write.as_ref(),
                &self.issuer_did,
            )
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
