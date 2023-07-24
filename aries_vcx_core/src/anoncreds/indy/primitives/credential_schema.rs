use crate::errors::error::VcxCoreResult;
use log::trace;
use vdrtools::domain::anoncreds::schema::AttributeNames;
use vdrtools::domain::crypto::did::DidValue;
use vdrtools::Locator;

// consider relocating out of primitive
pub async fn libindy_issuer_create_schema(
    issuer_did: &str,
    name: &str,
    version: &str,
    attrs: &str,
) -> VcxCoreResult<(String, String)> {
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
