use std::{sync::Arc, thread, time::Duration};

use aries_vcx::{
    common::{
        primitives::{
            credential_definition::CredentialDef, credential_schema::Schema,
            revocation_registry::RevocationRegistry,
        },
        test_utils::{
            create_and_write_test_cred_def, create_and_write_test_rev_reg,
            create_and_write_test_schema,
        },
    },
    core::profile::profile::Profile,
    handlers::{
        issuance::{holder::Holder, issuer::Issuer},
        util::OfferInfo,
    },
    protocols::{
        issuance::{holder::state_machine::HolderState, issuer::state_machine::IssuerState},
        mediated_connection::pairwise_info::PairwiseInfo,
    },
    utils::constants::TEST_TAILS_URL,
};
use messages::msg_fields::protocols::{
    cred_issuance::v1::{
        offer_credential::OfferCredentialV1, propose_credential::ProposeCredentialV1,
        request_credential::RequestCredentialV1,
    },
    report_problem::ProblemReport,
};
use serde_json::json;

use super::{attr_names_address_list, create_credential_proposal, credential_data_address_1};
use crate::utils::test_agent::TestAgent;

pub async fn create_address_schema_creddef_revreg(
    profile: &Arc<dyn Profile>,
    institution_did: &str,
) -> (Schema, CredentialDef, RevocationRegistry) {
    let ledger_read = profile.inject_anoncreds_ledger_read();
    let ledger_write = profile.inject_anoncreds_ledger_write();
    let anoncreds = profile.inject_anoncreds();

    let schema = create_and_write_test_schema(
        &anoncreds,
        &ledger_write,
        institution_did,
        &json!(attr_names_address_list()).to_string(),
    )
    .await;
    let cred_def = create_and_write_test_cred_def(
        &anoncreds,
        &ledger_read,
        &ledger_write,
        institution_did,
        &schema.schema_id,
        true,
    )
    .await;
    let rev_reg = create_and_write_test_rev_reg(
        &anoncreds,
        &ledger_write,
        institution_did,
        &cred_def.get_cred_def_id(),
    )
    .await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    (schema, cred_def, rev_reg)
}

pub fn create_holder_from_proposal(proposal: ProposeCredentialV1) -> Holder {
    let holder = Holder::create_with_proposal("TEST_CREDENTIAL", proposal).unwrap();
    assert_eq!(HolderState::ProposalSet, holder.get_state());
    holder
}

pub fn create_issuer_from_proposal(proposal: ProposeCredentialV1) -> Issuer {
    let issuer = Issuer::create_from_proposal("TEST_CREDENTIAL", &proposal).unwrap();
    assert_eq!(IssuerState::ProposalReceived, issuer.get_state());
    assert_eq!(proposal.clone(), issuer.get_proposal().unwrap());
    issuer
}

pub async fn accept_credential_proposal(
    faber: &mut TestAgent,
    issuer: &mut Issuer,
    cred_proposal: ProposeCredentialV1,
    rev_reg_id: Option<String>,
    tails_dir: Option<String>,
) -> OfferCredentialV1 {
    let offer_info = OfferInfo {
        credential_json: json!(cred_proposal.content.credential_proposal.attributes).to_string(),
        cred_def_id: cred_proposal.content.cred_def_id.clone(),
        rev_reg_id,
        tails_file: tails_dir,
    };
    issuer
        .build_credential_offer_msg(
            &faber.profile.inject_anoncreds(),
            offer_info,
            Some("comment".into()),
        )
        .await
        .unwrap();
    issuer.get_credential_offer().unwrap()
}

pub async fn accept_offer(
    alice: &mut TestAgent,
    cred_offer: OfferCredentialV1,
    holder: &mut Holder,
) -> RequestCredentialV1 {
    // TODO: Replace with message-specific handler
    holder
        .process_aries_msg(
            &alice.profile.inject_anoncreds_ledger_read(),
            &alice.profile.inject_anoncreds(),
            cred_offer.into(),
        )
        .await
        .unwrap();
    assert_eq!(HolderState::OfferReceived, holder.get_state());
    assert!(holder.get_offer().is_ok());
    holder
        .prepare_credential_request(
            &alice.profile.inject_anoncreds_ledger_read(),
            &alice.profile.inject_anoncreds(),
            PairwiseInfo::create(&alice.profile.inject_wallet())
                .await
                .unwrap()
                .pw_did,
        )
        .await
        .unwrap();
    assert_eq!(HolderState::RequestSet, holder.get_state());
    holder.get_msg_credential_request().unwrap()
}

pub async fn decline_offer(
    alice: &mut TestAgent,
    cred_offer: OfferCredentialV1,
    holder: &mut Holder,
) -> ProblemReport {
    // TODO: Replace with message-specific handler
    holder
        .process_aries_msg(
            &alice.profile.inject_anoncreds_ledger_read(),
            &alice.profile.inject_anoncreds(),
            cred_offer.into(),
        )
        .await
        .unwrap();
    assert_eq!(HolderState::OfferReceived, holder.get_state());
    let problem_report = holder.decline_offer(Some("Have a nice day")).unwrap();
    assert_eq!(HolderState::Failed, holder.get_state());
    problem_report
}

pub async fn send_credential(
    alice: &mut TestAgent,
    faber: &mut TestAgent,
    issuer_credential: &mut Issuer,
    holder_credential: &mut Holder,
    cred_request: RequestCredentialV1,
    revokable: bool,
) {
    let thread_id = issuer_credential.get_thread_id().unwrap();
    assert_eq!(IssuerState::OfferSet, issuer_credential.get_state());
    assert!(!issuer_credential.is_revokable());

    issuer_credential
        .receive_request(cred_request)
        .await
        .unwrap();
    assert_eq!(IssuerState::RequestReceived, issuer_credential.get_state());
    assert!(!issuer_credential.is_revokable());
    assert_eq!(thread_id, issuer_credential.get_thread_id().unwrap());

    issuer_credential
        .build_credential(&faber.profile.inject_anoncreds())
        .await
        .unwrap();
    let credential = issuer_credential.get_msg_issue_credential().unwrap();
    assert_eq!(thread_id, issuer_credential.get_thread_id().unwrap());

    assert_eq!(thread_id, holder_credential.get_thread_id().unwrap());
    assert_eq!(
        holder_credential
            .is_revokable(&alice.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap(),
        revokable
    );
    holder_credential
        .process_credential(
            &alice.profile.inject_anoncreds_ledger_read(),
            &alice.profile.inject_anoncreds(),
            credential,
        )
        .await
        .unwrap();
    assert_eq!(HolderState::Finished, holder_credential.get_state());
    assert_eq!(
        holder_credential
            .is_revokable(&alice.profile.inject_anoncreds_ledger_read())
            .await
            .unwrap(),
        revokable
    );
    assert_eq!(thread_id, holder_credential.get_thread_id().unwrap());

    if revokable {
        thread::sleep(Duration::from_millis(500));
        assert_eq!(
            holder_credential.get_tails_location().unwrap(),
            TEST_TAILS_URL.to_string()
        );
    }
}

pub async fn issue_address_credential(
    consumer: &mut TestAgent,
    institution: &mut TestAgent,
) -> (Schema, CredentialDef, RevocationRegistry, Issuer) {
    let (schema, cred_def, rev_reg) =
        create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did)
            .await;
    let issuer = exchange_credential(
        consumer,
        institution,
        credential_data_address_1().to_string(),
        &cred_def,
        &rev_reg,
        None,
    )
    .await;
    (schema, cred_def, rev_reg, issuer)
}

pub async fn exchange_credential(
    consumer: &mut TestAgent,
    institution: &mut TestAgent,
    credential_data: String,
    cred_def: &CredentialDef,
    rev_reg: &RevocationRegistry,
    comment: Option<&str>,
) -> Issuer {
    let mut issuer =
        create_credential_offer(institution, cred_def, rev_reg, &credential_data, comment).await;
    let mut holder_credential =
        create_credential_request(consumer, issuer.get_credential_offer().unwrap()).await;
    let cred_request = holder_credential.get_msg_credential_request().unwrap();
    send_credential(
        consumer,
        institution,
        &mut issuer,
        &mut holder_credential,
        cred_request,
        true,
    )
    .await;
    assert!(!holder_credential
        .is_revoked(
            &consumer.profile.inject_anoncreds_ledger_read(),
            &consumer.profile.inject_anoncreds(),
        )
        .await
        .unwrap());
    issuer
}

pub async fn exchange_credential_with_proposal(
    consumer: &mut TestAgent,
    institution: &mut TestAgent,
    schema_id: &str,
    cred_def_id: &str,
    rev_reg_id: Option<String>,
    tails_dir: Option<String>,
    comment: &str,
) -> (Holder, Issuer) {
    let cred_proposal = create_credential_proposal(schema_id, cred_def_id, comment);
    let mut holder = create_holder_from_proposal(cred_proposal.clone());
    let mut issuer = create_issuer_from_proposal(cred_proposal.clone());
    let cred_offer = accept_credential_proposal(
        institution,
        &mut issuer,
        cred_proposal,
        rev_reg_id,
        tails_dir,
    )
    .await;
    let cred_request = accept_offer(consumer, cred_offer, &mut holder).await;
    send_credential(
        consumer,
        institution,
        &mut issuer,
        &mut holder,
        cred_request,
        true,
    )
    .await;
    (holder, issuer)
}

async fn create_credential_offer(
    faber: &mut TestAgent,
    cred_def: &CredentialDef,
    rev_reg: &RevocationRegistry,
    credential_json: &str,
    comment: Option<&str>,
) -> Issuer {
    let offer_info = OfferInfo {
        credential_json: credential_json.to_string(),
        cred_def_id: cred_def.get_cred_def_id(),
        rev_reg_id: Some(rev_reg.get_rev_reg_id()),
        tails_file: Some(rev_reg.get_tails_dir()),
    };
    let mut issuer = Issuer::create("1").unwrap();
    issuer
        .build_credential_offer_msg(
            &faber.profile.inject_anoncreds(),
            offer_info,
            comment.map(String::from),
        )
        .await
        .unwrap();
    issuer
}

async fn create_credential_request(alice: &mut TestAgent, cred_offer: OfferCredentialV1) -> Holder {
    let mut holder = Holder::create_from_offer("TEST_CREDENTIAL", cred_offer).unwrap();
    assert_eq!(HolderState::OfferReceived, holder.get_state());
    holder
        .prepare_credential_request(
            &alice.profile.inject_anoncreds_ledger_read(),
            &alice.profile.inject_anoncreds(),
            PairwiseInfo::create(&alice.profile.inject_wallet())
                .await
                .unwrap()
                .pw_did,
        )
        .await
        .unwrap();
    holder
}
