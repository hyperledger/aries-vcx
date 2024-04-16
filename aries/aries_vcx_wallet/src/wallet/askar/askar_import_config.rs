use serde::Deserialize;

use crate::errors::error::VcxWalletResult;

#[derive(Deserialize, Clone, Copy, Debug)]
pub struct AskarImportConfig {}

impl AskarImportConfig {
    pub async fn import_wallet(&self) -> VcxWalletResult<()> {
        todo!()
    }
}
