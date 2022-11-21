use std::sync::Mutex;

use crate::error::*;
use crate::storage::object_cache::ObjectCache;
use aries_vcx::indy::ledger::transactions::get_schema_json;
use aries_vcx::indy::primitives::credential_schema::Schema;
use aries_vcx::vdrtools::{PoolHandle, WalletHandle};

pub struct ServiceSchemas {
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    issuer_did: String,
    schemas: ObjectCache<Schema>,
}

impl ServiceSchemas {
    pub fn new(wallet_handle: WalletHandle, pool_handle: PoolHandle, issuer_did: String) -> Self {
        Self {
            wallet_handle,
            pool_handle,
            issuer_did,
            schemas: ObjectCache::new("schemas"),
        }
    }

    pub async fn create_schema(
        &self,
        name: &str,
        version: &str,
        attributes: &Vec<String>,
    ) -> AgentResult<String> {
        let schema = Schema::create("", &self.issuer_did, name, version, attributes).await?;
        self.schemas.set(&schema.get_schema_id(), schema)
    }

    pub async fn publish_schema(&self, thread_id: &str) -> AgentResult<()> {
        let schema = self.schemas.get(thread_id)?;
        let schema = schema
            .publish(self.wallet_handle, self.pool_handle, None)
            .await?;
        self.schemas.set(thread_id, schema)?;
        Ok(())
    }

    pub async fn schema_json(&self, thread_id: &str) -> AgentResult<String> {
        Ok(get_schema_json(self.wallet_handle, self.pool_handle, thread_id)
            .await?
            .1)
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
