use vdrtools::{AttributeNames, DidValue, Locator, PoolHandle, WalletHandle};

use crate::{
    errors::error::VcxResult,
    global::settings,
    indy::ledger::transactions::{
        _check_schema_response, build_schema_request, set_endorser, sign_and_submit_to_ledger,
    },
};

// consider relocating out of primitive
pub async fn publish_schema(
    wallet_handle: WalletHandle,
    pool_handle: PoolHandle,
    submitter_did: &str,
    schema_json: &str,
    endorser_did: Option<String>,
) -> VcxResult<()> {
    trace!(
        "publish_schema >>> submitter_did: {:?}, schema_json: {:?}, endorser_did: {:?}",
        submitter_did,
        schema_json,
        endorser_did
    );

    if settings::indy_mocks_enabled() {
        debug!("publish_schema >>> mocked success");
        return Ok(());
    }

    let mut request = build_schema_request(submitter_did, schema_json).await?;
    if let Some(endorser_did) = endorser_did {
        request = set_endorser(wallet_handle, submitter_did, &request, &endorser_did).await?;
    }
    let response = sign_and_submit_to_ledger(wallet_handle, pool_handle, submitter_did, &request).await?;
    _check_schema_response(&response)?;

    Ok(())
}

// consider relocating out of primitive
pub async fn libindy_issuer_create_schema(
    issuer_did: &str,
    name: &str,
    version: &str,
    attrs: &str,
) -> VcxResult<(String, String)> {
    trace!(
        "libindy_issuer_create_schema >>> issuer_did: {}, name: {}, version: {}, attrs: {}",
        issuer_did,
        name,
        version,
        attrs
    );

    let attrs = serde_json::from_str::<AttributeNames>(attrs)?;

    let res = Locator::instance().issuer_controller.create_schema(
        DidValue(issuer_did.into()),
        name.into(),
        version.into(),
        attrs,
    )?;

    Ok(res)
}
