pub mod test_utils {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    use aries_vcx::common::test_utils::create_and_store_credential_def_and_rev_reg;
    use aries_vcx::core::profile::profile::Profile;
    use aries_vcx::errors::error::{AriesVcxError, AriesVcxErrorKind};
    use aries_vcx::handlers::proof_presentation::types::{
        RetrievedCredentialForReferent, RetrievedCredentials, SelectedCredentials,
    };
    use aries_vcx::handlers::util::{AnyInvitation, OfferInfo, PresentationProposalData};
    use aries_vcx::protocols::mediated_connection::pairwise_info::PairwiseInfo;
    use aries_vcx::protocols::SendClosureConnection;
    use async_channel::{bounded, Sender};
    use diddoc_legacy::aries::diddoc::AriesDidDoc;
    use messages::misc::MimeType;
    use messages::msg_fields::protocols::connection::request::Request;
    use messages::msg_fields::protocols::connection::Connection;
    use messages::msg_fields::protocols::cred_issuance::offer_credential::OfferCredential;
    use messages::msg_fields::protocols::cred_issuance::propose_credential::{
        ProposeCredential, ProposeCredentialContent, ProposeCredentialDecorators,
    };
    use messages::msg_fields::protocols::cred_issuance::{CredentialAttr, CredentialIssuance, CredentialPreview};
    use messages::msg_fields::protocols::present_proof::present::Presentation;
    use messages::msg_fields::protocols::present_proof::propose::PresentationAttr;
    use messages::msg_fields::protocols::present_proof::request::RequestPresentation;
    use messages::msg_fields::protocols::present_proof::PresentProof;
    use messages::AriesMessage;
    use serde_json::{json, Value};

    use crate::utils::devsetup_alice::Alice;
    use crate::utils::devsetup_faber::Faber;
    #[cfg(feature = "migration")]
    use crate::utils::migration::Migratable;
    use aries_vcx::common::ledger::transactions::into_did_doc;
    use aries_vcx::common::primitives::credential_definition::CredentialDef;
    use aries_vcx::common::primitives::revocation_registry::RevocationRegistry;
    use aries_vcx::common::proofs::proof_request::PresentationRequestData;
    use aries_vcx::common::proofs::proof_request_internal::AttrInfo;
    use aries_vcx::handlers::connection::mediated_connection::{ConnectionState, MediatedConnection};
    use aries_vcx::handlers::issuance::holder::Holder;
    use aries_vcx::handlers::issuance::issuer::Issuer;
    use aries_vcx::handlers::proof_presentation::prover::Prover;
    use aries_vcx::handlers::proof_presentation::verifier::Verifier;
    use aries_vcx::protocols::issuance::holder::state_machine::HolderState;
    use aries_vcx::protocols::issuance::issuer::state_machine::IssuerState;
    use aries_vcx::protocols::mediated_connection::invitee::state_machine::InviteeState;
    use aries_vcx::protocols::mediated_connection::inviter::state_machine::InviterState;
    use aries_vcx::protocols::proof_presentation::prover::state_machine::ProverState;
    use aries_vcx::protocols::proof_presentation::verifier::state_machine::VerifierState;
    use aries_vcx::protocols::proof_presentation::verifier::verification_status::PresentationVerificationStatus;
    use aries_vcx::utils::constants::{DEFAULT_PROOF_NAME, TEST_TAILS_URL};
    use aries_vcx_core::ledger::indy::pool::test_utils::get_temp_dir_path;

    pub fn _send_message(sender: Sender<AriesMessage>) -> Option<SendClosureConnection> {
        Some(Box::new(
            move |message: AriesMessage, _sender_vk: String, _did_doc: AriesDidDoc| {
                Box::pin(async move {
                    sender.send(message).await.map_err(|err| {
                        AriesVcxError::from_msg(
                            AriesVcxErrorKind::IOError,
                            format!("Failed to send message: {:?}", err),
                        )
                    })
                })
            },
        ))
    }

    pub fn attr_names() -> (String, String, String, String, String) {
        let address1 = "Address1".to_string();
        let address2 = "address2".to_string();
        let city = "CITY".to_string();
        let state = "State".to_string();
        let zip = "zip".to_string();
        (address1, address2, city, state, zip)
    }

    pub fn requested_attrs(did: &str, schema_id: &str, cred_def_id: &str, from: Option<u64>, to: Option<u64>) -> Value {
        let (address1, address2, city, state, zip) = attr_names();
        json!([
           {
               "name": address1,
               "non_revoked": {"from": from, "to": to},
               "restrictions": [{
                 "issuer_did": did,
                 "schema_id": schema_id,
                 "cred_def_id": cred_def_id,
               }]
           },
           {
               "name": address2,
               "non_revoked": {"from": from, "to": to},
               "restrictions": [{
                 "issuer_did": did,
                 "schema_id": schema_id,
                 "cred_def_id": cred_def_id,
               }],
           },
           {
               "name": city,
               "non_revoked": {"from": from, "to": to},
               "restrictions": [{
                 "issuer_did": did,
                 "schema_id": schema_id,
                 "cred_def_id": cred_def_id,
               }]
           },
           {
               "name": state,
               "non_revoked": {"from": from, "to": to},
               "restrictions": [{
                 "issuer_did": did,
                 "schema_id": schema_id,
                 "cred_def_id": cred_def_id,
               }]
           },
           {
               "name": zip,
               "non_revoked": {"from": from, "to": to},
               "restrictions": [{
                 "issuer_did": did,
                 "schema_id": schema_id,
                 "cred_def_id": cred_def_id,
               }]
           }
        ])
    }

    pub fn requested_attr_objects(cred_def_id: &str) -> Vec<PresentationAttr> {
        let (address1, address2, city, state, zip) = attr_names();
        let mut address1_attr = PresentationAttr::new(address1);
        address1_attr.cred_def_id = Some(cred_def_id.to_owned());
        address1_attr.value = Some("123 Main St".to_owned());

        let mut address2_attr = PresentationAttr::new(address2);
        address2_attr.cred_def_id = Some(cred_def_id.to_owned());
        address2_attr.value = Some("Suite 3".to_owned());

        let mut city_attr = PresentationAttr::new(city);
        city_attr.cred_def_id = Some(cred_def_id.to_owned());
        city_attr.value = Some("Draper".to_owned());

        let mut state_attr = PresentationAttr::new(state);
        state_attr.cred_def_id = Some(cred_def_id.to_owned());
        state_attr.value = Some("UT".to_owned());

        let mut zip_attr = PresentationAttr::new(zip);
        zip_attr.cred_def_id = Some(cred_def_id.to_owned());
        zip_attr.value = Some("84000".to_owned());

        vec![address1_attr, address2_attr, city_attr, state_attr, zip_attr]
    }

    pub fn requested_attr_objects_1(cred_def_id: &str) -> Vec<PresentationAttr> {
        let (address1, address2, city, state, zip) = attr_names();
        let mut address1_attr = PresentationAttr::new(address1);
        address1_attr.cred_def_id = Some(cred_def_id.to_owned());
        address1_attr.value = Some("456 Side St".to_owned());

        let mut address2_attr = PresentationAttr::new(address2);
        address2_attr.cred_def_id = Some(cred_def_id.to_owned());
        address2_attr.value = Some("Suite 666".to_owned());

        let mut city_attr = PresentationAttr::new(city);
        city_attr.cred_def_id = Some(cred_def_id.to_owned());
        city_attr.value = Some("Austin".to_owned());

        let mut state_attr = PresentationAttr::new(state);
        state_attr.cred_def_id = Some(cred_def_id.to_owned());
        state_attr.value = Some("TC".to_owned());

        let mut zip_attr = PresentationAttr::new(zip);
        zip_attr.cred_def_id = Some(cred_def_id.to_owned());
        zip_attr.value = Some("42000".to_owned());

        vec![address1_attr, address2_attr, city_attr, state_attr, zip_attr]
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

    pub async fn create_nonrevocable_cred_offer(
        faber: &mut Faber,
        cred_def: &CredentialDef,
        credential_json: &str,
        comment: Option<&str>,
    ) -> (Issuer, AriesMessage) {
        info!("create_nonrevocable_cred_offer >> creating issuer credential");
        let offer_info = OfferInfo {
            credential_json: credential_json.to_string(),
            cred_def_id: cred_def.get_cred_def_id(),
            rev_reg_id: None,
            tails_file: None,
        };
        let mut issuer = Issuer::create("1").unwrap();
        info!("create_nonrevocable_cred_offer :: building credential offer");
        issuer
            .build_credential_offer_msg(&faber.profile.inject_anoncreds(), offer_info, comment.map(String::from))
            .await
            .unwrap();
        let credential_offer = issuer.get_credential_offer_msg().unwrap();

        info!("create_nonrevocable_cred_offer :: credential offer was built");
        (issuer, credential_offer)
    }

    pub async fn create_credential_offer(
        faber: &mut Faber,
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

    pub async fn create_credential_request(alice: &mut Alice, cred_offer: AriesMessage) -> (Holder, AriesMessage) {
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

    pub async fn create_credential_proposal(
        alice: &mut Alice,
        schema_id: &str,
        cred_def_id: &str,
        comment: &str,
    ) -> ProposeCredential {
        let (address1, address2, city, state, zip) = attr_names();
        let mut attrs = Vec::new();

        let mut attr = CredentialAttr::new(address1, "123 Main Str".to_owned());
        attr.mime_type = Some(MimeType::Plain);
        attrs.push(attr);

        let mut attr = CredentialAttr::new(address2, "Suite 3".to_owned());
        attr.mime_type = Some(MimeType::Plain);
        attrs.push(attr);

        let mut attr = CredentialAttr::new(city, "Draper".to_owned());
        attr.mime_type = Some(MimeType::Plain);
        attrs.push(attr);

        let mut attr = CredentialAttr::new(state, "UT".to_owned());
        attr.mime_type = Some(MimeType::Plain);
        attrs.push(attr);

        let mut attr = CredentialAttr::new(zip, "84000".to_owned());
        attr.mime_type = Some(MimeType::Plain);
        attrs.push(attr);

        let preview = CredentialPreview::new(attrs);
        let mut content = ProposeCredentialContent::new(preview, schema_id.to_owned(), cred_def_id.to_owned());
        content.comment = Some(comment.to_owned());

        let decorators = ProposeCredentialDecorators::default();

        let id = "test".to_owned();
        ProposeCredential::with_decorators(id, content, decorators)
    }

    pub async fn accept_credential_proposal(
        faber: &mut Faber,
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

    pub async fn accept_offer(alice: &mut Alice, cred_offer: AriesMessage, holder: &mut Holder) -> AriesMessage {
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

    pub async fn decline_offer(alice: &mut Alice, cred_offer: AriesMessage, holder: &mut Holder) -> AriesMessage {
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
        alice: &mut Alice,
        faber: &mut Faber,
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

    pub async fn _exchange_credential(
        consumer: &mut Alice,
        institution: &mut Faber,
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

    pub async fn _exchange_credential_with_proposal(
        consumer: &mut Alice,
        institution: &mut Faber,
        schema_id: &str,
        cred_def_id: &str,
        rev_reg_id: Option<String>,
        tails_dir: Option<String>,
        comment: &str,
    ) -> (Holder, Issuer) {
        let cred_proposal = create_credential_proposal(consumer, schema_id, cred_def_id, comment).await;
        let mut holder = create_holder_from_proposal(cred_proposal.clone());
        let mut issuer = create_issuer_from_proposal(cred_proposal.clone());
        let cred_offer =
            accept_credential_proposal(institution, &mut issuer, cred_proposal, rev_reg_id, tails_dir).await;
        let cred_request = accept_offer(consumer, cred_offer, &mut holder).await;
        send_credential(consumer, institution, &mut issuer, &mut holder, cred_request, true).await;
        (holder, issuer)
    }

    pub async fn issue_address_credential(
        consumer: &mut Alice,
        institution: &mut Faber,
    ) -> (
        String,
        String,
        Option<String>,
        CredentialDef,
        RevocationRegistry,
        Issuer,
    ) {
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, rev_reg_id) =
            _create_address_schema_creddef_revreg(&institution.profile, &institution.institution_did).await;

        info!("issue_address_credential");
        let (address1, address2, city, state, zip) = attr_names();
        let credential_data =
            json!({address1: "123 Main St", address2: "Suite 3", city: "Draper", state: "UT", zip: "84000"})
                .to_string();

        let credential_handle =
            _exchange_credential(consumer, institution, credential_data, &cred_def, &rev_reg, None).await;
        (schema_id, cred_def_id, rev_reg_id, cred_def, rev_reg, credential_handle)
    }

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
        faber: &mut Faber,
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
            AriesMessage::PresentProof(PresentProof::ProposePresentation(presentation_proposal)) => {
                presentation_proposal
            }
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
            .decline_presentation_proposal("I don't like Alices") // :(
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
        faber: &mut Faber,
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
        alice: &mut Alice,
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

    pub async fn verify_proof(faber: &mut Faber, verifier: &mut Verifier, presentation: AriesMessage) -> AriesMessage {
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
        faber: &mut Faber,
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

    pub async fn revoke_credential_local(faber: &mut Faber, issuer_credential: &Issuer, rev_reg_id: &str) {
        let ledger = Arc::clone(&faber.profile).inject_anoncreds_ledger_read();
        let (_, delta, timestamp) = ledger.get_rev_reg_delta_json(rev_reg_id, None, None).await.unwrap();
        info!("revoking credential locally");

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
        faber: &mut Faber,
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

    pub async fn publish_revocation(institution: &mut Faber, rev_reg: &RevocationRegistry) {
        rev_reg
            .publish_local_revocations(
                &institution.profile.inject_anoncreds(),
                &institution.profile.inject_anoncreds_ledger_write(),
                &institution.institution_did,
            )
            .await
            .unwrap();
    }

    pub async fn _create_address_schema_creddef_revreg(
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

    pub async fn verifier_create_proof_and_send_request(
        institution: &mut Faber,
        schema_id: &str,
        cred_def_id: &str,
        request_name: Option<&str>,
    ) -> Verifier {
        let requested_attrs = requested_attrs(&institution.institution_did, &schema_id, &cred_def_id, None, None);
        let requested_attrs_string = serde_json::to_string(&requested_attrs).unwrap();
        let presentation_request_data =
            create_proof_request_data(institution, &requested_attrs_string, "[]", "{}", request_name).await;
        create_verifier_from_request_data(presentation_request_data).await
    }

    pub async fn prover_select_credentials(
        prover: &mut Prover,
        alice: &mut Alice,
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
        alice: &mut Alice,
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

    pub async fn connect_using_request_sent_to_public_agent(
        alice: &mut Alice,
        faber: &mut Faber,
        consumer_to_institution: &mut MediatedConnection,
        request: Request,
    ) -> MediatedConnection {
        tokio::time::sleep(Duration::from_millis(1000)).await;
        let mut institution_to_consumer = MediatedConnection::create_with_request(
            &faber.profile.inject_wallet(),
            request,
            faber.pairwise_info.clone(),
            &faber.agency_client,
        )
        .await
        .unwrap();
        assert_eq!(
            ConnectionState::Inviter(InviterState::Requested),
            institution_to_consumer.get_state()
        );
        institution_to_consumer
            .find_message_and_update_state(&faber.profile.inject_wallet(), &faber.agency_client)
            .await
            .unwrap();
        assert_eq!(
            ConnectionState::Inviter(InviterState::Responded),
            institution_to_consumer.get_state()
        );

        consumer_to_institution
            .find_message_and_update_state(&alice.profile.inject_wallet(), &alice.agency_client)
            .await
            .unwrap();
        assert_eq!(
            ConnectionState::Invitee(InviteeState::Completed),
            consumer_to_institution.get_state()
        );

        tokio::time::sleep(Duration::from_millis(1000)).await;
        institution_to_consumer
            .find_message_and_update_state(&faber.profile.inject_wallet(), &faber.agency_client)
            .await
            .unwrap();
        assert_eq!(
            ConnectionState::Inviter(InviterState::Completed),
            institution_to_consumer.get_state()
        );

        assert_eq!(
            institution_to_consumer.get_thread_id(),
            consumer_to_institution.get_thread_id()
        );

        institution_to_consumer
    }

    pub async fn create_connected_connections_via_public_invite(
        alice: &mut Alice,
        institution: &mut Faber,
    ) -> (MediatedConnection, MediatedConnection) {
        let (sender, receiver) = bounded::<AriesMessage>(1);
        let public_invite_json = institution.create_public_invite().unwrap();
        let public_invite: AnyInvitation = serde_json::from_str(&public_invite_json).unwrap();
        let ddo = into_did_doc(&alice.profile.inject_indy_ledger_read(), &public_invite)
            .await
            .unwrap();

        let mut consumer_to_institution = MediatedConnection::create_with_invite(
            "institution",
            &alice.profile.inject_wallet(),
            &alice.agency_client,
            public_invite,
            ddo,
            true,
        )
        .await
        .unwrap();
        consumer_to_institution
            .connect(
                &alice.profile.inject_wallet(),
                &alice.agency_client,
                _send_message(sender),
            )
            .await
            .unwrap();

        let request = if let AriesMessage::Connection(Connection::Request(request)) = receiver.recv().await.unwrap() {
            request
        } else {
            panic!("Received invalid message type")
        };

        let institution_to_consumer =
            connect_using_request_sent_to_public_agent(alice, institution, &mut consumer_to_institution, request).await;
        (consumer_to_institution, institution_to_consumer)
    }

    pub async fn create_connected_connections(
        alice: &mut Alice,
        faber: &mut Faber,
    ) -> (MediatedConnection, MediatedConnection) {
        debug!("Institution is going to create connection.");
        let mut institution_to_consumer =
            MediatedConnection::create("consumer", &faber.profile.inject_wallet(), &faber.agency_client, true)
                .await
                .unwrap();
        institution_to_consumer
            .connect(&faber.profile.inject_wallet(), &faber.agency_client, None)
            .await
            .unwrap();
        let details = institution_to_consumer.get_invite_details().unwrap();

        debug!("Consumer is going to accept connection invitation.");
        let ddo = into_did_doc(&alice.profile.inject_indy_ledger_read(), &details)
            .await
            .unwrap();
        let mut consumer_to_institution = MediatedConnection::create_with_invite(
            "institution",
            &alice.profile.inject_wallet(),
            &alice.agency_client,
            details.clone(),
            ddo,
            true,
        )
        .await
        .unwrap();

        consumer_to_institution
            .connect(&alice.profile.inject_wallet(), &alice.agency_client, None)
            .await
            .unwrap();

        let thread_id = consumer_to_institution.get_thread_id();

        debug!("Institution is going to process connection request.");
        tokio::time::sleep(Duration::from_millis(1000)).await;
        institution_to_consumer
            .find_message_and_update_state(&faber.profile.inject_wallet(), &faber.agency_client)
            .await
            .unwrap();
        assert_eq!(
            ConnectionState::Inviter(InviterState::Responded),
            institution_to_consumer.get_state()
        );
        assert_eq!(thread_id, institution_to_consumer.get_thread_id());

        debug!("Consumer is going to complete the connection protocol.");
        consumer_to_institution
            .find_message_and_update_state(&alice.profile.inject_wallet(), &alice.agency_client)
            .await
            .unwrap();
        assert_eq!(
            ConnectionState::Invitee(InviteeState::Completed),
            consumer_to_institution.get_state()
        );
        assert_eq!(thread_id, consumer_to_institution.get_thread_id());

        debug!("Institution is going to complete the connection protocol.");
        tokio::time::sleep(Duration::from_millis(1000)).await;
        institution_to_consumer
            .find_message_and_update_state(&faber.profile.inject_wallet(), &faber.agency_client)
            .await
            .unwrap();
        assert_eq!(
            ConnectionState::Inviter(InviterState::Completed),
            institution_to_consumer.get_state()
        );
        assert_eq!(thread_id, consumer_to_institution.get_thread_id());

        (consumer_to_institution, institution_to_consumer)
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
            selected_credentials.select_credential_for_referent_from_retrieved(
                referent.to_owned(),
                first_cred,
                tails_dir,
            );
        }
        return selected_credentials;
    }

    pub async fn exchange_proof_and_verify(
        institution: &mut Faber,
        consumer: &mut Alice,
        schema_id: &str,
        cred_def_id: &str,
        request_name: Option<&str>,
    ) {
        let mut verifier =
            verifier_create_proof_and_send_request(institution, &schema_id, &cred_def_id, request_name).await;

        #[cfg(feature = "migration")]
        consumer.migrate().await;

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
        assert_eq!(
            verifier.get_verification_status(),
            PresentationVerificationStatus::Valid
        );
    }

    // TODO: Deduplicate
    pub async fn exchange_proof_and_verify_invalid(
        institution: &mut Faber,
        consumer: &mut Alice,
        schema_id: &str,
        cred_def_id: &str,
        request_name: Option<&str>,
    ) {
        let mut verifier =
            verifier_create_proof_and_send_request(institution, &schema_id, &cred_def_id, request_name).await;

        #[cfg(feature = "migration")]
        consumer.migrate().await;

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
        assert_eq!(
            verifier.get_verification_status(),
            PresentationVerificationStatus::Invalid
        );
    }
}
