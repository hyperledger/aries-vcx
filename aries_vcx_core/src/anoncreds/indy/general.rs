use vdrtools::Locator;

use crate::{errors::error::VcxCoreResult, SearchHandle};

pub(crate) async fn blob_storage_open_reader(base_dir: &str) -> VcxCoreResult<i32> {
    let tails_config = json!(
        {
            "base_dir":    base_dir,
            "uri_pattern": ""         // TODO remove, unused
        }
    )
    .to_string();

    let res = Locator::instance()
        .blob_storage_controller
        .open_reader("default".into(), tails_config)
        .await?;

    Ok(res)
}

pub(crate) async fn close_search_handle(search_handle: SearchHandle) -> VcxCoreResult<()> {
    Locator::instance()
        .prover_controller
        .close_credentials_search_for_proof_req(search_handle)
        .await?;

    Ok(())
}

pub async fn generate_nonce() -> VcxCoreResult<String> {
    let res = Locator::instance().verifier_controller.generate_nonce()?;
    Ok(res)
}
