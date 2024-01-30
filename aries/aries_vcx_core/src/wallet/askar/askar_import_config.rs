use serde::Deserialize;

use crate::errors::error::VcxCoreResult;

#[derive(Deserialize)]
pub struct AskarImportConfig {}

impl AskarImportConfig {
    pub async fn import_wallet(&self) -> VcxCoreResult<()> {
        todo!()
    }
}
