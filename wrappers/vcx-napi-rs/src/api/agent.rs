use crate::error::to_napi_err;
use napi_derive::napi;
use vcx::api_vcx::api_handle::mediated_connection;

#[napi]
pub fn generate_public_invitation(public_did: String, label: String) -> napi::Result<String> {
    mediated_connection::generate_public_invitation(&public_did, &label).map_err(to_napi_err)
}
