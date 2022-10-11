use std::sync::Mutex;

use crate::error::*;
use crate::storage::in_memory::ObjectCache;
use aries_vcx::indy::primitives::credential_schema::Schema;
use aries_vcx::vdrtools_sys::{PoolHandle, WalletHandle};

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
        self.schemas.add(&schema.get_schema_id(), schema)
    }

    pub async fn publish_schema(&self, id: &str) -> AgentResult<()> {
        let schema = self.schemas.get_cloned(id)?;
        let schema = schema
            .publish(self.wallet_handle, self.pool_handle, None)
            .await?;
        self.schemas.add(&id, schema)?;
        Ok(())
    }

    pub async fn schema_json(&self, id: &str) -> AgentResult<String> {
        self.schemas
            .get_cloned(id)?
            .get_schema_json(self.wallet_handle, self.pool_handle)
            .await
            .map_err(|err| err.into())
    }

    pub async fn exists_by_name_and_version(&self, name: &str, version: &str) -> AgentResult<bool> {
        let name = name.to_string();
        let version = version.to_string();
        let f = |m: &&Mutex<Schema>| -> bool {
            let schema = m.lock().unwrap();
            schema.name == name && schema.version == version
        };
        self.schemas.exists(f)
    }

    pub fn get_by_id(&self, id: &str) -> AgentResult<Schema> {
        self.schemas.get_cloned(id)
    }
}
