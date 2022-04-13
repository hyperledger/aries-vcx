#![cfg_attr(feature = "fatal_warnings", deny(warnings))]

#[macro_use]
extern crate derivative;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate log;

#[macro_use]
mod utils;

mod demos {
    use serde_json::Value;
    use super::*;

    use utils::{
        did,
        crypto,
        vdr,
        Setup,
        anoncreds,
        constants::*,
        domain::{
            anoncreds::{
                schema::{Schema, SchemaV1},
                credential_definition::CredentialDefinition,
                credential::CredentialInfo,
                credential_offer::CredentialOffer,
                credential_for_proof_request::CredentialsForProofRequest,
                proof::Proof,
            },
        },
        rand_utils::get_rand_string,
        wallet,
        ledger,
    };

    #[cfg(feature = "cheqd")]
    use utils::{
        cheqd_setup::CheqdSetup,
        cheqd_keys,
        cheqd_ledger,
        environment,
        vdr::VDR,
    };

    const INDY_NAMESPACE_1: &'static str = "indytest";
    const INDY_NAMESPACE_2: &'static str = "indyfirst";
    const INDY_NAMESPACE_3: &'static str = "indysecond";
    const INDY_NAMESPACE_4: &'static str = "indythird";

    #[cfg(feature = "cheqd")]
    const CHEQD_NAMESPACE_1: &'static str = "testnet";
    #[cfg(feature = "cheqd")]
    const CHEQD_NAMESPACE_2: &'static str = "cheqdsecond";

    fn indy_namespace_list() -> String {
        json!(vec![INDY_NAMESPACE_1, INDY_NAMESPACE_2, INDY_NAMESPACE_3, INDY_NAMESPACE_4]).to_string()
    }

    #[cfg(feature = "cheqd")]
    fn cheqd_namespace_list() -> String {
        json!(vec![CHEQD_NAMESPACE_1, CHEQD_NAMESPACE_2]).to_string()
    }

    #[cfg(feature = "cheqd")]
    #[cfg(feature = "local_nodes_pool")]
    #[test]
    fn vdr_open_indy_and_cheqd_pools() {
        let setup = CheqdSetup::new();

        // 1. open VDR with indy and cheqd pools
        let mut vdr_builder = vdr::vdr_builder_create().unwrap();
        vdr::vdr_builder_register_indy_ledger(&mut vdr_builder,
                                              &indy_namespace_list(),
                                              &vdr::local_genesis_txn(), None).unwrap();
        vdr::vdr_builder_register_cheqd_ledger(&mut vdr_builder,
                                               &cheqd_namespace_list(),
                                               &environment::cheqd_test_chain_id(),
                                               &environment::cheqd_test_pool_ip()).unwrap();

        let vdr = vdr::vdr_builder_finalize(vdr_builder).unwrap();

        // 2. ping pool passing only sub part of specified indy and cheqd namespaces
        let namespace_sub_list = json!(vec![
            INDY_NAMESPACE_2,
            INDY_NAMESPACE_4,
            CHEQD_NAMESPACE_1
        ]).to_string();
        vdr::ping(&vdr, &namespace_sub_list).unwrap();

        // 3. request predefined DID from indy pool
        let (trustee_did, _) = did::create_store_predefined_trustee_did(setup.wallet_handle, Some(INDY_NAMESPACE_1)).unwrap();
        let _did_doc = vdr::resolve_did(&vdr, &trustee_did).unwrap();
        println!("_did_doc {}", _did_doc);

        // 4. request account balance from cheqd pool
        let query = cheqd_ledger::bank::bank_build_query_balance(&setup.account_id, &environment::cheqd_denom()).unwrap();
        let response = vdr::submit_query(&vdr, CHEQD_NAMESPACE_1, &query).unwrap();
        let _balance = cheqd_ledger::bank::parse_query_balance_resp(&response).unwrap();
        println!("_balance {}", _balance);

        vdr::cleanup(vdr).unwrap();
    }

    fn _vdr_indy_schema_demo(setup: &Setup, taa_config: Option<&str>) {
        let (trustee_did, trustee_verkey) = did::create_store_predefined_trustee_did(setup.wallet_handle, Some(INDY_NAMESPACE_1)).unwrap();

        // 1. open VDR with indy pool
        let mut vdr_builder = vdr::vdr_builder_create().unwrap();
        vdr::vdr_builder_register_indy_ledger(&mut vdr_builder,
                                              &indy_namespace_list(),
                                              &vdr::local_genesis_txn(),
                                              taa_config).unwrap();
        let vdr = vdr::vdr_builder_finalize(vdr_builder).unwrap();

        vdr::ping(&vdr, &indy_namespace_list()).unwrap();

        // 2. prepare Schema transaction
        let schema_name = get_rand_string(7);
        let (schema_id, schema_json) =
            anoncreds::issuer_create_schema(&trustee_did, &schema_name, SCHEMA_VERSION, GVT_SCHEMA_ATTRIBUTES).unwrap();

        let (namespace, signature_spec, txn_bytes, bytes_to_sign, _endorsement_spec) =
            vdr::prepare_schema(&vdr, &schema_json, &trustee_did, None).unwrap();
        // TODO VE-3079 check endorsement spec

        // 3. sign transaction
        let signature = crypto::sign(setup.wallet_handle, &trustee_verkey, &bytes_to_sign).unwrap();

        // 4. submit transaction
        let _response = vdr::submit_txn(&vdr, &namespace, &signature_spec, &txn_bytes, &signature, None).unwrap();
        // TODO VE-3079 compare response vs get result

        // 5. resolve schema and validate
        ::std::thread::sleep(::std::time::Duration::from_secs(5));
        let schema = vdr::resolve_schema(&vdr, &schema_id).unwrap();
        validate_schema(&schema, &schema_name, SCHEMA_VERSION);

        vdr::cleanup(vdr).unwrap();
    }

    #[cfg(feature = "local_nodes_pool")]
    #[test]
    fn vdr_indy_schema_demo() {
        let setup = Setup::trustee();

        _vdr_indy_schema_demo(&setup, None);
    }

    #[cfg(feature = "local_nodes_pool")]
    #[test]
    fn vdr_indy_schema_demo_with_taa() {
        let setup = Setup::trustee();// use old pool to set up TAA

        let (_, aml_label, _, _) = ledger::taa::set_aml(setup.pool_handle, setup.wallet_handle, &setup.did);
        let (_, _, taa_digest, _) = ledger::taa::set_taa(setup.pool_handle, setup.wallet_handle, &setup.did);

        let taa_config = json!({
            "taa_digest": taa_digest,
            "acc_mech_type": aml_label,
            "time": time::get_time().sec as u64,
        }).to_string();

        _vdr_indy_schema_demo(&setup, Some(&taa_config));

        ledger::taa::disable_taa(setup.pool_handle, setup.wallet_handle, &setup.did);
    }

    #[cfg(feature = "local_nodes_pool")]
    #[test]
    fn vdr_indy_demo_with_endorsement() {
        let setup = Setup::endorser();

        let (trustee_did, trustee_verkey) = did::create_store_predefined_trustee_did(setup.wallet_handle, Some(INDY_NAMESPACE_1)).unwrap();
        let (issuer_did, issuer_verkey) = did::create_my_did(setup.wallet_handle, &json!({"method_name": INDY_NAMESPACE_1}).to_string()).unwrap();
        let endorser_did = setup.did.clone();

        // 1. open VDR with indy pool
        let mut vdr_builder = vdr::vdr_builder_create().unwrap();
        vdr::vdr_builder_register_indy_ledger(&mut vdr_builder,
                                              &indy_namespace_list(),
                                              &vdr::local_genesis_txn(),
                                              None).unwrap();
        let vdr = vdr::vdr_builder_finalize(vdr_builder).unwrap();
        vdr::ping(&vdr, &indy_namespace_list()).unwrap();

        // 2. Trustee publish Issuer DID
        // 2.1 Trustee prepare DID transaction
        let did_txn_params = json!({"dest": issuer_did, "verkey": issuer_verkey}).to_string();
       let (namespace, txn_bytes, signature_spec, bytes_to_sign, _) =
                 vdr::prepare_did(&vdr, &did_txn_params, &trustee_did, None).unwrap();

        // 2.2. Trustee sign and submit DID transaction
        let signature = crypto::sign(setup.wallet_handle, &trustee_verkey, &bytes_to_sign).unwrap();
        vdr::submit_txn(&vdr, &namespace, &txn_bytes, &signature_spec, &signature, None).unwrap();

        // 3. Issuer prepare Schema transaction
        let schema_name = get_rand_string(7);
        let (schema_id, schema_json) =
            anoncreds::issuer_create_schema(&issuer_did, &schema_name, SCHEMA_VERSION, GVT_SCHEMA_ATTRIBUTES).unwrap();

        let (namespace, txn_bytes, signature_spec, bytes_to_sign, _endorsement_spec) =
            vdr::prepare_schema(&vdr, &schema_json, &issuer_did, Some(&endorser_did)).unwrap();
        // TODO VE-3079 check endorsement spec

        // 3. Issuer sign transaction
        let signature = crypto::sign(setup.wallet_handle, &issuer_verkey, &bytes_to_sign).unwrap();

        // 4. Trustee sign prepared transaction
        let endorsement_data = json!({"did": endorser_did}).to_string();
        let endorsement =
            vdr::indy_endorse(setup.wallet_handle,
                              &endorsement_data,
                              &signature_spec,
                              &bytes_to_sign).unwrap();

        // 5. Issuer submit transaction
        let _response = vdr::submit_txn(&vdr, &namespace, &txn_bytes, &signature_spec, &signature, Some(&endorsement)).unwrap();
        // TODO VE-3079 compare response vs get result

        // 6. resolve schema and validate
        ::std::thread::sleep(::std::time::Duration::from_secs(5));
        let schema = vdr::resolve_schema(&vdr, &schema_id).unwrap();
        validate_schema(&schema, &schema_name, SCHEMA_VERSION);

        vdr::cleanup(vdr).unwrap();
    }

    #[cfg(feature = "local_nodes_pool")]
    #[test]
    fn vdr_indy_anoncreds_demo() {
        Setup::wallet();

        let (trustee_wallet_handle, trustee_wallet_config) = wallet::create_and_open_default_wallet("vdr_indy_anoncreds_demo_trustee").unwrap();
        let (issuer_wallet_handle, issuer_wallet_config) = wallet::create_and_open_default_wallet("vdr_indy_anoncreds_demo_issuer").unwrap();
        let (holder_wallet_handle, holder_wallet_config) = wallet::create_and_open_default_wallet("vdr_indy_anoncreds_demo_holder").unwrap();

        let (trustee_did, trustee_verkey) = did::create_store_predefined_trustee_did(trustee_wallet_handle, Some(INDY_NAMESPACE_1)).unwrap();
        let (issuer_did, issuer_verkey) = did::create_my_did(issuer_wallet_handle, &json!({"method_name": INDY_NAMESPACE_1}).to_string()).unwrap();
        let (holder_did, _) = did::create_my_did(holder_wallet_handle, &json!({"method_name": INDY_NAMESPACE_1}).to_string()).unwrap();

        // 0. open VDR with indy pool
        let mut vdr_builder = vdr::vdr_builder_create().unwrap();
        vdr::vdr_builder_register_indy_ledger(&mut vdr_builder,
                                              &indy_namespace_list(),
                                              &vdr::local_genesis_txn(),
                                              None).unwrap();
        let vdr = vdr::vdr_builder_finalize(vdr_builder).unwrap();
        let ping_status = vdr::ping(&vdr, &indy_namespace_list()).unwrap();
        let ping_status = serde_json::from_str::<Value>(&ping_status).unwrap();
        let ping_statuses_map = ping_status.as_object().unwrap();
        for status in ping_statuses_map.values() {
            assert_eq!(status["code"].as_str().unwrap(), "SUCCESS")
        }

        // 1.1 Trustee publish Issuer DID to the Ledger
        // 1.2 Trustee prepare DID transaction
        let did_txn_params = json!({
            "dest": issuer_did,
            "verkey": issuer_verkey,
            "role": "TRUSTEE",
        }).to_string();
        let (namespace, txn_bytes, signature_spec, bytes_to_sign, _) =
            vdr::prepare_did(&vdr, &did_txn_params, &trustee_did, None).unwrap();

        // 1.3. Trustee sign and submit DID transaction
        let signature = crypto::sign(trustee_wallet_handle, &trustee_verkey, &bytes_to_sign).unwrap();
        vdr::submit_txn(&vdr, &namespace, &txn_bytes, &signature_spec, &signature, None).unwrap();

        // 2.1 Issuer create Schema
        let schema_name = get_rand_string(7);
        let (schema_id, schema_json) = anoncreds::issuer_create_schema(&issuer_did, &schema_name, SCHEMA_VERSION, GVT_SCHEMA_ATTRIBUTES).unwrap();

        // 2.2 Issuer prepare Schema transaction
        let (namespace, signature_spec, txn_bytes, bytes_to_sign, _) =
            vdr::prepare_schema(&vdr, &schema_json, &issuer_did, None).unwrap();

        // 2.3. Issuer sign and submit Schema transaction
        let signature = crypto::sign(issuer_wallet_handle, &issuer_verkey, &bytes_to_sign).unwrap();
        vdr::submit_txn(&vdr, &namespace, &signature_spec, &txn_bytes, &signature, None).unwrap();

        // 3.1 Issuer resolve Schema
        ::std::thread::sleep(::std::time::Duration::from_secs(5));
        let schema = vdr::resolve_schema(&vdr, &schema_id).unwrap();

        // 3.2 Issuer create CredDef
        let (cred_def_id, cred_def_json) = anoncreds::issuer_create_credential_definition(
            issuer_wallet_handle,
            &issuer_did,
            &schema,
            TAG_1,
            None,
            None,
        )
            .unwrap();

        // 3.3 Issuer prepare CredDef transaction
        let (namespace, signature_spec, txn_bytes, bytes_to_sign, _) =
            vdr::prepare_cred_def(&vdr, &cred_def_json, &issuer_did, None).unwrap();

        // 3.4. Issuer sign and submit CredDef transaction
        let signature = crypto::sign(issuer_wallet_handle, &issuer_verkey, &bytes_to_sign).unwrap();
        vdr::submit_txn(&vdr, &namespace, &signature_spec, &txn_bytes, &signature, None).unwrap();

        // 4. Issuer creates CredentialOffer
        // Issuer creates Credential Offer
        let cred_offer_json = anoncreds::issuer_create_credential_offer(issuer_wallet_handle, &cred_def_id).unwrap();

        // 5. Holder creates Credential Request
        // 5.1 Holder create MasterSecret
        anoncreds::prover_create_master_secret(holder_wallet_handle, anoncreds::COMMON_MASTER_SECRET).unwrap();

        // 5.1 Holder resolve CredDef
        let cred_offer: CredentialOffer = serde_json::from_str(&cred_offer_json).unwrap();
        let holder_cred_def = vdr::resolve_cred_def(&vdr, &cred_offer.cred_def_id.0).unwrap();

        // 5.2 Holder creates CredentialRequest
        let (cred_req_json, cred_req_metadata_json) = anoncreds::prover_create_credential_req(
            holder_wallet_handle,
            &holder_did,
            &cred_offer_json,
            &holder_cred_def,
            anoncreds::COMMON_MASTER_SECRET,
        ).unwrap();

        // 6. Issuer sign Credential
        let (cred_json, _, _) = anoncreds::issuer_create_credential(
            issuer_wallet_handle,
            &cred_offer_json,
            &cred_req_json,
            &anoncreds::gvt_credential_values_json(),
            None,
            None,
        ).unwrap();

        // 7. Holder store Credential
        anoncreds::prover_store_credential(
            holder_wallet_handle,
            anoncreds::CREDENTIAL1_ID,
            &cred_req_metadata_json,
            &cred_json,
            &holder_cred_def,
            None,
        ).unwrap();

        // 8. Verifying Holder's Credential
        let referent = "attr1_referent";
        let proof_request = json!({
           "nonce":"123432421212",
           "name":"proof_req",
           "version":"0.1",
           "requested_attributes": {
               referent: {
                   "name":"name",
               }
           },
           "requested_predicates": {},
           "non_revoked": {},
           "ver": "2.0"
        }).to_string();

        // 8.1 Holder gets credentials for proof request
        let credential_for_proof_req = anoncreds::prover_get_credentials_for_proof_req(holder_wallet_handle, &proof_request).unwrap();
        let credentials: CredentialsForProofRequest = serde_json::from_str(&credential_for_proof_req).unwrap();

        let credential_to_use: CredentialInfo = credentials.attrs[referent][0].clone().cred_info;

        // 8.2 Holder builds requested credentials
        let requested_credentials_json = json!({
            "self_attested_attributes": {},
            "requested_attributes": {
                referent: {
                    "cred_id": credential_to_use.referent,
                    "revealed":true
                }
            },
            "requested_predicates": {}
        }).to_string();

        // 8.3 Holder resolves Schema and CredDef for selected credential
        let holder_schema = vdr::resolve_schema(&vdr, &credential_to_use.schema_id.0).unwrap();
        let holder_cred_def = vdr::resolve_cred_def(&vdr, &credential_to_use.cred_def_id.0).unwrap();

        let schemas_json = json!({
            credential_to_use.schema_id.0: serde_json::from_str::<Schema>(&holder_schema).unwrap()
        }).to_string();

        let cred_defs_json = json!({
            credential_to_use.cred_def_id.0: serde_json::from_str::<CredentialDefinition>(&holder_cred_def).unwrap()
        }).to_string();

        // 8.4 Holder creates Proof
        let proof_json = anoncreds::prover_create_proof(
            holder_wallet_handle,
            &proof_request,
            &requested_credentials_json,
            anoncreds::COMMON_MASTER_SECRET,
            &schemas_json,
            &cred_defs_json,
            "{}",
        ).unwrap();

        // 9 Verifier verifies proof
        let proof: Proof = serde_json::from_str(&proof_json).unwrap();
        let identifier = proof.identifiers[0].clone();

        // 9.1 Verifier resolves Schema and CredDef for selected credential
        let verifier_schema = vdr::resolve_schema(&vdr, &identifier.schema_id.0).unwrap();
        let verifier_cred_def = vdr::resolve_cred_def(&vdr, &identifier.cred_def_id.0).unwrap();

        let schemas_json = json!({
            identifier.schema_id.0.clone(): serde_json::from_str::<Schema>(&verifier_schema).unwrap()
        }).to_string();

        let cred_defs_json = json!({
            identifier.cred_def_id.0.clone(): serde_json::from_str::<CredentialDefinition>(&verifier_cred_def).unwrap()
        }).to_string();

        // 9.2 Verifier verifies Proof
        anoncreds::verifier_verify_proof(
            &proof_request,
            &proof_json,
            &schemas_json,
            &cred_defs_json,
            &"{}",
            &"{}",
        ).unwrap();

        // 10. Clean up
        vdr::cleanup(vdr).unwrap();
        wallet::close_and_delete_wallet(trustee_wallet_handle, &trustee_wallet_config).unwrap();
        wallet::close_and_delete_wallet(issuer_wallet_handle, &issuer_wallet_config).unwrap();
        wallet::close_and_delete_wallet(holder_wallet_handle, &holder_wallet_config).unwrap();
    }

    #[cfg(feature = "local_nodes_pool")]
    #[test]
    fn vdr_indy_submit_txn_works_for_failed_txn() {
        let setup = Setup::did_fully_qualified();
        let mut vdr_builder = vdr::vdr_builder_create().unwrap();
        let namespace_list_of_default = json!(vec![DEFAULT_METHOD_NAME]).to_string();
        vdr::vdr_builder_register_indy_ledger(&mut vdr_builder, &namespace_list_of_default, &vdr::local_genesis_txn(), None).unwrap();
        let vdr = vdr::vdr_builder_finalize(vdr_builder).unwrap();
        vdr::ping(&vdr, &namespace_list_of_default).unwrap();

        let did_txn_params = json!({"dest": setup.did, "verkey": setup.verkey}).to_string();
        let (namespace, txn_bytes, signature_spec, bytes_to_sign, _) =
            vdr::prepare_did(&vdr, &did_txn_params, &setup.did, None).unwrap();
        let signature = crypto::sign(setup.wallet_handle, &setup.verkey, &bytes_to_sign).unwrap();
        let err = vdr::submit_txn(&vdr, &namespace, &txn_bytes, &signature_spec, &signature, None).unwrap_err();
        assert!(!err.message.contains("no failure reason provided"));
        assert!(err.message.contains("UnauthorizedClientRequest"));
    }

    #[cfg(feature = "cheqd")]
    #[test]
    fn vdr_cheqd_demo_prepare_did() {
        let setup = CheqdSetup::new();

        // 1. open VDR with cheqd pool
        let mut vdr_builder = vdr::vdr_builder_create().unwrap();
        vdr::vdr_builder_register_cheqd_ledger(&mut vdr_builder,
                                               &cheqd_namespace_list(),
                                               &environment::cheqd_test_chain_id(),
                                               &environment::cheqd_test_pool_ip()).unwrap();
        let vdr = vdr::vdr_builder_finalize(vdr_builder).unwrap();
        vdr::ping(&vdr, &cheqd_namespace_list()).unwrap();

        // 2. prepare DID message
        let (did, verkey) = did::create_my_did(setup.wallet_handle, &cheqd_ledger::cheqd::did_info()).unwrap();

        let did_data = json!({"did": did, "verkey": verkey}).to_string();
        let (namespace, txn_bytes, signature_spec, bytes_to_sign, _endorsement_spec) =
            vdr::prepare_did(&vdr, &did_data, &did, None).unwrap();
        // TODO VE-3079 check endorsement spec

        // 3. sign DID message using identy key
        let txn_signature = crypto::sign(setup.wallet_handle, &verkey, &bytes_to_sign).unwrap();

        // 4. prepare endorsment
        let endorsement_data = vdr::prepare_cheqd_endorsement_data(&vdr,
                                                                   setup.wallet_handle,
                                                                   &setup.key_alias,
                                                                   &did,
                                                                   &txn_bytes,
                                                                   &txn_signature,
                                                                   25,
                                                                   "test").unwrap();

        // 5. sign by endorser
        let endorsement = vdr::cheqd_endorse(setup.wallet_handle,
                                             &endorsement_data,
                                             &signature_spec,
                                             &txn_bytes,
                                             &txn_signature).unwrap();

        // 6. submit transaction
        let _response = vdr::submit_txn(&vdr,
                                       &namespace,
                                       &txn_bytes,
                                       &signature_spec,
                                       &txn_signature,
                                       Some(&endorsement)).unwrap();
        // TODO VE-3079 compare response vs get result

        // 7. Resolve DID
        ::std::thread::sleep(::std::time::Duration::from_secs(5));
        let did = vdr::resolve_did(&vdr, &did).unwrap();
        println!("DID: {:?}", did);

        vdr::cleanup(vdr).unwrap();
    }

    #[cfg(feature = "cheqd")]
    #[test]
    fn vdr_cheqd_demo_payment() {
        let setup = CheqdSetup::new();

        let (second_account_id, _) = create_account(&setup);

        // 1. open VDR with cheqd pool
        let mut vdr_builder = vdr::vdr_builder_create().unwrap();
        vdr::vdr_builder_register_cheqd_ledger(&mut vdr_builder,
                                               &cheqd_namespace_list(),
                                               &environment::cheqd_test_chain_id(),
                                               &environment::cheqd_test_pool_ip()).unwrap();
        let vdr = vdr::vdr_builder_finalize(vdr_builder).unwrap();
        vdr::ping(&vdr, &cheqd_namespace_list()).unwrap();

        let query_resp = get_account_balance(&vdr, &setup.account_id);
        println!("Current balance response: {:?}", query_resp);

        // 2. prepare Token Transfer message
        let amount_for_transfer = "100";
        let msg = cheqd_ledger::bank::build_msg_send(&setup.account_id, &second_account_id, amount_for_transfer, &setup.denom).unwrap();

        // 3. build cheqd send tokens transaction and sign
        let (account_number, account_sequence) = setup.get_base_account_number_and_sequence(&setup.account_id).unwrap();
        let txn_bytes = cheqd_ledger::auth::build_tx(
            &setup.pool_alias,
            &setup.pub_key,
            &msg,
            account_number,
            account_sequence,
            300000,
            7500000,
            &setup.denom,
            setup.get_timeout_height(),
            "memo",
        ).unwrap();

        // 4. Sign message
        let signed_txn = cheqd_ledger::auth::sign_tx(setup.wallet_handle, &setup.key_alias, &txn_bytes).unwrap();

        // 5. submit cheqd transaction
        let _response = vdr::submit_raw_txn(&vdr, CHEQD_NAMESPACE_1, &signed_txn).unwrap();
        // TODO VE-3079 compare response vs get result

        // 6. query account balance
        ::std::thread::sleep(::std::time::Duration::from_secs(5));

        let query_resp = get_account_balance(&vdr, &setup.account_id);
        println!("New balance response: {:?}", query_resp);

        let query_resp = get_account_balance(&vdr, &second_account_id);
        println!("New account balance response: {:?}", query_resp);

        let new_balance_response: serde_json::Value = serde_json::from_str(&query_resp).unwrap();
        let new_balance = new_balance_response.as_object().unwrap()
            .get("balance").unwrap().as_object().unwrap()
            .get("amount").unwrap().as_str().unwrap();

        assert_eq!(amount_for_transfer, new_balance);

        vdr::cleanup(vdr).unwrap();
    }

    fn validate_schema(schema: &str, name: &str, version: &str) -> SchemaV1 {
        let schema: SchemaV1 = serde_json::from_str(&schema).unwrap();
        assert_eq!(name, schema.name);
        assert_eq!(version, schema.version);
        schema
    }

    #[cfg(feature = "cheqd")]
    fn create_account(setup: &CheqdSetup) -> (String, String) {
        let alias = get_rand_string(7);
        let key_info = cheqd_keys::add_random(setup.wallet_handle, &alias).unwrap();
        let key_info = serde_json::from_str::<serde_json::Value>(&key_info).unwrap();
        let account_id = key_info["account_id"].as_str().unwrap();
        let pub_key = key_info["pub_key"].as_str().unwrap();
        (account_id.to_string(), pub_key.to_string())
    }

    #[cfg(feature = "cheqd")]
    fn get_account_balance(vdr: &VDR, account_id: &str) -> String {
        let query = cheqd_ledger::bank::bank_build_query_balance(&account_id, &environment::cheqd_denom()).unwrap();
        let response = vdr::submit_query(vdr, CHEQD_NAMESPACE_1, &query).unwrap();
        let response = cheqd_ledger::bank::parse_query_balance_resp(&response).unwrap();
        response
    }
}
