use std::sync::{Arc, Mutex};

use anoncreds_types::data_types::identifiers::schema_id::SchemaId;
use aries_vcx::{common::primitives::credential_definition::CredentialDef, did_parser_nom::Did};
use aries_vcx_anoncreds::anoncreds::anoncreds::Anoncreds;
use aries_vcx_ledger::ledger::indy_vdr_ledger::{DefaultIndyLedgerRead, DefaultIndyLedgerWrite};
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;

use crate::{
    error::*,
    storage::{agent_storage_inmem::AgentStorageInMem, AgentStorage},
};

pub struct ServiceCredentialDefinitions<T> {
    ledger_read: Arc<DefaultIndyLedgerRead>,
    ledger_write: Arc<DefaultIndyLedgerWrite>,
    anoncreds: Anoncreds,
    wallet: Arc<T>,
    cred_defs: AgentStorageInMem<CredentialDef>,
}

impl<T: BaseWallet> ServiceCredentialDefinitions<T> {
    pub fn new(
        ledger_read: Arc<DefaultIndyLedgerRead>,
        ledger_write: Arc<DefaultIndyLedgerWrite>,
        anoncreds: Anoncreds,
        wallet: Arc<T>,
    ) -> Self {
        Self {
            cred_defs: AgentStorageInMem::new("cred-defs"),
            ledger_read,
            ledger_write,
            anoncreds,
            wallet,
        }
    }

    pub async fn create_cred_def(
        &self,
        issuer_did: Did,
        schema_id: SchemaId,
        tag: String,
    ) -> AgentResult<String> {
        let cd = CredentialDef::create(
            self.wallet.as_ref(),
            self.ledger_read.as_ref(),
            &self.anoncreds,
            "".to_string(),
            issuer_did,
            schema_id,
            tag,
            true,
        )
        .await?;
        self.cred_defs.insert(&cd.get_cred_def_id().to_string(), cd)
    }

    pub async fn publish_cred_def(&self, thread_id: &str) -> AgentResult<()> {
        let cred_def = self.cred_defs.get(thread_id)?;
        let cred_def = cred_def
            .publish_cred_def(
                self.wallet.as_ref(),
                self.ledger_read.as_ref(),
                self.ledger_write.as_ref(),
            )
            .await?;
        self.cred_defs.insert(thread_id, cred_def)?;
        Ok(())
    }

    pub fn cred_def_json(&self, thread_id: &str) -> AgentResult<String> {
        self.cred_defs
            .get(thread_id)?
            .get_data_json()
            .map_err(|err| err.into())
    }

    pub fn find_by_schema_id(&self, schema_id: &str) -> AgentResult<Vec<String>> {
        let schema_id = schema_id.to_string();
        let f = |(id, m): (&String, &Mutex<CredentialDef>)| -> Option<String> {
            let cred_def = m.lock().unwrap();
            if cred_def.get_schema_id().to_string() == schema_id {
                Some(id.clone())
            } else {
                None
            }
        };
        self.cred_defs.find_by(f)
    }
}
