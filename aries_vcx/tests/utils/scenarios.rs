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
    use messages::msg_fields::protocols::cred_issuance::{CredentialAttr, CredentialPreview};
    use messages::msg_fields::protocols::present_proof::propose::PresentationAttr;
    use messages::msg_fields::protocols::present_proof::request::RequestPresentation;
    use messages::AriesMessage;
    use serde_json::{json, Value};

    use crate::utils::devsetup_alice::Alice;
    use crate::utils::devsetup_faber::Faber;
    use aries_vcx::common::ledger::transactions::into_did_doc;
    use aries_vcx::common::primitives::credential_definition::CredentialDef;
    use aries_vcx::common::primitives::revocation_registry::RevocationRegistry;
    use aries_vcx::common::proofs::proof_request::PresentationRequestData;
    use aries_vcx::common::proofs::proof_request_internal::AttrInfo;
    use aries_vcx::handlers::connection::mediated_connection::{ConnectionState, MediatedConnection};
    use aries_vcx::handlers::issuance::holder::test_utils::get_credential_offer_messages;
    use aries_vcx::handlers::issuance::holder::Holder;
    use aries_vcx::handlers::issuance::issuer::test_utils::get_credential_proposal_messages;
    use aries_vcx::handlers::issuance::issuer::Issuer;
    use aries_vcx::handlers::proof_presentation::prover::test_utils::get_proof_request_messages;
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
    use aries_vcx::utils::filters::{filter_credential_offers_by_comment, filter_proof_requests_by_name};
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

    pub async fn create_and_send_nonrevocable_cred_offer(
        faber: &mut Faber,
        cred_def: &CredentialDef,
        connection: &MediatedConnection,
        credential_json: &str,
        comment: Option<&str>,
    ) -> Issuer {
        info!("create_and_send_nonrevocable_cred_offer >> creating issuer credential");
        let offer_info = OfferInfo {
            credential_json: credential_json.to_string(),
            cred_def_id: cred_def.get_cred_def_id(),
            rev_reg_id: None,
            tails_file: None,
        };
        let mut issuer = Issuer::create("1").unwrap();
        info!("create_and_send_nonrevocable_cred_offer :: sending credential offer");
        issuer
            .build_credential_offer_msg(&faber.profile.inject_anoncreds(), offer_info, comment.map(String::from))
            .await
            .unwrap();
        issuer
            .send_credential_offer(
                connection
                    .send_message_closure(faber.profile.inject_wallet())
                    .await
                    .unwrap(),
            )
            .await
            .unwrap();
        info!("create_and_send_nonrevocable_cred_offer :: credential offer was sent");
        tokio::time::sleep(Duration::from_millis(1000)).await;
        issuer
    }

    pub async fn create_and_send_cred_offer(
        faber: &mut Faber,
        cred_def: &CredentialDef,
        rev_reg: &RevocationRegistry,
        connection: &MediatedConnection,
        credential_json: &str,
        comment: Option<&str>,
    ) -> Issuer {
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
        issuer
            .send_credential_offer(
                connection
                    .send_message_closure(faber.profile.inject_wallet())
                    .await
                    .unwrap(),
            )
            .await
            .unwrap();
        info!("create_and_send_cred_offer :: credential offer was sent");
        tokio::time::sleep(Duration::from_millis(1000)).await;
        issuer
    }

    pub async fn send_cred_req(alice: &mut Alice, connection: &MediatedConnection, comment: Option<&str>) -> Holder {
        info!("send_cred_req >>> switching to consumer");
        info!("send_cred_req :: getting offers");
        let credential_offers = get_credential_offer_messages(&alice.agency_client, connection)
            .await
            .unwrap();
        let credential_offers = match comment {
            Some(comment) => {
                let filtered = filter_credential_offers_by_comment(&credential_offers, comment).unwrap();
                info!(
                    "send_cred_req :: credential offer  messages filtered by comment {}: {}",
                    comment, filtered
                );
                filtered
            }
            _ => credential_offers.to_string(),
        };
        let offers: Value = serde_json::from_str(&credential_offers).unwrap();
        let offers = offers.as_array().unwrap();
        assert_eq!(offers.len(), 1);
        let offer = serde_json::to_string(&offers[0]).unwrap();
        info!("send_cred_req :: creating credential from offer");
        let cred_offer: OfferCredential = serde_json::from_str(&offer).unwrap();
        let mut holder = Holder::create_from_offer("TEST_CREDENTIAL", cred_offer).unwrap();
        assert_eq!(HolderState::OfferReceived, holder.get_state());
        info!("send_cred_req :: sending credential request");
        let my_pw_did = connection.pairwise_info().pw_did.to_string();
        holder
            .send_request(
                &alice.profile.inject_anoncreds_ledger_read(),
                &alice.profile.inject_anoncreds(),
                my_pw_did,
                connection
                    .send_message_closure(alice.profile.inject_wallet())
                    .await
                    .unwrap(),
            )
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(1000)).await;
        holder
    }

    pub async fn send_cred_proposal(
        alice: &mut Alice,
        connection: &MediatedConnection,
        schema_id: &str,
        cred_def_id: &str,
        comment: &str,
    ) -> Holder {
        let (address1, address2, city, state, zip) = attr_names();
        let id = "test".to_owned();
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

        let proposal = ProposeCredential::with_decorators(id, content, decorators);
        let mut holder = Holder::create("TEST_CREDENTIAL").unwrap();
        assert_eq!(HolderState::Initial, holder.get_state());
        holder
            .send_proposal(
                proposal,
                connection
                    .send_message_closure(alice.profile.inject_wallet())
                    .await
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(HolderState::ProposalSent, holder.get_state());
        tokio::time::sleep(Duration::from_millis(1000)).await;
        holder
    }

    pub async fn send_cred_proposal_1(
        holder: &mut Holder,
        alice: &mut Alice,
        connection: &MediatedConnection,
        schema_id: &str,
        cred_def_id: &str,
        comment: &str,
    ) {
        holder
            .update_state(
                &alice.profile.inject_anoncreds_ledger_read(),
                &alice.profile.inject_anoncreds(),
                &alice.profile.inject_wallet(),
                &alice.agency_client,
                connection,
            )
            .await
            .unwrap();
        assert_eq!(HolderState::OfferReceived, holder.get_state());
        assert!(holder.get_offer().is_ok());
        let (address1, address2, city, state, zip) = attr_names();
        let id = "test".to_owned();
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

        let proposal = ProposeCredential::with_decorators(id, content, decorators);
        holder
            .send_proposal(
                proposal,
                connection
                    .send_message_closure(alice.profile.inject_wallet())
                    .await
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(HolderState::ProposalSent, holder.get_state());
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }

    pub async fn accept_cred_proposal(
        faber: &mut Faber,
        connection: &MediatedConnection,
        rev_reg_id: Option<String>,
        tails_dir: Option<String>,
    ) -> Issuer {
        let proposals: Vec<(String, ProposeCredential)> =
            get_credential_proposal_messages(&faber.agency_client, connection)
                .await
                .unwrap();

        let (uid, proposal) = proposals.last().unwrap();
        connection
            .update_message_status(uid, &faber.agency_client)
            .await
            .unwrap();
        let mut issuer = Issuer::create_from_proposal("TEST_CREDENTIAL", proposal).unwrap();
        assert_eq!(IssuerState::ProposalReceived, issuer.get_state());
        assert_eq!(proposal.clone(), issuer.get_proposal().unwrap());
        let offer_info = OfferInfo {
            credential_json: json!(proposal.content.credential_proposal.attributes).to_string(),
            cred_def_id: proposal.content.cred_def_id.clone(),
            rev_reg_id,
            tails_file: tails_dir,
        };
        issuer
            .build_credential_offer_msg(&faber.profile.inject_anoncreds(), offer_info, Some("comment".into()))
            .await
            .unwrap();
        issuer
            .send_credential_offer(
                connection
                    .send_message_closure(faber.profile.inject_wallet())
                    .await
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(IssuerState::OfferSent, issuer.get_state());
        tokio::time::sleep(Duration::from_millis(1000)).await;
        issuer
    }

    pub async fn accept_cred_proposal_1(
        issuer: &mut Issuer,
        faber: &mut Faber,
        connection: &MediatedConnection,
        rev_reg_id: Option<String>,
        tails_dir: Option<String>,
    ) {
        assert_eq!(IssuerState::OfferSent, issuer.get_state());
        issuer
            .update_state(
                &faber.profile.inject_wallet(),
                &faber.profile.inject_anoncreds(),
                &faber.agency_client,
                connection,
            )
            .await
            .unwrap();
        assert_eq!(IssuerState::ProposalReceived, issuer.get_state());
        let proposal = issuer.get_proposal().unwrap();
        let offer_info = OfferInfo {
            credential_json: json!(proposal.content.credential_proposal.attributes).to_string(),
            cred_def_id: proposal.content.cred_def_id.clone(),
            rev_reg_id,
            tails_file: tails_dir,
        };
        issuer
            .build_credential_offer_msg(&faber.profile.inject_anoncreds(), offer_info, Some("comment".into()))
            .await
            .unwrap();
        issuer
            .send_credential_offer(
                connection
                    .send_message_closure(faber.profile.inject_wallet())
                    .await
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(IssuerState::OfferSent, issuer.get_state());
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }

    pub async fn accept_offer(alice: &mut Alice, connection: &MediatedConnection, holder: &mut Holder) {
        holder
            .update_state(
                &alice.profile.inject_anoncreds_ledger_read(),
                &alice.profile.inject_anoncreds(),
                &alice.profile.inject_wallet(),
                &alice.agency_client,
                connection,
            )
            .await
            .unwrap();
        assert_eq!(HolderState::OfferReceived, holder.get_state());
        assert!(holder.get_offer().is_ok());
        let my_pw_did = connection.pairwise_info().pw_did.to_string();
        holder
            .send_request(
                &alice.profile.inject_anoncreds_ledger_read(),
                &alice.profile.inject_anoncreds(),
                my_pw_did,
                connection
                    .send_message_closure(alice.profile.inject_wallet())
                    .await
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(HolderState::RequestSent, holder.get_state());
    }

    pub async fn decline_offer(alice: &mut Alice, connection: &MediatedConnection, holder: &mut Holder) {
        holder
            .update_state(
                &alice.profile.inject_anoncreds_ledger_read(),
                &alice.profile.inject_anoncreds(),
                &alice.profile.inject_wallet(),
                &alice.agency_client,
                connection,
            )
            .await
            .unwrap();
        assert_eq!(HolderState::OfferReceived, holder.get_state());
        holder
            .decline_offer(
                Some("Have a nice day"),
                connection
                    .send_message_closure(alice.profile.inject_wallet())
                    .await
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(HolderState::Failed, holder.get_state());
    }

    pub async fn send_credential(
        alice: &mut Alice,
        faber: &mut Faber,
        issuer_credential: &mut Issuer,
        issuer_to_consumer: &MediatedConnection,
        consumer_to_issuer: &MediatedConnection,
        holder_credential: &mut Holder,
        revokable: bool,
    ) {
        info!("send_credential >>> getting offers");
        let thread_id = issuer_credential.get_thread_id().unwrap();
        assert_eq!(IssuerState::OfferSent, issuer_credential.get_state());
        assert!(!issuer_credential.is_revokable());

        issuer_credential
            .update_state(
                &faber.profile.inject_wallet(),
                &faber.profile.inject_anoncreds(),
                &faber.agency_client,
                issuer_to_consumer,
            )
            .await
            .unwrap();
        assert_eq!(IssuerState::RequestReceived, issuer_credential.get_state());
        assert!(!issuer_credential.is_revokable());
        assert_eq!(thread_id, issuer_credential.get_thread_id().unwrap());

        info!("send_credential >>> sending credential");
        issuer_credential
            .send_credential(
                &faber.profile.inject_anoncreds(),
                issuer_to_consumer
                    .send_message_closure(faber.profile.inject_wallet())
                    .await
                    .unwrap(),
            )
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(1000)).await;
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
            .update_state(
                &alice.profile.inject_anoncreds_ledger_read(),
                &alice.profile.inject_anoncreds(),
                &alice.profile.inject_wallet(),
                &alice.agency_client,
                consumer_to_issuer,
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

    pub async fn send_proof_proposal(alice: &mut Alice, connection: &MediatedConnection, cred_def_id: &str) -> Prover {
        let attrs = requested_attr_objects(cred_def_id);
        let mut proposal_data = PresentationProposalData::default();
        for attr in attrs.into_iter() {
            proposal_data.attributes.push(attr);
        }
        let mut prover = Prover::create("1").unwrap();
        prover
            .send_proposal(
                proposal_data,
                connection
                    .send_message_closure(alice.profile.inject_wallet())
                    .await
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(prover.get_state(), ProverState::PresentationProposalSent);
        tokio::time::sleep(Duration::from_millis(1000)).await;
        prover
    }

    pub async fn send_proof_proposal_1(
        alice: &mut Alice,
        prover: &mut Prover,
        connection: &MediatedConnection,
        cred_def_id: &str,
    ) {
        prover
            .update_state(
                &alice.profile.inject_anoncreds_ledger_read(),
                &alice.profile.inject_anoncreds(),
                &alice.profile.inject_wallet(),
                &alice.agency_client,
                connection,
            )
            .await
            .unwrap();
        assert_eq!(prover.get_state(), ProverState::PresentationRequestReceived);
        let attrs = requested_attr_objects_1(cred_def_id);
        let mut proposal_data = PresentationProposalData::default();
        for attr in attrs.into_iter() {
            proposal_data.attributes.push(attr);
        }
        prover
            .send_proposal(
                proposal_data,
                connection
                    .send_message_closure(alice.profile.inject_wallet())
                    .await
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(prover.get_state(), ProverState::PresentationProposalSent);
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }

    pub async fn accept_proof_proposal(faber: &mut Faber, verifier: &mut Verifier, connection: &MediatedConnection) {
        verifier
            .update_state(
                &faber.profile.inject_wallet(),
                &faber.profile.inject_anoncreds_ledger_read(),
                &faber.profile.inject_anoncreds(),
                &faber.agency_client,
                connection,
            )
            .await
            .unwrap();
        assert_eq!(verifier.get_state(), VerifierState::PresentationProposalReceived);
        let proposal = verifier.get_presentation_proposal().unwrap();
        let attrs = proposal
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
        verifier.set_request(presentation_request_data, None).unwrap();
        verifier
            .send_presentation_request(
                connection
                    .send_message_closure(faber.profile.inject_wallet())
                    .await
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    pub async fn reject_proof_proposal(faber: &mut Faber, connection: &MediatedConnection) -> Verifier {
        let mut verifier = Verifier::create("1").unwrap();
        verifier
            .update_state(
                &faber.profile.inject_wallet(),
                &faber.profile.inject_anoncreds_ledger_read(),
                &faber.profile.inject_anoncreds(),
                &faber.agency_client,
                connection,
            )
            .await
            .unwrap();
        assert_eq!(verifier.get_state(), VerifierState::PresentationProposalReceived);
        verifier
            .decline_presentation_proposal(
                connection
                    .send_message_closure(faber.profile.inject_wallet())
                    .await
                    .unwrap(),
                "I don't like Alices",
            )
            .await
            .unwrap();
        assert_eq!(verifier.get_state(), VerifierState::Failed);
        verifier
    }

    pub async fn receive_proof_proposal_rejection(
        alice: &mut Alice,
        prover: &mut Prover,
        connection: &MediatedConnection,
    ) {
        assert_eq!(prover.get_state(), ProverState::PresentationProposalSent);
        prover
            .update_state(
                &alice.profile.inject_anoncreds_ledger_read(),
                &alice.profile.inject_anoncreds(),
                &alice.profile.inject_wallet(),
                &alice.agency_client,
                connection,
            )
            .await
            .unwrap();
        assert_eq!(prover.get_state(), ProverState::Failed);
    }

    pub async fn send_proof_request(
        faber: &mut Faber,
        connection: &MediatedConnection,
        requested_attrs: &str,
        requested_preds: &str,
        revocation_interval: &str,
        request_name: Option<&str>,
    ) -> Verifier {
        let presentation_request_data =
            PresentationRequestData::create(&faber.profile.inject_anoncreds(), request_name.unwrap_or("name"))
                .await
                .unwrap()
                .set_requested_attributes_as_string(requested_attrs.to_string())
                .unwrap()
                .set_requested_predicates_as_string(requested_preds.to_string())
                .unwrap()
                .set_not_revoked_interval(revocation_interval.to_string())
                .unwrap();
        let mut verifier = Verifier::create_from_request("1".to_string(), &presentation_request_data).unwrap();
        verifier
            .send_presentation_request(
                connection
                    .send_message_closure(faber.profile.inject_wallet())
                    .await
                    .unwrap(),
            )
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(1000)).await;
        verifier
    }

    pub async fn create_proof_request(
        _faber: &mut Faber,
        requested_attrs: &str,
        requested_preds: &str,
        revocation_interval: &str,
        request_name: Option<&str>,
    ) -> RequestPresentation {
        let presentation_request =
            PresentationRequestData::create(&_faber.profile.inject_anoncreds(), request_name.unwrap_or("name"))
                .await
                .unwrap()
                .set_requested_attributes_as_string(requested_attrs.to_string())
                .unwrap()
                .set_requested_predicates_as_string(requested_preds.to_string())
                .unwrap()
                .set_not_revoked_interval(revocation_interval.to_string())
                .unwrap();
        let verifier = Verifier::create_from_request("1".to_string(), &presentation_request).unwrap();
        verifier.get_presentation_request().unwrap()
    }

    pub async fn create_proof(
        alice: &mut Alice,
        connection: &MediatedConnection,
        request_name: Option<&str>,
    ) -> Prover {
        info!("create_proof >>> getting proof request messages");
        let requests = {
            let _requests = get_proof_request_messages(&alice.agency_client, connection)
                .await
                .unwrap();
            info!("create_proof :: get proof request messages returned {}", _requests);
            match request_name {
                Some(request_name) => {
                    let filtered = filter_proof_requests_by_name(&_requests, request_name).unwrap();
                    info!(
                        "create_proof :: proof request messages filtered by name {}: {}",
                        request_name, filtered
                    );
                    filtered
                }
                _ => _requests.to_string(),
            }
        };
        let requests: Value = serde_json::from_str(&requests).unwrap();
        let requests = requests.as_array().unwrap();
        assert_eq!(requests.len(), 1);
        let request = serde_json::to_string(&requests[0]).unwrap();
        let presentation_request: RequestPresentation = serde_json::from_str(&request).unwrap();
        Prover::create_from_request(DEFAULT_PROOF_NAME, presentation_request).unwrap()
    }

    pub async fn generate_and_send_proof(
        alice: &mut Alice,
        prover: &mut Prover,
        connection: &MediatedConnection,
        selected_credentials: SelectedCredentials,
    ) {
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
            prover
                .send_presentation(
                    connection
                        .send_message_closure(alice.profile.inject_wallet())
                        .await
                        .unwrap(),
                )
                .await
                .unwrap();
            info!("generate_and_send_proof :: proof sent");
            assert_eq!(thread_id, prover.get_thread_id().unwrap());
            tokio::time::sleep(Duration::from_millis(1000)).await;
        }
    }

    pub async fn verify_proof(faber: &mut Faber, verifier: &mut Verifier, connection: &MediatedConnection) {
        verifier
            .update_state(
                &faber.profile.inject_wallet(),
                &faber.profile.inject_anoncreds_ledger_read(),
                &faber.profile.inject_anoncreds(),
                &faber.agency_client,
                &connection,
            )
            .await
            .unwrap();
        assert_eq!(verifier.get_state(), VerifierState::Finished);
        assert_eq!(
            verifier.get_verification_status(),
            PresentationVerificationStatus::Valid
        );
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
        let mut rev_reg_new = RevocationRegistry::create(
            &faber.profile.inject_anoncreds(),
            &faber.institution_did,
            &credential_def.get_cred_def_id(),
            &rev_reg.get_tails_dir(),
            10,
            2,
        )
        .await
        .unwrap();
        rev_reg_new
            .publish_revocation_primitives(&faber.profile.inject_anoncreds_ledger_write(), TEST_TAILS_URL)
            .await
            .unwrap();
        rev_reg_new
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
        let (schema_id, schema_json, cred_def_id, cred_def_json, rev_reg_id, tails_dir, cred_def, rev_reg) =
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

    pub async fn _exchange_credential(
        consumer: &mut Alice,
        institution: &mut Faber,
        credential_data: String,
        cred_def: &CredentialDef,
        rev_reg: &RevocationRegistry,
        consumer_to_issuer: &MediatedConnection,
        issuer_to_consumer: &MediatedConnection,
        comment: Option<&str>,
    ) -> Issuer {
        info!("Generated credential data: {}", credential_data);
        let mut issuer_credential = create_and_send_cred_offer(
            institution,
            cred_def,
            rev_reg,
            issuer_to_consumer,
            &credential_data,
            comment,
        )
        .await;
        info!("AS CONSUMER SEND CREDENTIAL REQUEST");
        let mut holder_credential = send_cred_req(consumer, consumer_to_issuer, comment).await;
        info!("AS INSTITUTION SEND CREDENTIAL");
        send_credential(
            consumer,
            institution,
            &mut issuer_credential,
            issuer_to_consumer,
            consumer_to_issuer,
            &mut holder_credential,
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
        consumer_to_issuer: &MediatedConnection,
        issuer_to_consumer: &MediatedConnection,
        schema_id: &str,
        cred_def_id: &str,
        rev_reg_id: Option<String>,
        tails_dir: Option<String>,
        comment: &str,
    ) -> (Holder, Issuer) {
        let mut holder = send_cred_proposal(consumer, consumer_to_issuer, schema_id, cred_def_id, comment).await;
        let mut issuer = accept_cred_proposal(institution, issuer_to_consumer, rev_reg_id, tails_dir).await;
        accept_offer(consumer, consumer_to_issuer, &mut holder).await;
        tokio::time::sleep(Duration::from_millis(1000)).await;
        send_credential(
            consumer,
            institution,
            &mut issuer,
            issuer_to_consumer,
            consumer_to_issuer,
            &mut holder,
            true,
        )
        .await;
        (holder, issuer)
    }

    pub async fn issue_address_credential(
        consumer: &mut Alice,
        institution: &mut Faber,
        consumer_to_institution: &MediatedConnection,
        institution_to_consumer: &MediatedConnection,
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

        info!("test_real_proof_with_revocation :: AS INSTITUTION SEND CREDENTIAL OFFER");
        let (address1, address2, city, state, zip) = attr_names();
        let credential_data =
            json!({address1: "123 Main St", address2: "Suite 3", city: "Draper", state: "UT", zip: "84000"})
                .to_string();

        let credential_handle = _exchange_credential(
            consumer,
            institution,
            credential_data,
            &cred_def,
            &rev_reg,
            consumer_to_institution,
            institution_to_consumer,
            None,
        )
        .await;
        (schema_id, cred_def_id, rev_reg_id, cred_def, rev_reg, credential_handle)
    }

    pub async fn verifier_create_proof_and_send_request(
        institution: &mut Faber,
        institution_to_consumer: &MediatedConnection,
        schema_id: &str,
        cred_def_id: &str,
        request_name: Option<&str>,
    ) -> Verifier {
        let _requested_attrs = requested_attrs(&institution.institution_did, &schema_id, &cred_def_id, None, None);
        let requested_attrs_string = serde_json::to_string(&_requested_attrs).unwrap();
        send_proof_request(
            institution,
            institution_to_consumer,
            &requested_attrs_string,
            "[]",
            "{}",
            request_name,
        )
        .await
    }

    pub async fn prover_select_credentials(
        prover: &mut Prover,
        alice: &mut Alice,
        connection: &MediatedConnection,
        requested_values: Option<&str>,
    ) -> SelectedCredentials {
        prover
            .update_state(
                &alice.profile.inject_anoncreds_ledger_read(),
                &alice.profile.inject_anoncreds(),
                &alice.profile.inject_wallet(),
                &alice.agency_client,
                connection,
            )
            .await
            .unwrap();
        assert_eq!(prover.get_state(), ProverState::PresentationRequestReceived);
        let retrieved_credentials = prover
            .retrieve_credentials(&alice.profile.inject_anoncreds())
            .await
            .unwrap();
        let selected_credentials = match requested_values {
            Some(requested_values) => {
                let credential_data = prover.presentation_request_data().unwrap();
                retrieved_to_selected_credentials_specific(
                    &retrieved_credentials,
                    requested_values,
                    &credential_data,
                    true,
                )
            }
            _ => retrieved_to_selected_credentials_simple(&retrieved_credentials, true),
        };

        selected_credentials
    }

    pub async fn prover_select_credentials_and_send_proof_and_assert(
        alice: &mut Alice,
        consumer_to_institution: &MediatedConnection,
        request_name: Option<&str>,
        requested_values: Option<&str>,
        expected_prover_state: ProverState,
    ) {
        let mut prover = create_proof(alice, consumer_to_institution, request_name).await;
        let selected_credentials =
            prover_select_credentials(&mut prover, alice, consumer_to_institution, requested_values).await;
        info!(
            "Prover :: Retrieved credential converted to selected: {:?}",
            &selected_credentials
        );
        generate_and_send_proof(alice, &mut prover, consumer_to_institution, selected_credentials).await;
        assert_eq!(expected_prover_state, prover.get_state());
    }

    pub async fn prover_select_credentials_and_send_proof(
        consumer: &mut Alice,
        consumer_to_institution: &MediatedConnection,
        request_name: Option<&str>,
        requested_values: Option<&str>,
    ) {
        prover_select_credentials_and_send_proof_and_assert(
            consumer,
            consumer_to_institution,
            request_name,
            requested_values,
            ProverState::PresentationSent,
        )
        .await
    }

    pub async fn prover_select_credentials_and_fail_to_generate_proof(
        consumer: &mut Alice,
        consumer_to_institution: &MediatedConnection,
        request_name: Option<&str>,
        requested_values: Option<&str>,
    ) {
        prover_select_credentials_and_send_proof_and_assert(
            consumer,
            consumer_to_institution,
            request_name,
            requested_values,
            ProverState::PresentationPreparationFailed,
        )
        .await
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
            "test_real_proof >>> retrieved matching credentials {:?}",
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

    pub fn retrieved_to_selected_credentials_specific(
        retrieved_credentials: &RetrievedCredentials,
        requested_values: &str,
        credential_data: &str,
        with_tails: bool,
    ) -> SelectedCredentials {
        info!(
            "test_real_proof >>> retrieved matching credentials {:?}",
            retrieved_credentials
        );
        let credential_data: Value = serde_json::from_str(credential_data).unwrap();
        let requested_values: Value = serde_json::from_str(requested_values).unwrap();
        let requested_attributes: &Value = &credential_data["requested_attributes"];

        let mut selected_credentials = SelectedCredentials::default();

        for (referent, cred_array) in retrieved_credentials.credentials_by_referent.iter() {
            let filtered: Vec<RetrievedCredentialForReferent> = cred_array
                .clone()
                .into_iter()
                .filter_map(|cred| {
                    let attribute_name = requested_attributes[referent]["name"].as_str().unwrap();
                    let requested_value = requested_values[attribute_name].as_str().unwrap();
                    if cred.cred_info.attributes[attribute_name] == requested_value {
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
}
