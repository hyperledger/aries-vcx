use std::collections::HashMap;
use std::sync::Arc;

use aries_vcx::handlers::proof_presentation::types::{
    RetrievedCredentialForReferent, RetrievedCredentials, SelectedCredentials,
};
use aries_vcx::handlers::util::PresentationProposalData;
use messages::msg_fields::protocols::present_proof::present::Presentation;
use messages::msg_fields::protocols::present_proof::request::RequestPresentation;
use messages::msg_fields::protocols::present_proof::PresentProof;
use messages::AriesMessage;
use serde_json::Value;

use crate::utils::scenarios::requested_attr_objects;
use crate::utils::test_agent::TestAgent;
use aries_vcx::common::primitives::credential_definition::CredentialDef;
use aries_vcx::common::primitives::revocation_registry::RevocationRegistry;
use aries_vcx::common::proofs::proof_request::PresentationRequestData;
use aries_vcx::common::proofs::proof_request_internal::AttrInfo;
use aries_vcx::handlers::issuance::issuer::Issuer;
use aries_vcx::handlers::proof_presentation::prover::Prover;
use aries_vcx::handlers::proof_presentation::verifier::Verifier;
use aries_vcx::protocols::proof_presentation::prover::state_machine::ProverState;
use aries_vcx::protocols::proof_presentation::verifier::state_machine::VerifierState;
use aries_vcx::protocols::proof_presentation::verifier::verification_status::PresentationVerificationStatus;
use aries_vcx::utils::constants::{DEFAULT_PROOF_NAME, TEST_TAILS_URL};
use aries_vcx_core::ledger::indy::pool::test_utils::get_temp_dir_path;

use super::requested_attrs_address;

pub async fn create_proof_proposal(prover: &mut Prover, cred_def_id: &str) -> AriesMessage {
    let attrs = requested_attr_objects(cred_def_id);
    let mut proposal_data = PresentationProposalData::default();
    for attr in attrs.into_iter() {
        proposal_data.attributes.push(attr);
    }
    let proposal = prover.build_presentation_proposal(proposal_data).await.unwrap();
    assert_eq!(prover.get_state(), ProverState::PresentationProposalSent);
    proposal.into()
}

pub async fn accept_proof_proposal(
    faber: &mut TestAgent,
    verifier: &mut Verifier,
    presentation_proposal: AriesMessage,
) -> AriesMessage {
    verifier
        .process_aries_msg(
            &faber.profile.inject_anoncreds_ledger_read(),
            &faber.profile.inject_anoncreds(),
            presentation_proposal.clone(),
        )
        .await
        .unwrap();
    assert_eq!(verifier.get_state(), VerifierState::PresentationProposalReceived);
    let presentation_proposal = match presentation_proposal {
        AriesMessage::PresentProof(PresentProof::ProposePresentation(presentation_proposal)) => presentation_proposal,
        _ => panic!("Unexpected message"),
    };
    let attrs = presentation_proposal
        .content
        .presentation_proposal
        .attributes
        .into_iter()
        .map(|attr| AttrInfo {
            name: Some(attr.name),
            ..AttrInfo::default()
        })
        .collect();
    let presentation_request_data = PresentationRequestData::create(&faber.profile.inject_anoncreds(), "request-1")
        .await
        .unwrap()
        .set_requested_attributes_as_vec(attrs)
        .unwrap();
    verifier
        .set_presentation_request(presentation_request_data, None)
        .unwrap();
    let presentation_request = verifier.mark_presentation_request_sent().unwrap();
    presentation_request
}

pub async fn reject_proof_proposal(presentation_proposal: &AriesMessage) -> AriesMessage {
    let presentation_proposal = match presentation_proposal {
        AriesMessage::PresentProof(PresentProof::ProposePresentation(proposal)) => proposal,
        _ => panic!("Unexpected message"),
    };
    let mut verifier = Verifier::create_from_proposal("1", presentation_proposal).unwrap();
    assert_eq!(verifier.get_state(), VerifierState::PresentationProposalReceived);
    let message = verifier
        .decline_presentation_proposal("I don't like Fabers") // :(
        .await
        .unwrap();
    assert_eq!(verifier.get_state(), VerifierState::Failed);
    message
}

pub async fn receive_proof_proposal_rejection(prover: &mut Prover, rejection: AriesMessage) {
    assert_eq!(prover.get_state(), ProverState::PresentationProposalSent);
    prover.process_aries_msg(rejection).await.unwrap();
    assert_eq!(prover.get_state(), ProverState::Failed);
}

pub async fn create_proof_request_data(
    faber: &mut TestAgent,
    requested_attrs: &str,
    requested_preds: &str,
    revocation_interval: &str,
    request_name: Option<&str>,
) -> PresentationRequestData {
    PresentationRequestData::create(&faber.profile.inject_anoncreds(), request_name.unwrap_or("name"))
        .await
        .unwrap()
        .set_requested_attributes_as_string(requested_attrs.to_string())
        .unwrap()
        .set_requested_predicates_as_string(requested_preds.to_string())
        .unwrap()
        .set_not_revoked_interval(revocation_interval.to_string())
        .unwrap()
}

pub async fn create_prover_from_request(presentation_request: RequestPresentation) -> Prover {
    Prover::create_from_request(DEFAULT_PROOF_NAME, presentation_request).unwrap()
}

pub async fn create_verifier_from_request_data(presentation_request_data: PresentationRequestData) -> Verifier {
    let mut verifier = Verifier::create_from_request("1".to_string(), &presentation_request_data).unwrap();
    verifier.mark_presentation_request_sent().unwrap();
    verifier
}

pub async fn generate_and_send_proof(
    alice: &mut TestAgent,
    prover: &mut Prover,
    selected_credentials: SelectedCredentials,
) -> Option<AriesMessage> {
    let thread_id = prover.get_thread_id().unwrap();
    info!(
        "generate_and_send_proof >>> generating proof using selected credentials {:?}",
        selected_credentials
    );
    prover
        .generate_presentation(
            &alice.profile.inject_anoncreds_ledger_read(),
            &alice.profile.inject_anoncreds(),
            selected_credentials,
            HashMap::new(),
        )
        .await
        .unwrap();
    assert_eq!(thread_id, prover.get_thread_id().unwrap());
    if ProverState::PresentationPrepared == prover.get_state() {
        info!("generate_and_send_proof :: proof generated, sending proof");
        let message = prover.mark_presentation_sent().unwrap();
        info!("generate_and_send_proof :: proof sent");
        assert_eq!(thread_id, prover.get_thread_id().unwrap());
        Some(message)
    } else {
        None
    }
}

pub async fn verify_proof(faber: &mut TestAgent, verifier: &mut Verifier, presentation: AriesMessage) -> AriesMessage {
    let presentation = match presentation {
        AriesMessage::PresentProof(PresentProof::Presentation(presentation)) => presentation,
        _ => panic!("Unexpected message type"),
    };
    let msg = verifier
        .verify_presentation(
            &faber.profile.inject_anoncreds_ledger_read(),
            &faber.profile.inject_anoncreds(),
            presentation,
        )
        .await
        .unwrap();
    assert_eq!(verifier.get_state(), VerifierState::Finished);
    assert_eq!(
        verifier.get_verification_status(),
        PresentationVerificationStatus::Valid
    );
    msg
}

pub async fn revoke_credential_and_publish_accumulator(
    faber: &mut TestAgent,
    issuer_credential: &Issuer,
    rev_reg: &RevocationRegistry,
) {
    revoke_credential_local(faber, issuer_credential, &rev_reg.rev_reg_id).await;

    rev_reg
        .publish_local_revocations(
            &faber.profile.inject_anoncreds(),
            &faber.profile.inject_anoncreds_ledger_write(),
            &faber.institution_did,
        )
        .await
        .unwrap();
}

pub async fn revoke_credential_local(faber: &mut TestAgent, issuer_credential: &Issuer, rev_reg_id: &str) {
    let ledger = Arc::clone(&faber.profile).inject_anoncreds_ledger_read();
    let (_, delta, timestamp) = ledger.get_rev_reg_delta_json(rev_reg_id, None, None).await.unwrap();

    issuer_credential
        .revoke_credential_local(&faber.profile.inject_anoncreds())
        .await
        .unwrap();

    let (_, delta_after_revoke, _) = ledger
        .get_rev_reg_delta_json(rev_reg_id, Some(timestamp + 1), None)
        .await
        .unwrap();

    assert_ne!(delta, delta_after_revoke); // They will not equal as we have saved the delta in cache
}

pub async fn rotate_rev_reg(
    faber: &mut TestAgent,
    credential_def: &CredentialDef,
    rev_reg: &RevocationRegistry,
) -> RevocationRegistry {
    let mut rev_reg = RevocationRegistry::create(
        &faber.profile.inject_anoncreds(),
        &faber.institution_did,
        &credential_def.get_cred_def_id(),
        &rev_reg.get_tails_dir(),
        10,
        2,
    )
    .await
    .unwrap();
    rev_reg
        .publish_revocation_primitives(&faber.profile.inject_anoncreds_ledger_write(), TEST_TAILS_URL)
        .await
        .unwrap();
    rev_reg
}

pub async fn publish_revocation(institution: &mut TestAgent, rev_reg: &RevocationRegistry) {
    rev_reg
        .publish_local_revocations(
            &institution.profile.inject_anoncreds(),
            &institution.profile.inject_anoncreds_ledger_write(),
            &institution.institution_did,
        )
        .await
        .unwrap();
}

pub async fn verifier_create_proof_and_send_request(
    institution: &mut TestAgent,
    schema_id: &str,
    cred_def_id: &str,
    request_name: Option<&str>,
) -> Verifier {
    let requested_attrs = requested_attrs_address(&institution.institution_did, &schema_id, &cred_def_id, None, None);
    let presentation_request_data =
        create_proof_request_data(institution, &requested_attrs.to_string(), "[]", "{}", request_name).await;
    create_verifier_from_request_data(presentation_request_data).await
}

pub async fn prover_select_credentials(
    prover: &mut Prover,
    alice: &mut TestAgent,
    presentation_request: AriesMessage,
    preselected_credentials: Option<&str>,
) -> SelectedCredentials {
    prover.process_aries_msg(presentation_request).await.unwrap();
    assert_eq!(prover.get_state(), ProverState::PresentationRequestReceived);
    let retrieved_credentials = prover
        .retrieve_credentials(&alice.profile.inject_anoncreds())
        .await
        .unwrap();
    info!("prover_select_credentials >> retrieved_credentials: {retrieved_credentials:?}");
    let selected_credentials = match preselected_credentials {
        Some(preselected_credentials) => {
            let credential_data = prover.presentation_request_data().unwrap();
            match_preselected_credentials(&retrieved_credentials, preselected_credentials, &credential_data, true)
        }
        _ => retrieved_to_selected_credentials_simple(&retrieved_credentials, true),
    };

    selected_credentials
}

pub async fn prover_select_credentials_and_send_proof(
    alice: &mut TestAgent,
    presentation_request: RequestPresentation,
    preselected_credentials: Option<&str>,
) -> Presentation {
    let mut prover = create_prover_from_request(presentation_request.clone()).await;
    let selected_credentials =
        prover_select_credentials(&mut prover, alice, presentation_request.into(), preselected_credentials).await;
    info!(
        "Prover :: Retrieved credential converted to selected: {:?}",
        &selected_credentials
    );
    let presentation = generate_and_send_proof(alice, &mut prover, selected_credentials)
        .await
        .unwrap();
    let presentation = match presentation {
        AriesMessage::PresentProof(PresentProof::Presentation(presentation)) => presentation,
        _ => panic!("Unexpected message type"),
    };
    assert_eq!(ProverState::PresentationSent, prover.get_state());
    presentation
}

pub fn retrieved_to_selected_credentials_simple(
    retrieved_credentials: &RetrievedCredentials,
    with_tails: bool,
) -> SelectedCredentials {
    info!(
        "retrieved_to_selected_credentials_simple >>> retrieved matching credentials {:?}",
        retrieved_credentials
    );
    let mut selected_credentials = SelectedCredentials::default();

    for (referent, cred_array) in retrieved_credentials.credentials_by_referent.iter() {
        if cred_array.len() > 0 {
            let first_cred = cred_array[0].clone();
            let tails_dir = with_tails.then_some(get_temp_dir_path().to_str().unwrap().to_owned());
            selected_credentials.select_credential_for_referent_from_retrieved(
                referent.to_owned(),
                first_cred,
                tails_dir,
            );
        }
    }
    return selected_credentials;
}

pub fn match_preselected_credentials(
    retrieved_credentials: &RetrievedCredentials,
    preselected_credentials: &str,
    credential_data: &str,
    with_tails: bool,
) -> SelectedCredentials {
    info!(
        "retrieved_to_selected_credentials_specific >>> retrieved matching credentials {:?}",
        retrieved_credentials
    );
    let credential_data: Value = serde_json::from_str(credential_data).unwrap();
    let preselected_credentials: Value = serde_json::from_str(preselected_credentials).unwrap();
    let requested_attributes: &Value = &credential_data["requested_attributes"];

    let mut selected_credentials = SelectedCredentials::default();

    for (referent, cred_array) in retrieved_credentials.credentials_by_referent.iter() {
        let filtered: Vec<RetrievedCredentialForReferent> = cred_array
            .clone()
            .into_iter()
            .filter_map(|cred| {
                let attribute_name = requested_attributes[referent]["name"].as_str().unwrap();
                let preselected_credential = preselected_credentials[attribute_name].as_str().unwrap();
                if cred.cred_info.attributes[attribute_name] == preselected_credential {
                    Some(cred)
                } else {
                    None
                }
            })
            .collect();
        let first_cred = filtered[0].clone();
        let tails_dir = with_tails.then_some(get_temp_dir_path().to_str().unwrap().to_owned());
        selected_credentials.select_credential_for_referent_from_retrieved(referent.to_owned(), first_cred, tails_dir);
    }
    return selected_credentials;
}

pub async fn exchange_proof(
    institution: &mut TestAgent,
    consumer: &mut TestAgent,
    schema_id: &str,
    cred_def_id: &str,
    request_name: Option<&str>,
) -> Verifier {
    let mut verifier =
        verifier_create_proof_and_send_request(institution, &schema_id, &cred_def_id, request_name).await;
    let presentation =
        prover_select_credentials_and_send_proof(consumer, verifier.get_presentation_request_msg().unwrap(), None)
            .await;

    verifier
        .verify_presentation(
            &institution.profile.inject_anoncreds_ledger_read(),
            &institution.profile.inject_anoncreds(),
            presentation,
        )
        .await
        .unwrap();
    verifier
}
