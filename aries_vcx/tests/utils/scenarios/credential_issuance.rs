use std::sync::Arc;
use std::thread;
use std::time::Duration;

use aries_vcx::common::test_utils::create_and_store_credential_def_and_rev_reg;
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::handlers::util::OfferInfo;
use aries_vcx::protocols::mediated_connection::pairwise_info::PairwiseInfo;
use messages::misc::MimeType;
use messages::msg_fields::protocols::cred_issuance::offer_credential::OfferCredential;
use messages::msg_fields::protocols::cred_issuance::propose_credential::{ProposeCredential, ProposeCredentialContent};
use messages::msg_fields::protocols::cred_issuance::{CredentialAttr, CredentialIssuance, CredentialPreview};
use messages::AriesMessage;
use serde_json::json;

use crate::utils::test_agent::TestAgent;
use aries_vcx::common::primitives::credential_definition::CredentialDef;
use aries_vcx::common::primitives::revocation_registry::RevocationRegistry;
use aries_vcx::handlers::issuance::holder::Holder;
use aries_vcx::handlers::issuance::issuer::Issuer;
use aries_vcx::protocols::issuance::holder::state_machine::HolderState;
use aries_vcx::protocols::issuance::issuer::state_machine::IssuerState;
use aries_vcx::utils::constants::TEST_TAILS_URL;

use super::attr_names;

pub async fn create_address_schema_creddef_revreg(
    profile: &Arc<dyn Profile>,
    institution_did: &str,
) -> (
    String,
    String,
    String,
    String,
    CredentialDef,
    RevocationRegistry,
    Option<String>,
) {
    info!("_create_address_schema >>> ");
    let attrs_list = json!(["address1", "address2", "city", "state", "zip"]).to_string();
    let (schema_id, schema_json, cred_def_id, cred_def_json, rev_reg_id, _, cred_def, rev_reg) =
        create_and_store_credential_def_and_rev_reg(
            &profile.inject_anoncreds(),
            &profile.inject_anoncreds_ledger_read(),
            &profile.inject_anoncreds_ledger_write(),
            &institution_did,
            &attrs_list,
        )
        .await;
    (
        schema_id,
        schema_json,
        cred_def_id.to_string(),
        cred_def_json,
        cred_def,
        rev_reg,
        Some(rev_reg_id),
    )
}

pub fn create_holder_from_proposal(proposal: ProposeCredential) -> Holder {
    let holder = Holder::create_with_proposal("TEST_CREDENTIAL", proposal).unwrap();
    assert_eq!(HolderState::ProposalSet, holder.get_state());
    holder
}

pub fn create_issuer_from_proposal(proposal: ProposeCredential) -> Issuer {
    let issuer = Issuer::create_from_proposal("TEST_CREDENTIAL", &proposal).unwrap();
    assert_eq!(IssuerState::ProposalReceived, issuer.get_state());
    assert_eq!(proposal.clone(), issuer.get_proposal().unwrap());
    issuer
}

pub async fn create_credential_offer(
    faber: &mut TestAgent,
    cred_def: &CredentialDef,
    rev_reg: &RevocationRegistry,
    credential_json: &str,
    comment: Option<&str>,
) -> (Issuer, AriesMessage) {
    let offer_info = OfferInfo {
        credential_json: credential_json.to_string(),
        cred_def_id: cred_def.get_cred_def_id(),
        rev_reg_id: Some(rev_reg.get_rev_reg_id()),
        tails_file: Some(rev_reg.get_tails_dir()),
    };
    info!("create_and_send_cred_offer :: sending credential offer, offer_info: {offer_info:?}");
    let mut issuer = Issuer::create("1").unwrap();
    issuer
        .build_credential_offer_msg(&faber.profile.inject_anoncreds(), offer_info, comment.map(String::from))
        .await
        .unwrap();
    let credential_offer = issuer.get_credential_offer_msg().unwrap();
    info!("create_and_send_cred_offer :: credential offer was created");
    (issuer, credential_offer)
}

async fn create_credential_request(alice: &mut TestAgent, cred_offer: AriesMessage) -> (Holder, AriesMessage) {
    info!("create_credential_request >>>");
    let cred_offer: OfferCredential = match cred_offer {
        AriesMessage::CredentialIssuance(CredentialIssuance::OfferCredential(cred_offer)) => cred_offer,
        _ => panic!("Unexpected message type"),
    };
    let mut holder = Holder::create_from_offer("TEST_CREDENTIAL", cred_offer).unwrap();
    assert_eq!(HolderState::OfferReceived, holder.get_state());
    info!("create_credential_request :: sending credential request");
    let cred_request = holder
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
    (holder, cred_request)
}

pub async fn create_credential_proposal(schema_id: &str, cred_def_id: &str, comment: &str) -> ProposeCredential {
    let (address1, address2, city, state, zip) = attr_names();
    let mut attrs = Vec::new();

    let attr = CredentialAttr::builder()
        .name(address1)
        .value("123 Main Str".to_owned())
        .mime_type(MimeType::Plain)
        .build();

    attrs.push(attr);

    let attr = CredentialAttr::builder()
        .name(address2)
        .value("Suite 3".to_owned())
        .mime_type(MimeType::Plain)
        .build();
    attrs.push(attr);

    let attr = CredentialAttr::builder()
        .name(city)
        .value("Draper".to_owned())
        .mime_type(MimeType::Plain)
        .build();
    attrs.push(attr);

    let attr = CredentialAttr::builder()
        .name(state)
        .value("UT".to_owned())
        .mime_type(MimeType::Plain)
        .build();
    attrs.push(attr);

    let attr = CredentialAttr::builder()
        .name(zip)
        .value("84000".to_owned())
        .mime_type(MimeType::Plain)
        .build();
    attrs.push(attr);

    let preview = CredentialPreview::new(attrs);
    let content = ProposeCredentialContent::builder()
        .credential_proposal(preview)
        .schema_id(schema_id.to_owned())
        .cred_def_id(cred_def_id.to_owned())
        .comment(comment.to_owned())
        .build();

    let id = "test".to_owned();
    ProposeCredential::builder().id(id).content(content).build()
}

pub async fn accept_credential_proposal(
    faber: &mut TestAgent,
    issuer: &mut Issuer,
    cred_proposal: ProposeCredential,
    rev_reg_id: Option<String>,
    tails_dir: Option<String>,
) -> AriesMessage {
    let offer_info = OfferInfo {
        credential_json: json!(cred_proposal.content.credential_proposal.attributes).to_string(),
        cred_def_id: cred_proposal.content.cred_def_id.clone(),
        rev_reg_id,
        tails_file: tails_dir,
    };
    issuer
        .build_credential_offer_msg(&faber.profile.inject_anoncreds(), offer_info, Some("comment".into()))
        .await
        .unwrap();
    let credential_offer = issuer.get_credential_offer_msg().unwrap();
    credential_offer
}

pub async fn accept_offer(alice: &mut TestAgent, cred_offer: AriesMessage, holder: &mut Holder) -> AriesMessage {
    holder
        .process_aries_msg(
            &alice.profile.inject_anoncreds_ledger_read(),
            &alice.profile.inject_anoncreds(),
            cred_offer,
        )
        .await
        .unwrap();
    assert_eq!(HolderState::OfferReceived, holder.get_state());
    assert!(holder.get_offer().is_ok());
    let cred_request = holder
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
    cred_request
}

pub async fn decline_offer(alice: &mut TestAgent, cred_offer: AriesMessage, holder: &mut Holder) -> AriesMessage {
    holder
        .process_aries_msg(
            &alice.profile.inject_anoncreds_ledger_read(),
            &alice.profile.inject_anoncreds(),
            cred_offer,
        )
        .await
        .unwrap();
    assert_eq!(HolderState::OfferReceived, holder.get_state());
    let problem_report = holder.decline_offer(Some("Have a nice day")).unwrap();
    assert_eq!(HolderState::Failed, holder.get_state());
    problem_report.into()
}

pub async fn send_credential(
    alice: &mut TestAgent,
    faber: &mut TestAgent,
    issuer_credential: &mut Issuer,
    holder_credential: &mut Holder,
    cred_request: AriesMessage,
    revokable: bool,
) {
    info!("send_credential >>> getting offers");
    let thread_id = issuer_credential.get_thread_id().unwrap();
    assert_eq!(IssuerState::OfferSet, issuer_credential.get_state());
    assert!(!issuer_credential.is_revokable());

    let cred_request = match cred_request {
        AriesMessage::CredentialIssuance(CredentialIssuance::RequestCredential(request)) => request,
        _ => panic!("Unexpected message type"),
    };

    issuer_credential.receive_request(cred_request).await.unwrap();
    assert_eq!(IssuerState::RequestReceived, issuer_credential.get_state());
    assert!(!issuer_credential.is_revokable());
    assert_eq!(thread_id, issuer_credential.get_thread_id().unwrap());

    info!("send_credential >>> sending credential");
    issuer_credential
        .build_credential(&faber.profile.inject_anoncreds())
        .await
        .unwrap();
    let credential = issuer_credential.get_msg_issue_credential().unwrap();
    assert_eq!(thread_id, issuer_credential.get_thread_id().unwrap());

    info!("send_credential >>> storing credential");
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
) -> (
    String,
    String,
    Option<String>,
    CredentialDef,
    RevocationRegistry,
    Issuer,
) {
    let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, rev_reg_id) =
        create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;

    info!("issue_address_credential");
    let (address1, address2, city, state, zip) = attr_names();
    let credential_data =
        json!({address1: "123 Main St", address2: "Suite 3", city: "Draper", state: "UT", zip: "84000"}).to_string();

    let credential_handle =
        exchange_credential(consumer, institution, credential_data, &cred_def, &rev_reg, None).await;
    (schema_id, cred_def_id, rev_reg_id, cred_def, rev_reg, credential_handle)
}

pub async fn exchange_credential(
    consumer: &mut TestAgent,
    institution: &mut TestAgent,
    credential_data: String,
    cred_def: &CredentialDef,
    rev_reg: &RevocationRegistry,
    comment: Option<&str>,
) -> Issuer {
    info!("Generated credential data: {}", credential_data);
    let (mut issuer_credential, cred_offer) =
        create_credential_offer(institution, cred_def, rev_reg, &credential_data, comment).await;
    info!("AS CONSUMER SEND CREDENTIAL REQUEST");
    let (mut holder_credential, cred_request) = create_credential_request(consumer, cred_offer).await;
    info!("AS INSTITUTION SEND CREDENTIAL");
    send_credential(
        consumer,
        institution,
        &mut issuer_credential,
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
    issuer_credential
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
    let cred_proposal = create_credential_proposal(schema_id, cred_def_id, comment).await;
    let mut holder = create_holder_from_proposal(cred_proposal.clone());
    let mut issuer = create_issuer_from_proposal(cred_proposal.clone());
    let cred_offer = accept_credential_proposal(institution, &mut issuer, cred_proposal, rev_reg_id, tails_dir).await;
    let cred_request = accept_offer(consumer, cred_offer, &mut holder).await;
    send_credential(consumer, institution, &mut issuer, &mut holder, cred_request, true).await;
    (holder, issuer)
}
