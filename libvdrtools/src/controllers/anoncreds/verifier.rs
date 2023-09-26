use std::sync::Arc;

use indy_api_types::errors::prelude::*;
use log::trace;

use crate::{
    domain::anoncreds::{
        credential_definition::{cred_defs_map_to_cred_defs_v1_map, CredentialDefinitions},
        proof::Proof,
        proof_request::ProofRequest,
        revocation_registry::{rev_regs_map_to_rev_regs_local_map, RevocationRegistries},
        revocation_registry_definition::{
            rev_reg_defs_map_to_rev_reg_defs_v1_map, RevocationRegistryDefinitions,
        },
        schema::{schemas_map_to_schemas_v1_map, Schemas},
    },
    services::VerifierService,
};

pub struct VerifierController {
    verifier_service: Arc<VerifierService>,
}

impl VerifierController {
    pub(crate) fn new(verifier_service: Arc<VerifierService>) -> VerifierController {
        VerifierController { verifier_service }
    }

    /// Verifies a proof (of multiple credential).
    /// All required schemas, public keys and revocation registries must be provided.
    ///
    /// IMPORTANT: You must use *_id's (`schema_id`, `cred_def_id`, `rev_reg_id`) listed in
    /// `proof[identifiers]` as the keys for corresponding `schemas_json`,
    /// `credential_defs_json`, `rev_reg_defs_json`, `rev_regs_json` objects.
    ///
    /// #Params
    /// wallet_handle: wallet handle (created by open_wallet).

    /// proof_request_json: proof request json
    ///     {
    ///         "name": string,
    ///         "version": string,
    ///         "nonce": string, - a decimal number represented as a string (use
    /// `indy_generate_nonce` function to generate 80-bit number)         "requested_attributes"
    /// : { // set of requested attributes              "<attr_referent>": <attr_info>, // see
    /// below              ...,
    ///         },
    ///         "requested_predicates": { // set of requested predicates
    ///              "<predicate_referent>": <predicate_info>, // see below
    ///              ...,
    ///          },
    ///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
    ///                        // If specified prover must proof non-revocation
    ///                        // for date in this interval for each attribute
    ///                        // (can be overridden on attribute level)
    ///         "ver": Optional<str>  - proof request version:
    ///             - omit or "1.0" to use unqualified identifiers for restrictions
    ///             - "2.0" to use fully qualified identifiers for restrictions
    ///     }
    /// proof_json: created for request proof json
    ///     {
    ///         "requested_proof": {
    ///             "revealed_attrs": {
    ///                 "requested_attr1_id": {sub_proof_index: number, raw: string, encoded:
    /// string}, // NOTE: check that `encoded` value match to `raw` value on application level
    ///                 "requested_attr4_id": {sub_proof_index: number: string, encoded: string}, //
    /// NOTE: check that `encoded` value match to `raw` value on application level             
    /// },             "revealed_attr_groups": {
    ///                 "requested_attr5_id": {
    ///                     "sub_proof_index": number,
    ///                     "values": {
    ///                         "attribute_name": {
    ///                             "raw": string,
    ///                             "encoded": string
    ///                         }
    ///                     }, // NOTE: check that `encoded` value match to `raw` value on
    /// application level                 }
    ///             },
    ///             "unrevealed_attrs": {
    ///                 "requested_attr3_id": {sub_proof_index: number}
    ///             },
    ///             "self_attested_attrs": {
    ///                 "requested_attr2_id": self_attested_value,
    ///             },
    ///             "requested_predicates": {
    ///                 "requested_predicate_1_referent": {sub_proof_index: int},
    ///                 "requested_predicate_2_referent": {sub_proof_index: int},
    ///             }
    ///         }
    ///         "proof": {
    ///             "proofs": [ <credential_proof>, <credential_proof>, <credential_proof> ],
    ///             "aggregated_proof": <aggregated_proof>
    ///         }
    ///         "identifiers": [{schema_id, cred_def_id, Optional<rev_reg_id>, Optional<timestamp>}]
    ///     }
    /// schemas_json: all schemas participating in the proof
    ///     {
    ///         <schema1_id>: <schema1>,
    ///         <schema2_id>: <schema2>,
    ///         <schema3_id>: <schema3>,
    ///     }
    /// credential_defs_json: all credential definitions participating in the proof
    ///     {
    ///         "cred_def1_id": <credential_def1>,
    ///         "cred_def2_id": <credential_def2>,
    ///         "cred_def3_id": <credential_def3>,
    ///     }
    /// rev_reg_defs_json: all revocation registry definitions participating in the proof
    ///     {
    ///         "rev_reg_def1_id": <rev_reg_def1>,
    ///         "rev_reg_def2_id": <rev_reg_def2>,
    ///         "rev_reg_def3_id": <rev_reg_def3>,
    ///     }
    /// rev_regs_json: all revocation registries participating in the proof
    ///     {
    ///         "rev_reg_def1_id": {
    ///             "timestamp1": <rev_reg1>,
    ///             "timestamp2": <rev_reg2>,
    ///         },
    ///         "rev_reg_def2_id": {
    ///             "timestamp3": <rev_reg3>
    ///         },
    ///         "rev_reg_def3_id": {
    ///             "timestamp4": <rev_reg4>
    ///         },
    ///     }
    /// where
    /// attr_referent: Proof-request local identifier of requested attribute
    /// attr_info: Describes requested attribute
    ///     {
    ///         "name": Optional<string>, // attribute name, (case insensitive and ignore spaces)
    ///         "names": Optional<[string, string]>, // attribute names, (case insensitive and
    /// ignore spaces)                                              // NOTE: should either be
    /// "name" or "names", not both and not none of them.                                       
    /// // Use "names" to specify several attributes that have to match a single credential.
    ///         "restrictions": Optional<wql query>, // see below
    ///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
    ///                        // If specified prover must proof non-revocation
    ///                        // for date in this interval this attribute
    ///                        // (overrides proof level interval)
    ///     }
    /// predicate_referent: Proof-request local identifier of requested attribute predicate
    /// predicate_info: Describes requested attribute predicate
    ///     {
    ///         "name": attribute name, (case insensitive and ignore spaces)
    ///         "p_type": predicate type (">=", ">", "<=", "<")
    ///         "p_value": predicate value
    ///         "restrictions": Optional<wql query>, // see below
    ///         "non_revoked": Optional<<non_revoc_interval>>, // see below,
    ///                        // If specified prover must proof non-revocation
    ///                        // for date in this interval this attribute
    ///                        // (overrides proof level interval)
    ///     }
    /// non_revoc_interval: Defines non-revocation interval
    ///     {
    ///         "from": Optional<int>, // timestamp of interval beginning
    ///         "to": Optional<int>, // timestamp of interval ending
    ///     }
    /// where wql query: indy-sdk/docs/design/011-wallet-query-language/README.md
    ///     The list of allowed keys that can be combine into complex queries.
    ///         "schema_id": <credential schema id>,
    ///         "schema_issuer_did": <credential schema issuer did>,
    ///         "schema_name": <credential schema name>,
    ///         "schema_version": <credential schema version>,
    ///         "issuer_did": <credential issuer did>,
    ///         "cred_def_id": <credential definition id>,
    ///         "rev_reg_id": <credential revocation registry id>, // "None" as string if not
    /// present         // the following keys can be used for every `attribute name` in
    /// credential.         "attr::<attribute name>::marker": "1", - to filter based on
    /// existence of a specific attribute         "attr::<attribute name>::value": <attribute
    /// raw value>, - to filter based on value of a specific attribute
    ///
    ///
    /// #Returns
    /// valid: true - if signature is valid, false - otherwise
    ///
    /// #Errors
    /// Anoncreds*
    /// Common*
    /// Wallet*
    pub fn verify_proof(
        &self,
        proof_req: ProofRequest,
        proof: Proof,
        schemas: Schemas,
        cred_defs: CredentialDefinitions,
        rev_reg_defs: RevocationRegistryDefinitions,
        rev_regs: RevocationRegistries,
    ) -> IndyResult<bool> {
        trace!(
            "verify_proof > proof_req {:?} proof {:?} schemas {:?} cred_defs {:?} rev_reg_defs \
             {:?} rev_regs {:?}",
            proof_req,
            proof,
            schemas,
            cred_defs,
            rev_reg_defs,
            rev_regs
        );

        let schemas = schemas_map_to_schemas_v1_map(schemas);
        let cred_defs = cred_defs_map_to_cred_defs_v1_map(cred_defs);
        let rev_reg_defs = rev_reg_defs_map_to_rev_reg_defs_v1_map(rev_reg_defs);
        let rev_regs = rev_regs_map_to_rev_regs_local_map(rev_regs);

        let valid = self.verifier_service.verify(
            &proof,
            &proof_req.value(),
            &schemas,
            &cred_defs,
            &rev_reg_defs,
            &rev_regs,
        )?;

        let res = Ok(valid);
        trace!("verify_proof < {:?}", res);
        res
    }

    ///  Generates 80-bit numbers that can be used as a nonce for proof request.
    ///
    /// #Params

    ///
    /// #Returns
    /// nonce: generated number as a string
    pub fn generate_nonce(&self) -> IndyResult<String> {
        trace!("generate_nonce >");

        let nonce = self
            .verifier_service
            .generate_nonce()?
            .to_dec()
            .to_indy(IndyErrorKind::InvalidState, "Cannot serialize Nonce")?;

        let res = Ok(nonce);
        trace!("generate_nonce < {:?}", res);
        res
    }
}
