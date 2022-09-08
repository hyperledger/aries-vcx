#[cfg(feature = "test_utils")]
pub mod test_utils {
    use std::thread;
    use std::time::Duration;

    use indy_sys::WalletHandle;
    use serde_json::{json, Value};

    use aries_vcx::global::settings;
    use aries_vcx::handlers::connection::connection::{Connection, ConnectionState};
    use aries_vcx::handlers::issuance::holder::test_utils::get_credential_offer_messages;
    use aries_vcx::handlers::issuance::holder::Holder;
    use aries_vcx::handlers::issuance::issuer::test_utils::get_credential_proposal_messages;
    use aries_vcx::handlers::issuance::issuer::Issuer;
    use aries_vcx::handlers::proof_presentation::prover::test_utils::get_proof_request_messages;
    use aries_vcx::handlers::proof_presentation::prover::Prover;
    use aries_vcx::handlers::proof_presentation::verifier::Verifier;
    use aries_vcx::libindy;
    use aries_vcx::libindy::credential_def::revocation_registry::RevocationRegistry;
    use aries_vcx::libindy::credential_def::CredentialDef;
    use aries_vcx::libindy::proofs::proof_request_internal::AttrInfo;
    use aries_vcx::libindy::utils::anoncreds::test_utils::create_and_store_credential_def;
    use aries_vcx::messages::connection::invite::Invitation;
    use aries_vcx::messages::issuance::credential_offer::{CredentialOffer, OfferInfo};
    use aries_vcx::messages::issuance::credential_proposal::{CredentialProposal, CredentialProposalData};
    use aries_vcx::messages::mime_type::MimeType;
    use aries_vcx::messages::proof_presentation::presentation_proposal::{Attribute, PresentationProposalData};
    use aries_vcx::messages::proof_presentation::presentation_request::{PresentationRequest, PresentationRequestData};
    use aries_vcx::protocols::connection::invitee::state_machine::InviteeState;
    use aries_vcx::protocols::connection::inviter::state_machine::InviterState;
    use aries_vcx::protocols::issuance::holder::state_machine::HolderState;
    use aries_vcx::protocols::issuance::issuer::state_machine::IssuerState;
    use aries_vcx::protocols::proof_presentation::prover::state_machine::ProverState;
    use aries_vcx::protocols::proof_presentation::verifier::state_machine::VerifierState;
    use aries_vcx::utils::constants::{DEFAULT_PROOF_NAME, TAILS_DIR, TEST_TAILS_URL};
    use aries_vcx::utils::filters::{filter_credential_offers_by_comment, filter_proof_requests_by_name};
    use aries_vcx::utils::get_temp_dir_path;

    use crate::utils::devsetup_agent::test_utils::{Alice, Faber};
    use crate::utils::test_macros::ProofStateType;

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

    pub fn requested_attr_objects(cred_def_id: &str) -> Vec<Attribute> {
        let (address1, address2, city, state, zip) = attr_names();
        let address1_attr = Attribute::create(&address1)
            .set_cred_def_id(cred_def_id)
            .set_value("123 Main St");
        let address2_attr = Attribute::create(&address2)
            .set_cred_def_id(cred_def_id)
            .set_value("Suite 3");
        let city_attr = Attribute::create(&city)
            .set_cred_def_id(cred_def_id)
            .set_value("Draper");
        let state_attr = Attribute::create(&state).set_cred_def_id(cred_def_id).set_value("UT");
        let zip_attr = Attribute::create(&zip).set_cred_def_id(cred_def_id).set_value("84000");
        vec![address1_attr, address2_attr, city_attr, state_attr, zip_attr]
    }

    pub fn requested_attr_objects_1(cred_def_id: &str) -> Vec<Attribute> {
        let (address1, address2, city, state, zip) = attr_names();
        let address1_attr = Attribute::create(&address1)
            .set_cred_def_id(cred_def_id)
            .set_value("456 Side St");
        let address2_attr = Attribute::create(&address2)
            .set_cred_def_id(cred_def_id)
            .set_value("Suite 666");
        let city_attr = Attribute::create(&city)
            .set_cred_def_id(cred_def_id)
            .set_value("Austin");
        let state_attr = Attribute::create(&state).set_cred_def_id(cred_def_id).set_value("TC");
        let zip_attr = Attribute::create(&zip).set_cred_def_id(cred_def_id).set_value("42000");
        vec![address1_attr, address2_attr, city_attr, state_attr, zip_attr]
    }

    pub async fn create_and_send_nonrevocable_cred_offer(
        faber: &mut Faber,
        cred_def: &CredentialDef,
        connection: &Connection,
        credential_json: &str,
        comment: Option<&str>,
    ) -> Issuer {
        info!("create_and_send_nonrevocable_cred_offer >> creating issuer credential");
        let offer_info = OfferInfo {
            credential_json: credential_json.to_string(),
            cred_def_id: cred_def.cred_def_id.clone(),
            rev_reg_id: None,
            tails_file: None,
        };
        let mut issuer = Issuer::create("1").unwrap();
        info!("create_and_send_nonrevocable_cred_offer :: sending credential offer");
        issuer
            .build_credential_offer_msg(faber.wallet_handle, offer_info, comment.map(String::from))
            .await
            .unwrap();
        issuer
            .send_credential_offer(connection.send_message_closure(faber.wallet_handle).unwrap())
            .await
            .unwrap();
        info!("create_and_send_nonrevocable_cred_offer :: credential offer was sent");
        thread::sleep(Duration::from_millis(100));
        issuer
    }

    pub async fn create_and_send_cred_offer(
        faber: &mut Faber,
        cred_def: &CredentialDef,
        rev_reg: &RevocationRegistry,
        connection: &Connection,
        credential_json: &str,
        comment: Option<&str>,
    ) -> Issuer {
        info!("create_and_send_cred_offer >> creating issuer credential");
        let offer_info = OfferInfo {
            credential_json: credential_json.to_string(),
            cred_def_id: cred_def.cred_def_id.clone(),
            rev_reg_id: Some(rev_reg.get_rev_reg_id()),
            tails_file: Some(rev_reg.get_tails_dir()),
        };
        let mut issuer = Issuer::create("1").unwrap();
        info!("create_and_send_cred_offer :: sending credential offer");
        issuer
            .build_credential_offer_msg(faber.wallet_handle, offer_info, comment.map(String::from))
            .await
            .unwrap();
        issuer
            .send_credential_offer(connection.send_message_closure(faber.wallet_handle).unwrap())
            .await
            .unwrap();
        info!("create_and_send_cred_offer :: credential offer was sent");
        thread::sleep(Duration::from_millis(100));
        issuer
    }

    pub async fn send_cred_req(alice: &mut Alice, connection: &Connection, comment: Option<&str>) -> Holder {
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
        let cred_offer: CredentialOffer = serde_json::from_str(&offer).unwrap();
        let mut holder = Holder::create_from_offer("TEST_CREDENTIAL", cred_offer).unwrap();
        assert_eq!(HolderState::OfferReceived, holder.get_state());
        info!("send_cred_req :: sending credential request");
        let my_pw_did = connection.pairwise_info().pw_did.to_string();
        holder
            .send_request(
                alice.wallet_handle,
                alice.pool_handle,
                my_pw_did,
                connection.send_message_closure(alice.wallet_handle).unwrap(),
            )
            .await
            .unwrap();
        thread::sleep(Duration::from_millis(100));
        holder
    }

    pub async fn send_cred_proposal(
        alice: &mut Alice,
        connection: &Connection,
        schema_id: &str,
        cred_def_id: &str,
        comment: &str,
    ) -> Holder {
        let (address1, address2, city, state, zip) = attr_names();
        let proposal = CredentialProposalData::create()
            .set_schema_id(schema_id.to_string())
            .set_cred_def_id(cred_def_id.to_string())
            .set_comment(comment.to_string())
            .add_credential_preview_data(&address1, "123 Main St", MimeType::Plain)
            .add_credential_preview_data(&address2, "Suite 3", MimeType::Plain)
            .add_credential_preview_data(&city, "Draper", MimeType::Plain)
            .add_credential_preview_data(&state, "UT", MimeType::Plain)
            .add_credential_preview_data(&zip, "84000", MimeType::Plain);
        let mut holder = Holder::create("TEST_CREDENTIAL").unwrap();
        assert_eq!(HolderState::Initial, holder.get_state());
        holder
            .send_proposal(
                alice.wallet_handle,
                alice.pool_handle,
                proposal,
                connection.send_message_closure(alice.wallet_handle).unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(HolderState::ProposalSent, holder.get_state());
        thread::sleep(Duration::from_millis(100));
        holder
    }

    pub async fn send_cred_proposal_1(
        holder: &mut Holder,
        alice: &mut Alice,
        connection: &Connection,
        schema_id: &str,
        cred_def_id: &str,
        comment: &str,
    ) {
        holder
            .update_state(alice.wallet_handle, alice.pool_handle, &alice.agency_client, connection)
            .await
            .unwrap();
        assert_eq!(HolderState::OfferReceived, holder.get_state());
        assert!(holder.get_offer().is_ok());
        let (address1, address2, city, state, zip) = attr_names();
        let proposal = CredentialProposalData::create()
            .set_schema_id(schema_id.to_string())
            .set_cred_def_id(cred_def_id.to_string())
            .set_comment(comment.to_string())
            .add_credential_preview_data(&address1, "456 Side St", MimeType::Plain)
            .add_credential_preview_data(&address2, "Suite 666", MimeType::Plain)
            .add_credential_preview_data(&city, "Austin", MimeType::Plain)
            .add_credential_preview_data(&state, "TX", MimeType::Plain)
            .add_credential_preview_data(&zip, "42000", MimeType::Plain);
        holder
            .send_proposal(
                alice.wallet_handle,
                alice.pool_handle,
                proposal,
                connection.send_message_closure(alice.wallet_handle).unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(HolderState::ProposalSent, holder.get_state());
        thread::sleep(Duration::from_millis(100));
    }

    pub async fn accept_cred_proposal(
        faber: &mut Faber,
        connection: &Connection,
        rev_reg_id: Option<String>,
        tails_file: Option<String>,
    ) -> Issuer {
        let proposals: Vec<CredentialProposal> = serde_json::from_str(
            &get_credential_proposal_messages(&faber.agency_client, connection)
                .await
                .unwrap(),
        )
        .unwrap();
        let proposal = proposals.last().unwrap();
        let mut issuer = Issuer::create_from_proposal("TEST_CREDENTIAL", proposal).unwrap();
        assert_eq!(IssuerState::ProposalReceived, issuer.get_state());
        assert_eq!(proposal.clone(), issuer.get_proposal().unwrap());
        let offer_info = OfferInfo {
            credential_json: proposal.credential_proposal.to_string().unwrap(),
            cred_def_id: proposal.cred_def_id.clone(),
            rev_reg_id,
            tails_file,
        };
        issuer
            .build_credential_offer_msg(faber.wallet_handle, offer_info, Some("comment".into()))
            .await
            .unwrap();
        issuer
            .send_credential_offer(connection.send_message_closure(faber.wallet_handle).unwrap())
            .await
            .unwrap();
        assert_eq!(IssuerState::OfferSent, issuer.get_state());
        thread::sleep(Duration::from_millis(100));
        issuer
    }

    pub async fn accept_cred_proposal_1(
        issuer: &mut Issuer,
        faber: &mut Faber,
        connection: &Connection,
        rev_reg_id: Option<String>,
        tails_file: Option<String>,
    ) {
        assert_eq!(IssuerState::OfferSent, issuer.get_state());
        issuer
            .update_state(faber.wallet_handle, &faber.agency_client, connection)
            .await
            .unwrap();
        assert_eq!(IssuerState::ProposalReceived, issuer.get_state());
        let proposal = issuer.get_proposal().unwrap();
        let offer_info = OfferInfo {
            credential_json: proposal.credential_proposal.to_string().unwrap(),
            cred_def_id: proposal.cred_def_id.clone(),
            rev_reg_id,
            tails_file,
        };
        issuer
            .build_credential_offer_msg(faber.wallet_handle, offer_info, Some("comment".into()))
            .await
            .unwrap();
        issuer
            .send_credential_offer(connection.send_message_closure(faber.wallet_handle).unwrap())
            .await
            .unwrap();
        assert_eq!(IssuerState::OfferSent, issuer.get_state());
        thread::sleep(Duration::from_millis(100));
    }

    pub async fn accept_offer(alice: &mut Alice, connection: &Connection, holder: &mut Holder) {
        holder
            .update_state(alice.wallet_handle, alice.pool_handle, &alice.agency_client, connection)
            .await
            .unwrap();
        assert_eq!(HolderState::OfferReceived, holder.get_state());
        assert!(holder.get_offer().is_ok());
        let my_pw_did = connection.pairwise_info().pw_did.to_string();
        holder
            .send_request(
                alice.wallet_handle,
                alice.pool_handle,
                my_pw_did,
                connection.send_message_closure(alice.wallet_handle).unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(HolderState::RequestSent, holder.get_state());
    }

    pub async fn decline_offer(alice: &mut Alice, connection: &Connection, holder: &mut Holder) {
        holder
            .update_state(alice.wallet_handle, alice.pool_handle, &alice.agency_client, connection)
            .await
            .unwrap();
        assert_eq!(HolderState::OfferReceived, holder.get_state());
        holder
            .decline_offer(
                alice.wallet_handle,
                alice.pool_handle,
                Some("Have a nice day"),
                connection.send_message_closure(alice.wallet_handle).unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(HolderState::Failed, holder.get_state());
    }

    pub async fn send_credential(
        alice: &mut Alice,
        faber: &mut Faber,
        issuer_credential: &mut Issuer,
        issuer_to_consumer: &Connection,
        consumer_to_issuer: &Connection,
        holder_credential: &mut Holder,
        revokable: bool,
    ) {
        info!("send_credential >>> getting offers");
        let thread_id = issuer_credential.get_thread_id().unwrap();
        assert_eq!(IssuerState::OfferSent, issuer_credential.get_state());
        assert_eq!(issuer_credential.is_revokable(), false);
        issuer_credential
            .update_state(faber.wallet_handle, &faber.agency_client, issuer_to_consumer)
            .await
            .unwrap();
        assert_eq!(IssuerState::RequestReceived, issuer_credential.get_state());
        assert_eq!(issuer_credential.is_revokable(), false);
        assert_eq!(thread_id, issuer_credential.get_thread_id().unwrap());

        info!("send_credential >>> sending credential");
        issuer_credential
            .send_credential(
                faber.wallet_handle,
                issuer_to_consumer.send_message_closure(faber.wallet_handle).unwrap(),
            )
            .await
            .unwrap();
        thread::sleep(Duration::from_millis(100));
        assert_eq!(thread_id, issuer_credential.get_thread_id().unwrap());

        info!("send_credential >>> storing credential");
        assert_eq!(thread_id, holder_credential.get_thread_id().unwrap());
        let pool_handle = aries_vcx::global::pool::get_main_pool_handle().unwrap();
        assert_eq!(
            holder_credential.is_revokable(alice.wallet_handle, pool_handle).await.unwrap(),
            revokable
        );
        holder_credential
            .update_state(alice.wallet_handle, alice.pool_handle, &alice.agency_client, consumer_to_issuer)
            .await
            .unwrap();
        assert_eq!(HolderState::Finished, holder_credential.get_state());
        assert_eq!(
            holder_credential.is_revokable(alice.wallet_handle, pool_handle).await.unwrap(),
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

    pub async fn send_proof_proposal(alice: &mut Alice, connection: &Connection, cred_def_id: &str) -> Prover {
        let attrs = requested_attr_objects(cred_def_id);
        let mut proposal_data = PresentationProposalData::create();
        for attr in attrs.into_iter() {
            proposal_data = proposal_data.add_attribute(attr);
        }
        let mut prover = Prover::create("1").unwrap();
        prover
            .send_proposal(
                alice.wallet_handle,
                proposal_data,
                connection.send_message_closure(alice.wallet_handle).unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(prover.get_state(), ProverState::PresentationProposalSent);
        thread::sleep(Duration::from_millis(100));
        prover
    }

    pub async fn send_proof_proposal_1(
        alice: &mut Alice,
        prover: &mut Prover,
        connection: &Connection,
        cred_def_id: &str,
    ) {
        prover
            .update_state(alice.wallet_handle, &alice.agency_client, connection)
            .await
            .unwrap();
        assert_eq!(prover.get_state(), ProverState::PresentationRequestReceived);
        let attrs = requested_attr_objects_1(cred_def_id);
        let mut proposal_data = PresentationProposalData::create();
        for attr in attrs.into_iter() {
            proposal_data = proposal_data.add_attribute(attr);
        }
        prover
            .send_proposal(
                alice.wallet_handle,
                proposal_data,
                connection.send_message_closure(alice.wallet_handle).unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(prover.get_state(), ProverState::PresentationProposalSent);
        thread::sleep(Duration::from_millis(100));
    }

    pub async fn accept_proof_proposal(faber: &mut Faber, verifier: &mut Verifier, connection: &Connection) {
        verifier
            .update_state(faber.wallet_handle, &faber.agency_client, connection)
            .await
            .unwrap();
        assert_eq!(verifier.get_state(), VerifierState::PresentationProposalReceived);
        let proposal = verifier.get_presentation_proposal().unwrap();
        let attrs = proposal
            .presentation_proposal
            .attributes
            .into_iter()
            .map(|attr| AttrInfo {
                name: Some(attr.name.clone()),
                ..AttrInfo::default()
            })
            .collect();
        let presentation_request_data = PresentationRequestData::create("request-1")
            .await
            .unwrap()
            .set_requested_attributes_as_vec(attrs)
            .unwrap();
        verifier.set_request(presentation_request_data, None).unwrap();
        verifier
            .send_presentation_request(connection.send_message_closure(faber.wallet_handle).unwrap())
            .await
            .unwrap();
    }

    pub async fn reject_proof_proposal(faber: &mut Faber, connection: &Connection) -> Verifier {
        let mut verifier = Verifier::create("1").unwrap();
        verifier
            .update_state(faber.wallet_handle, &faber.agency_client, connection)
            .await
            .unwrap();
        assert_eq!(verifier.get_state(), VerifierState::PresentationProposalReceived);
        verifier
            .decline_presentation_proposal(
                faber.wallet_handle,
                connection.send_message_closure(faber.wallet_handle).unwrap(),
                "I don't like Alices",
            )
            .await
            .unwrap();
        assert_eq!(verifier.get_state(), VerifierState::Failed);
        verifier
    }

    pub async fn receive_proof_proposal_rejection(alice: &mut Alice, prover: &mut Prover, connection: &Connection) {
        assert_eq!(prover.get_state(), ProverState::PresentationProposalSent);
        prover
            .update_state(alice.wallet_handle, &alice.agency_client, connection)
            .await
            .unwrap();
        assert_eq!(prover.get_state(), ProverState::Failed);
    }

    pub async fn send_proof_request(
        faber: &mut Faber,
        connection: &Connection,
        requested_attrs: &str,
        requested_preds: &str,
        revocation_interval: &str,
        request_name: Option<&str>,
    ) -> Verifier {
        let presentation_request_data = PresentationRequestData::create(request_name.unwrap_or("name"))
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
            .send_presentation_request(connection.send_message_closure(faber.wallet_handle).unwrap())
            .await
            .unwrap();
        thread::sleep(Duration::from_millis(100));
        verifier
    }

    pub async fn create_proof_request(
        faber: &mut Faber,
        requested_attrs: &str,
        requested_preds: &str,
        revocation_interval: &str,
        request_name: Option<&str>,
    ) -> PresentationRequest {
        let presentation_request = PresentationRequestData::create(request_name.unwrap_or("name"))
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

    pub async fn create_proof(alice: &mut Alice, connection: &Connection, request_name: Option<&str>) -> Prover {
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
        let presentation_request: PresentationRequest = serde_json::from_str(&request).unwrap();
        Prover::create_from_request(DEFAULT_PROOF_NAME, presentation_request).unwrap()
    }

    pub async fn generate_and_send_proof(
        alice: &mut Alice,
        prover: &mut Prover,
        connection: &Connection,
        selected_credentials: &str,
    ) {
        let thread_id = prover.get_thread_id().unwrap();
        info!(
            "generate_and_send_proof >>> generating proof using selected credentials {}",
            selected_credentials
        );
        prover
            .generate_presentation(alice.wallet_handle, selected_credentials.into(), "{}".to_string())
            .await
            .unwrap();
        assert_eq!(thread_id, prover.get_thread_id().unwrap());
        if ProverState::PresentationPrepared == prover.get_state() {
            info!("generate_and_send_proof :: proof generated, sending proof");
            prover
                .send_presentation(
                    alice.wallet_handle,
                    connection.send_message_closure(alice.wallet_handle).unwrap(),
                )
                .await
                .unwrap();
            info!("generate_and_send_proof :: proof sent");
            assert_eq!(thread_id, prover.get_thread_id().unwrap());
            thread::sleep(Duration::from_millis(100));
        }
    }

    pub async fn verify_proof(faber: &mut Faber, verifier: &mut Verifier, connection: &Connection) {
        verifier
            .update_state(faber.wallet_handle, &faber.agency_client, &connection)
            .await
            .unwrap();
        assert_eq!(verifier.get_state(), VerifierState::Finished);
        assert_eq!(
            verifier.get_presentation_status(),
            ProofStateType::ProofValidated as u32
        );
    }

    pub async fn revoke_credential(faber: &mut Faber, issuer_credential: &Issuer, rev_reg_id: String) {
        let pool_handle = aries_vcx::global::pool::get_main_pool_handle().unwrap();
        let (_, delta, timestamp) = libindy::utils::anoncreds::get_rev_reg_delta_json(pool_handle, &rev_reg_id.clone(), None, None)
            .await
            .unwrap();
        info!("revoking credential");
        issuer_credential
            .revoke_credential(faber.wallet_handle, pool_handle, &faber.config_issuer.institution_did, true)
            .await
            .unwrap();
        let (_, delta_after_revoke, _) =
            libindy::utils::anoncreds::get_rev_reg_delta_json(pool_handle, &rev_reg_id, Some(timestamp + 1), None)
                .await
                .unwrap();
        assert_ne!(delta, delta_after_revoke);
    }

    pub async fn revoke_credential_local(faber: &mut Faber, issuer_credential: &Issuer, rev_reg_id: String) {
        let pool_handle = aries_vcx::global::pool::get_main_pool_handle().unwrap();
        let (_, delta, timestamp) = libindy::utils::anoncreds::get_rev_reg_delta_json(pool_handle, &rev_reg_id.clone(), None, None)
            .await
            .unwrap();
        info!("revoking credential locally");
        issuer_credential
            .revoke_credential(faber.wallet_handle, pool_handle, &faber.config_issuer.institution_did, false)
            .await
            .unwrap();
        let (_, delta_after_revoke, _) =
            libindy::utils::anoncreds::get_rev_reg_delta_json(pool_handle, &rev_reg_id, Some(timestamp + 1), None)
                .await
                .unwrap();
        assert_ne!(delta, delta_after_revoke); // They will not equal as we have saved the delta in cache
    }

    pub async fn rotate_rev_reg(
        faber: &mut Faber,
        credential_def: &CredentialDef,
        rev_reg: &RevocationRegistry,
    ) -> RevocationRegistry {
        let pool_handle = aries_vcx::global::pool::get_main_pool_handle().unwrap();
        let mut rev_reg_new = RevocationRegistry::create(
            faber.wallet_handle,
            &faber.config_issuer.institution_did,
            &credential_def.cred_def_id,
            &rev_reg.get_tails_dir(),
            10,
            2,
        )
        .await
        .unwrap();
        rev_reg_new
            .publish_revocation_primitives(faber.wallet_handle, pool_handle, TEST_TAILS_URL)
            .await
            .unwrap();
        rev_reg_new
    }

    pub async fn publish_revocation(institution: &mut Faber, rev_reg_id: String) {
        let pool_handle = aries_vcx::global::pool::get_main_pool_handle().unwrap();
        libindy::utils::anoncreds::publish_local_revocations(institution.wallet_handle, pool_handle, &institution.config_issuer.institution_did, rev_reg_id.as_str())
            .await
            .unwrap();
    }

    pub async fn _create_address_schema(
        wallet_handle: WalletHandle,
        institution_did: &str
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
        let (schema_id, schema_json, cred_def_id, cred_def_json, rev_reg_id, cred_def, rev_reg) =
            create_and_store_credential_def(wallet_handle, &institution_did, &attrs_list).await;
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
        consumer_to_issuer: &Connection,
        issuer_to_consumer: &Connection,
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
        issuer_credential
    }

    pub async fn _exchange_credential_with_proposal(
        consumer: &mut Alice,
        institution: &mut Faber,
        consumer_to_issuer: &Connection,
        issuer_to_consumer: &Connection,
        schema_id: &str,
        cred_def_id: &str,
        rev_reg_id: Option<String>,
        tails_file: Option<String>,
        comment: &str,
    ) -> (Holder, Issuer) {
        let mut holder = send_cred_proposal(consumer, consumer_to_issuer, schema_id, cred_def_id, comment).await;
        let mut issuer = accept_cred_proposal(institution, issuer_to_consumer, rev_reg_id, tails_file).await;
        accept_offer(consumer, consumer_to_issuer, &mut holder).await;
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
        consumer_to_institution: &Connection,
        institution_to_consumer: &Connection,
    ) -> (
        String,
        String,
        Option<String>,
        CredentialDef,
        RevocationRegistry,
        Issuer,
    ) {
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg, rev_reg_id) =
            _create_address_schema(institution.wallet_handle, &institution.config_issuer.institution_did).await;

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
        institution_to_consumer: &Connection,
        schema_id: &str,
        cred_def_id: &str,
        request_name: Option<&str>,
    ) -> Verifier {
        let _requested_attrs = requested_attrs(&institution.config_issuer.institution_did, &schema_id, &cred_def_id, None, None);
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
        connection: &Connection,
        requested_values: Option<&str>,
    ) -> String {
        prover
            .update_state(alice.wallet_handle, &alice.agency_client, connection)
            .await
            .unwrap();
        assert_eq!(prover.get_state(), ProverState::PresentationRequestReceived);
        let retrieved_credentials = prover.retrieve_credentials(alice.wallet_handle).await.unwrap();
        let selected_credentials_value = match requested_values {
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
        serde_json::to_string(&selected_credentials_value).unwrap()
    }

    pub async fn prover_select_credentials_and_send_proof_and_assert(
        alice: &mut Alice,
        consumer_to_institution: &Connection,
        request_name: Option<&str>,
        requested_values: Option<&str>,
        expected_prover_state: ProverState,
    ) {
        let mut prover = create_proof(alice, consumer_to_institution, request_name).await;
        let selected_credentials_str =
            prover_select_credentials(&mut prover, alice, consumer_to_institution, requested_values).await;
        info!(
            "Prover :: Retrieved credential converted to selected: {}",
            &selected_credentials_str
        );
        generate_and_send_proof(alice, &mut prover, consumer_to_institution, &selected_credentials_str).await;
        assert_eq!(expected_prover_state, prover.get_state());
    }

    pub async fn prover_select_credentials_and_send_proof(
        consumer: &mut Alice,
        consumer_to_institution: &Connection,
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
        consumer_to_institution: &Connection,
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
        consumer_to_institution: &mut Connection,
    ) -> Connection {
        thread::sleep(Duration::from_millis(100));
        let mut conn_requests = faber
            .agent
            .download_connection_requests(&faber.agency_client, None)
            .await
            .unwrap();
        assert_eq!(conn_requests.len(), 1);
        let mut institution_to_consumer = Connection::create_with_request(
            faber.wallet_handle,
            conn_requests.pop().unwrap(),
            &faber.agent,
            &faber.agency_client,
        )
        .await
        .unwrap();
        assert_eq!(
            ConnectionState::Inviter(InviterState::Requested),
            institution_to_consumer.get_state()
        );
        institution_to_consumer
            .find_message_and_update_state(faber.wallet_handle, &faber.agency_client)
            .await
            .unwrap();
        assert_eq!(
            ConnectionState::Inviter(InviterState::Responded),
            institution_to_consumer.get_state()
        );

        consumer_to_institution
            .find_message_and_update_state(alice.wallet_handle, &alice.agency_client)
            .await
            .unwrap();
        assert_eq!(
            ConnectionState::Invitee(InviteeState::Completed),
            consumer_to_institution.get_state()
        );

        thread::sleep(Duration::from_millis(100));
        institution_to_consumer
            .find_message_and_update_state(faber.wallet_handle, &faber.agency_client)
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
    ) -> (Connection, Connection) {
        let public_invite_json = institution.create_public_invite().unwrap();
        let public_invite: Invitation = serde_json::from_str(&public_invite_json).unwrap();

        let mut consumer_to_institution = Connection::create_with_invite(
            "institution",
            alice.wallet_handle,
            &alice.agency_client,
            public_invite,
            true,
        )
        .await
        .unwrap();
        consumer_to_institution
            .connect(alice.wallet_handle, &alice.agency_client)
            .await
            .unwrap();
        consumer_to_institution
            .find_message_and_update_state(alice.wallet_handle, &alice.agency_client)
            .await
            .unwrap();

        let institution_to_consumer =
            connect_using_request_sent_to_public_agent(alice, institution, &mut consumer_to_institution).await;
        (consumer_to_institution, institution_to_consumer)
    }

    pub async fn create_connected_connections(alice: &mut Alice, faber: &mut Faber) -> (Connection, Connection) {
        debug!("Institution is going to create connection.");
        let mut institution_to_consumer =
            Connection::create("consumer", faber.wallet_handle, &faber.agency_client, true)
                .await
                .unwrap();
        institution_to_consumer
            .connect(faber.wallet_handle, &faber.agency_client)
            .await
            .unwrap();
        let details = institution_to_consumer.get_invite_details().unwrap();

        debug!("Consumer is going to accept connection invitation.");
        let mut consumer_to_institution = Connection::create_with_invite(
            "institution",
            alice.wallet_handle,
            &alice.agency_client,
            details.clone(),
            true,
        )
        .await
        .unwrap();

        consumer_to_institution
            .connect(alice.wallet_handle, &alice.agency_client)
            .await
            .unwrap();
        consumer_to_institution
            .find_message_and_update_state(alice.wallet_handle, &alice.agency_client)
            .await
            .unwrap();

        let thread_id = consumer_to_institution.get_thread_id();

        debug!("Institution is going to process connection request.");
        thread::sleep(Duration::from_millis(100));
        institution_to_consumer
            .find_message_and_update_state(faber.wallet_handle, &faber.agency_client)
            .await
            .unwrap();
        assert_eq!(
            ConnectionState::Inviter(InviterState::Responded),
            institution_to_consumer.get_state()
        );
        assert_eq!(thread_id, institution_to_consumer.get_thread_id());

        debug!("Consumer is going to complete the connection protocol.");
        consumer_to_institution
            .find_message_and_update_state(alice.wallet_handle, &alice.agency_client)
            .await
            .unwrap();
        assert_eq!(
            ConnectionState::Invitee(InviteeState::Completed),
            consumer_to_institution.get_state()
        );
        assert_eq!(thread_id, consumer_to_institution.get_thread_id());

        debug!("Institution is going to complete the connection protocol.");
        thread::sleep(Duration::from_millis(100));
        institution_to_consumer
            .find_message_and_update_state(faber.wallet_handle, &faber.agency_client)
            .await
            .unwrap();
        assert_eq!(
            ConnectionState::Inviter(InviterState::Completed),
            institution_to_consumer.get_state()
        );
        assert_eq!(thread_id, consumer_to_institution.get_thread_id());

        (consumer_to_institution, institution_to_consumer)
    }

    pub fn retrieved_to_selected_credentials_simple(retrieved_credentials: &str, with_tails: bool) -> Value {
        info!(
            "test_real_proof >>> retrieved matching credentials {}",
            retrieved_credentials
        );
        let data: Value = serde_json::from_str(retrieved_credentials).unwrap();
        let mut credentials_mapped: Value = json!({"attrs":{}});

        for (key, val) in data["attrs"].as_object().unwrap().iter() {
            let cred_array = val.as_array().unwrap();
            if cred_array.len() > 0 {
                let first_cred = &cred_array[0];
                credentials_mapped["attrs"][key]["credential"] = first_cred.clone();
                if with_tails {
                    credentials_mapped["attrs"][key]["tails_file"] =
                        Value::from(get_temp_dir_path(TAILS_DIR).to_str().unwrap());
                }
            }
        }
        return credentials_mapped;
    }

    pub fn retrieved_to_selected_credentials_specific(
        retrieved_credentials: &str,
        requested_values: &str,
        credential_data: &str,
        with_tails: bool,
    ) -> Value {
        info!(
            "test_real_proof >>> retrieved matching credentials {}",
            retrieved_credentials
        );
        let retrieved_credentials: Value = serde_json::from_str(retrieved_credentials).unwrap();
        let credential_data: Value = serde_json::from_str(credential_data).unwrap();
        let requested_values: Value = serde_json::from_str(requested_values).unwrap();
        let requested_attributes: &Value = &credential_data["requested_attributes"];
        let mut credentials_mapped: Value = json!({"attrs":{}});

        for (key, val) in retrieved_credentials["attrs"].as_object().unwrap().iter() {
            let filtered: Vec<&Value> = val
                .as_array()
                .unwrap()
                .into_iter()
                .filter_map(|cred| {
                    let attribute_name = requested_attributes[key]["name"].as_str().unwrap();
                    let requested_value = requested_values[attribute_name].as_str().unwrap();
                    if cred["cred_info"]["attrs"][attribute_name].as_str().unwrap() == requested_value {
                        Some(cred)
                    } else {
                        None
                    }
                })
                .collect();
            let first_cred: &serde_json::Value = &filtered[0];
            credentials_mapped["attrs"][key]["credential"] = first_cred.clone();
            if with_tails {
                credentials_mapped["attrs"][key]["tails_file"] =
                    Value::from(get_temp_dir_path(TAILS_DIR).to_str().unwrap());
            }
        }
        return credentials_mapped;
    }
}
