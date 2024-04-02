mod prover_internal;

use std::collections::HashMap;

use anoncreds_types::data_types::messages::{
    cred_selection::SelectedCredentials, pres_request::PresentationRequest,
    presentation::Presentation,
};
use aries_vcx_core::{
    anoncreds::base_anoncreds::BaseAnonCreds, ledger::base_ledger::AnoncredsLedgerRead,
};
use aries_vcx_wallet::wallet::base_wallet::BaseWallet;

use crate::{
    common::proofs::prover::prover_internal::{
        build_cred_defs_json_prover, build_requested_credentials_json, build_rev_states_json,
        build_schemas_json_prover, credential_def_identifiers,
    },
    errors::error::prelude::*,
    global::settings,
};

pub async fn generate_indy_proof(
    wallet: &impl BaseWallet,
    ledger: &impl AnoncredsLedgerRead,
    anoncreds: &impl BaseAnonCreds,
    credentials: &SelectedCredentials,
    self_attested_attrs: HashMap<String, String>,
    proof_req_data_json: PresentationRequest,
) -> VcxResult<Presentation> {
    trace!(
        "generate_indy_proof >>> credentials: {:?}, self_attested_attrs: {:?}",
        secret!(&credentials),
        secret!(&self_attested_attrs)
    );

    let mut credentials_identifiers =
        credential_def_identifiers(credentials, &proof_req_data_json)?;

    let revoc_states_json =
        build_rev_states_json(ledger, anoncreds, &mut credentials_identifiers).await?;
    let requested_credentials = build_requested_credentials_json(
        &credentials_identifiers,
        self_attested_attrs,
        &proof_req_data_json,
    )?;

    let schemas_json = build_schemas_json_prover(ledger, &credentials_identifiers).await?;
    let credential_defs_json =
        build_cred_defs_json_prover(ledger, &credentials_identifiers).await?;

    anoncreds
        .prover_create_proof(
            wallet,
            proof_req_data_json,
            requested_credentials,
            &settings::DEFAULT_LINK_SECRET_ALIAS.to_string(),
            schemas_json,
            credential_defs_json,
            Some(revoc_states_json),
        )
        .await
        .map_err(Into::into)
}
