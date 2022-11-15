use serde_json::{Map, Value};
use vdrtools_sys::WalletHandle;

use crate::error::prelude::*;
use crate::global::settings;
use crate::indy::anoncreds::close_search_handle;
use crate::utils;
use crate::utils::constants::{ATTRS, PROOF_REQUESTED_PREDICATES, REQUESTED_ATTRIBUTES};
use crate::utils::mockdata::mock_settings::{get_mock_creds_retrieved_for_proof_request};

pub async fn libindy_prover_create_proof(
    wallet_handle: WalletHandle,
    proof_req_json: &str,
    requested_credentials_json: &str,
    master_secret_id: &str,
    schemas_json: &str,
    credential_defs_json: &str,
    revoc_states_json: Option<&str>,
) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(utils::constants::PROOF_JSON.to_owned());
    }

    let revoc_states_json = revoc_states_json.unwrap_or("{}");
    vdrtools::anoncreds::prover_create_proof(
        wallet_handle,
        proof_req_json,
        requested_credentials_json,
        master_secret_id,
        schemas_json,
        credential_defs_json,
        revoc_states_json,
    )
    .await
    .map_err(VcxError::from)
}

async fn fetch_credentials(search_handle: i32, requested_attributes: Map<String, Value>) -> VcxResult<String> {
    let mut v: Value = json!({});
    for item_referent in requested_attributes.keys() {
        v[ATTRS][item_referent] = serde_json::from_str(
            &vdrtools::anoncreds::prover_fetch_credentials_for_proof_req(search_handle, item_referent, 100).await?,
        )
        .map_err(|_| {
            error!("Invalid Json Parsing of Object Returned from Libindy. Did Libindy change its structure?");
            VcxError::from_msg(
                VcxErrorKind::InvalidConfiguration,
                "Invalid Json Parsing of Object Returned from Libindy. Did Libindy change its structure?",
            )
        })?
    }

    Ok(v.to_string())
}

pub async fn libindy_prover_get_credentials(
    wallet_handle: WalletHandle,
    filter_json: Option<&str>,
) -> VcxResult<String> {
    Ok(vdrtools::anoncreds::prover_get_credentials(wallet_handle, filter_json)
        .await
        .map_err(|ec| {
            error!("Getting prover credentials failed.");
            ec
        })?)
}

pub async fn libindy_prover_get_credentials_for_proof_req(
    wallet_handle: WalletHandle,
    proof_req: &str,
) -> VcxResult<String> {
    trace!(
        "libindy_prover_get_credentials_for_proof_req >>> proof_req: {}",
        proof_req
    );
    match get_mock_creds_retrieved_for_proof_request() {
        None => {}
        Some(mocked_creds) => {
            warn!("get_mock_creds_retrieved_for_proof_request  returning mocked response");
            return Ok(mocked_creds);
        }
    }

    // this may be too redundant since Prover::search_credentials will validate the proof reqeuest already.
    let proof_request_json: Map<String, Value> = serde_json::from_str(proof_req).map_err(|err| {
        VcxError::from_msg(
            VcxErrorKind::InvalidProofRequest,
            format!("Cannot deserialize ProofRequest: {:?}", err),
        )
    })?;

    // since the search_credentials_for_proof request validates that the proof_req is properly structured, this get()
    // fn should never fail, unless libindy changes their formats.
    let requested_attributes: Option<Map<String, Value>> = proof_request_json.get(REQUESTED_ATTRIBUTES).and_then(|v| {
        serde_json::from_value(v.clone())
            .map_err(|_| {
                error!("Invalid Json Parsing of Requested Attributes Retrieved From Libindy. Did Libindy change its structure?");
            })
            .ok()
    });

    let requested_predicates: Option<Map<String, Value>> = proof_request_json.get(PROOF_REQUESTED_PREDICATES).and_then(|v| {
        serde_json::from_value(v.clone())
            .map_err(|_| {
                error!("Invalid Json Parsing of Requested Predicates Retrieved From Libindy. Did Libindy change its structure?");
            })
            .ok()
    });

    // handle special case of "empty because json is bad" vs "empty because no attributes sepected"
    if requested_attributes == None && requested_predicates == None {
        return Err(VcxError::from_msg(
            VcxErrorKind::InvalidAttributesStructure,
            "Invalid Json Parsing of Requested Attributes Retrieved From Libindy",
        ));
    }

    let mut fetch_attrs: Map<String, Value> = match requested_attributes {
        Some(attrs) => attrs.clone(),
        None => Map::new(),
    };
    match requested_predicates {
        Some(attrs) => fetch_attrs.extend(attrs),
        None => (),
    }
    if !fetch_attrs.is_empty() {
        let search_handle =
            vdrtools::anoncreds::prover_search_credentials_for_proof_req(wallet_handle, proof_req, None)
                .await
                .map_err(|ec| {
                    error!("Opening Indy Search for Credentials Failed");
                    ec
                })?;
        let creds: String = fetch_credentials(search_handle, fetch_attrs).await?;

        // should an error on closing a search handle throw an error, or just a warning?
        // for now we're are just outputting to the user that there is an issue, and continuing on.
        let _ = close_search_handle(search_handle);
        Ok(creds)
    } else {
        Ok("{}".to_string())
    }
}
