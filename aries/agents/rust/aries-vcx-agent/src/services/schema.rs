use std::sync::{Arc, Mutex};

use aries_vcx::common::primitives::credential_schema::Schema;
use aries_vcx_core::{
    anoncreds::credx_anoncreds::IndyCredxAnonCreds,
    ledger::{
        base_ledger::AnoncredsLedgerRead,
        indy_vdr_ledger::{DefaultIndyLedgerRead, DefaultIndyLedgerWrite},
    },
    wallet::{base_wallet::BaseWallet, indy::IndySdkWallet},
};

use crate::{
    error::*,
    storage::{object_cache::ObjectCache, Storage},
};

pub struct ServiceSchemas {
    ledger_read: Arc<DefaultIndyLedgerRead>,
    ledger_write: Arc<DefaultIndyLedgerWrite>,
    anoncreds: IndyCredxAnonCreds,
    wallet: Arc<dyn BaseWallet>,
    issuer_did: String,
    schemas: ObjectCache<Schema>,
}

impl ServiceSchemas {
    pub fn new(
        ledger_read: Arc<DefaultIndyLedgerRead>,
        ledger_write: Arc<DefaultIndyLedgerWrite>,
        anoncreds: IndyCredxAnonCreds,
        wallet: Arc<dyn BaseWallet>,
        issuer_did: String,
    ) -> Self {
        Self {
            issuer_did,
            schemas: ObjectCache::new("schemas"),
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
        attributes: &Vec<String>,
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
        self.schemas.insert(&schema.get_schema_id(), schema)
    }

    pub async fn publish_schema(&self, thread_id: &str) -> AgentResult<()> {
        let schema = self.schemas.get(thread_id)?;
        let schema = schema
            .publish(&self.wallet, self.ledger_write.as_ref())
            .await?;
        self.schemas.insert(thread_id, schema)?;
        Ok(())
    }

    pub async fn schema_json(&self, thread_id: &str) -> AgentResult<String> {
        let ledger = self.ledger_read.as_ref();
        Ok(ledger.get_schema(thread_id, None).await?)
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
