use std::sync::{Arc, Mutex};

use aries_vcx::{common::primitives::credential_schema::Schema, did_parser_nom::Did};
use aries_vcx_anoncreds::anoncreds::anoncreds::Anoncreds;
use aries_vcx_ledger::ledger::{
    base_ledger::AnoncredsLedgerRead,
    indy_vdr_ledger::{DefaultIndyLedgerRead, DefaultIndyLedgerWrite},
};
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;

use crate::{
    error::*,
    storage::{agent_storage_inmem::AgentStorageInMem, AgentStorage},
};

pub struct ServiceSchemas<T> {
    ledger_read: Arc<DefaultIndyLedgerRead>,
    ledger_write: Arc<DefaultIndyLedgerWrite>,
    anoncreds: Anoncreds,
    wallet: Arc<T>,
    issuer_did: Did,
    schemas: AgentStorageInMem<Schema>,
}

impl<T: BaseWallet> ServiceSchemas<T> {
    pub fn new(
        ledger_read: Arc<DefaultIndyLedgerRead>,
        ledger_write: Arc<DefaultIndyLedgerWrite>,
        anoncreds: Anoncreds,
        wallet: Arc<T>,
        issuer_did: String,
    ) -> Self {
        Self {
            issuer_did: Did::parse(issuer_did).unwrap(), // TODO
            schemas: AgentStorageInMem::new("schemas"),
            ledger_read,
            ledger_write,
            anoncreds,
            wallet,
        }
    }

    pub async fn create_schema(
        &self,
        name: &str,
        version: &str,
        attributes: Vec<String>,
    ) -> AgentResult<String> {
        let schema = Schema::create(
            &self.anoncreds,
            "",
            &self.issuer_did,
            name,
            version,
            attributes,
        )
        .await?;
        self.schemas
            .insert(&schema.get_schema_id().to_string(), schema)
    }

    pub async fn publish_schema(&self, thread_id: &str) -> AgentResult<()> {
        let schema = self.schemas.get(thread_id)?;
        let schema = schema
            .publish(self.wallet.as_ref(), self.ledger_write.as_ref())
            .await?;
        self.schemas.insert(thread_id, schema)?;
        Ok(())
    }

    pub async fn schema_json(&self, thread_id: &str) -> AgentResult<String> {
        let ledger = self.ledger_read.as_ref();
        Ok(serde_json::to_string(
            &ledger
                .get_schema(&thread_id.to_string().try_into()?, None)
                .await?,
        )?)
    }

    pub fn find_by_name_and_version(&self, name: &str, version: &str) -> AgentResult<Vec<String>> {
        let name = name.to_string();
        let version = version.to_string();
        let f = |(id, m): (&String, &Mutex<Schema>)| -> Option<String> {
            let schema = m.lock().unwrap();
            if schema.name == name && schema.version == version {
                Some(id.to_string())
            } else {
                None
            }
        };
        self.schemas.find_by(f)
    }

    pub fn get_by_id(&self, thread_id: &str) -> AgentResult<Schema> {
        self.schemas.get(thread_id)
    }
}
