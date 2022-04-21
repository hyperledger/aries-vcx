#[macro_use]
extern crate log;
extern crate serde;
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tokio;
extern crate async_trait;

use std::fmt;

pub mod utils;

macro_rules! enum_number {
    ($name:ident { $($variant:ident = $value:expr, )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            $($variant = $value,)*
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: ::serde::Serializer
            {
                // Serialize the enum as a u64.
                serializer.serialize_u64(*self as u64)
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: ::serde::Deserializer<'de>
            {
                struct Visitor;

                impl<'de> ::serde::de::Visitor<'de> for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("positive integer")
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<$name, E>
                        where E: ::serde::de::Error
                    {
                        // Rust does not come with a simple way of converting a
                        // number to an enum, so use a big `match`.
                        match value {
                            $( $value => Ok($name::$variant), )*
                            _ => Err(E::custom(
                                format!("unknown {} value: {}",
                                stringify!($name), value))),
                        }
                    }
                }

                // Deserialize the enum from a u64.
                deserializer.deserialize_u64(Visitor)
            }
        }
    }
}

enum_number!(ProofStateType
{
    ProofUndefined = 0,
    ProofValidated = 1,
    ProofInvalid = 2,
});

#[allow(unused_imports)]
#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use std::thread;
    use std::time::Duration;

    use rand::Rng;
    use serde_json::Value;

    use aries_vcx::{libindy, utils};
    use aries_vcx::agency_client::get_message::download_messages_noauth;
    use aries_vcx::agency_client::MessageStatusCode;
    use aries_vcx::agency_client::mocking::AgencyMockDecrypted;
    use aries_vcx::agency_client::payload::PayloadKinds;
    use aries_vcx::agency_client::update_message::{UIDsByConn, update_agency_messages};
    use aries_vcx::error::VcxResult;
    use aries_vcx::handlers::connection::connection::{Connection, ConnectionState};
    use aries_vcx::handlers::issuance::holder::Holder;
    use aries_vcx::handlers::issuance::holder::test_utils::get_credential_offer_messages;
    use aries_vcx::handlers::issuance::issuer::{Issuer, IssuerConfig};
    use aries_vcx::handlers::issuance::issuer::test_utils::get_credential_proposal_messages;
    use aries_vcx::handlers::out_of_band::{GoalCode, HandshakeProtocol, OutOfBandInvitation};
    use aries_vcx::handlers::out_of_band::receiver::OutOfBandReceiver;
    use aries_vcx::handlers::out_of_band::sender::OutOfBandSender;
    use aries_vcx::handlers::proof_presentation::prover::Prover;
    use aries_vcx::handlers::proof_presentation::prover::test_utils::get_proof_request_messages;
    use aries_vcx::handlers::proof_presentation::verifier::Verifier;
    use aries_vcx::libindy::credential_def::{CredentialDef, CredentialDefConfigBuilder, RevocationDetailsBuilder};
    use aries_vcx::libindy::proofs::proof_request_internal::{AttrInfo, NonRevokedInterval, PredicateInfo};
    use aries_vcx::libindy::utils::anoncreds::test_utils::create_and_write_test_schema;
    use aries_vcx::libindy::utils::wallet::*;
    use aries_vcx::messages::a2a::A2AMessage;
    use aries_vcx::messages::ack::test_utils::_ack;
    use aries_vcx::messages::connection::invite::Invitation;
    use aries_vcx::messages::connection::service::FullService;
    use aries_vcx::messages::connection::service::ServiceResolvable;
    use aries_vcx::messages::issuance::credential_offer::{CredentialOffer, OfferInfo};
    use aries_vcx::messages::issuance::credential_proposal::{CredentialProposal, CredentialProposalData};
    use aries_vcx::messages::mime_type::MimeType;
    use aries_vcx::messages::proof_presentation::presentation_proposal::{Attribute, PresentationProposal, PresentationProposalData};
    use aries_vcx::messages::proof_presentation::presentation_request::{PresentationRequest, PresentationRequestData};
    use aries_vcx::protocols::connection::invitee::state_machine::InviteeState;
    use aries_vcx::protocols::connection::inviter::state_machine::InviterState;
    use aries_vcx::protocols::issuance::holder::state_machine::HolderState;
    use aries_vcx::protocols::issuance::issuer::state_machine::IssuerState;
    use aries_vcx::protocols::proof_presentation::prover::state_machine::ProverState;
    use aries_vcx::protocols::proof_presentation::verifier::state_machine::VerifierState;
    use aries_vcx::settings;
    use aries_vcx::utils::{
        constants::{TAILS_DIR, TEST_TAILS_URL},
        get_temp_dir_path,
    };
    use aries_vcx::utils::constants;
    use aries_vcx::utils::devsetup::*;
    use aries_vcx::utils::filters;
    use aries_vcx::utils::mockdata::mockdata_connection::{ARIES_CONNECTION_ACK, ARIES_CONNECTION_INVITATION, ARIES_CONNECTION_REQUEST, CONNECTION_SM_INVITEE_COMPLETED, CONNECTION_SM_INVITEE_INVITED, CONNECTION_SM_INVITEE_REQUESTED, CONNECTION_SM_INVITER_COMPLETED};
    use aries_vcx::utils::mockdata::mockdata_proof::REQUESTED_ATTRIBUTES;

    use crate::utils::devsetup_agent::test::{Alice, Faber, TestAgent};

    use super::*;

    pub async fn create_and_store_credential_def(attr_list: &str, support_rev: bool) -> (String, String, String, String, CredentialDef, Option<String>) {
        let (schema_id, schema_json) = create_and_write_test_schema(attr_list).await;
        let config = CredentialDefConfigBuilder::default()
            .issuer_did(settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap())
            .schema_id(&schema_id)
            .tag("1")
            .build()
            .unwrap();
        let (revocation_details, tails_url) = if support_rev {
            (RevocationDetailsBuilder::default()
                 .support_revocation(support_rev)
                 .tails_dir(get_temp_dir_path(TAILS_DIR).to_str().unwrap())
                 .max_creds(10 as u32)
                 .build()
                 .unwrap(),
             Some(TEST_TAILS_URL))
        } else {
            (RevocationDetailsBuilder::default()
                 .support_revocation(support_rev)
                 .build()
                 .unwrap(),
             None)
        };
        let mut cred_def = CredentialDef::create_and_store("1".to_string(),
                                                           config,
                                                           revocation_details).await.unwrap()
            .publish_cred_def().await.unwrap();
        if let Some(tails_url) = tails_url {
            cred_def.publish_revocation_primitives(tails_url).await.unwrap();
        }
        thread::sleep(Duration::from_millis(1000));
        let cred_def_id = cred_def.get_cred_def_id();
        thread::sleep(Duration::from_millis(1000));
        let (_, cred_def_json) = libindy::utils::anoncreds::get_cred_def_json(&cred_def_id).await.unwrap();
        let rev_reg_id = cred_def.get_rev_reg_id();
        (schema_id, schema_json, cred_def_id.to_string(), cred_def_json, cred_def, rev_reg_id)
    }

    fn attr_names() -> (String, String, String, String, String) {
        let address1 = "Address1".to_string();
        let address2 = "address2".to_string();
        let city = "CITY".to_string();
        let state = "State".to_string();
        let zip = "zip".to_string();
        (address1, address2, city, state, zip)
    }

    fn requested_attrs(did: &str, schema_id: &str, cred_def_id: &str, from: Option<u64>, to: Option<u64>) -> Value {
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

    fn requested_attr_objects(cred_def_id: &str) -> Vec<Attribute> {
        let (address1, address2, city, state, zip) = attr_names();
        let address1_attr = Attribute::create(&address1).set_cred_def_id(cred_def_id).set_value("123 Main St");
        let address2_attr = Attribute::create(&address2).set_cred_def_id(cred_def_id).set_value("Suite 3");
        let city_attr = Attribute::create(&city).set_cred_def_id(cred_def_id).set_value("Draper");
        let state_attr = Attribute::create(&state).set_cred_def_id(cred_def_id).set_value("UT");
        let zip_attr = Attribute::create(&zip).set_cred_def_id(cred_def_id).set_value("84000");
        vec![address1_attr, address2_attr, city_attr, state_attr, zip_attr]
    }

    fn requested_attr_objects_1(cred_def_id: &str) -> Vec<Attribute> {
        let (address1, address2, city, state, zip) = attr_names();
        let address1_attr = Attribute::create(&address1).set_cred_def_id(cred_def_id).set_value("456 Side St");
        let address2_attr = Attribute::create(&address2).set_cred_def_id(cred_def_id).set_value("Suite 666");
        let city_attr = Attribute::create(&city).set_cred_def_id(cred_def_id).set_value("Austin");
        let state_attr = Attribute::create(&state).set_cred_def_id(cred_def_id).set_value("TC");
        let zip_attr = Attribute::create(&zip).set_cred_def_id(cred_def_id).set_value("42000");
        vec![address1_attr, address2_attr, city_attr, state_attr, zip_attr]
    }

    async fn create_and_send_cred_offer(faber: &mut Faber, cred_def: &CredentialDef, connection: &Connection, credential_json: &str, comment: Option<&str>) -> Issuer {
        faber.activate().await.unwrap();
        info!("create_and_send_cred_offer >> creating issuer credential");
        let offer_info = OfferInfo {
            credential_json: credential_json.to_string(),
            cred_def_id: cred_def.get_cred_def_id(),
            rev_reg_id: cred_def.get_rev_reg_id(),
            tails_file: cred_def.get_tails_dir(),
        };
        let mut issuer = Issuer::create("1").unwrap();
        info!("create_and_send_cred_offer :: sending credential offer");
        issuer.build_credential_offer_msg(offer_info, comment.map(String::from)).await.unwrap();
        issuer.send_credential_offer(connection.send_message_closure().unwrap()).await.unwrap();
        info!("create_and_send_cred_offer :: credential offer was sent");
        thread::sleep(Duration::from_millis(2000));
        issuer
    }

    async fn send_cred_req(alice: &mut Alice, connection: &Connection, comment: Option<&str>) -> Holder {
        info!("send_cred_req >>> switching to consumer");
        alice.activate().await.unwrap();
        info!("send_cred_req :: getting offers");
        let credential_offers = get_credential_offer_messages(connection).await.unwrap();
        let credential_offers = match comment {
            Some(comment) => {
                let filtered = filters::filter_credential_offers_by_comment(&credential_offers, comment).unwrap();
                info!("send_cred_req :: credential offer  messages filtered by comment {}: {}", comment, filtered);
                filtered
            }
            _ => credential_offers.to_string()
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
        holder.send_request(my_pw_did, connection.send_message_closure().unwrap()).await.unwrap();
        thread::sleep(Duration::from_millis(2000));
        holder
    }

    async fn send_cred_proposal(alice: &mut Alice, connection: &Connection, schema_id: &str, cred_def_id: &str, comment: &str) -> Holder {
        alice.activate().await.unwrap();
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
        holder.send_proposal(proposal, connection.send_message_closure().unwrap()).await.unwrap();
        assert_eq!(HolderState::ProposalSent, holder.get_state());
        thread::sleep(Duration::from_millis(1000));
        holder
    }

    async fn send_cred_proposal_1(holder: &mut Holder, alice: &mut Alice, connection: &Connection, schema_id: &str, cred_def_id: &str, comment: &str) {
        alice.activate().await.unwrap();
        holder.update_state(connection).await.unwrap();
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
        holder.send_proposal(proposal, connection.send_message_closure().unwrap()).await.unwrap();
        assert_eq!(HolderState::ProposalSent, holder.get_state());
        thread::sleep(Duration::from_millis(1000));
    }

    async fn accept_cred_proposal(faber: &mut Faber, connection: &Connection, rev_reg_id: Option<String>, tails_file: Option<String>) -> Issuer {
        faber.activate().await.unwrap();
        let proposals: Vec<CredentialProposal> = serde_json::from_str(&get_credential_proposal_messages(connection).await.unwrap()).unwrap();
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
        issuer.build_credential_offer_msg(offer_info, Some("comment".into())).await.unwrap();
        issuer.send_credential_offer(connection.send_message_closure().unwrap()).await.unwrap();
        assert_eq!(IssuerState::OfferSent, issuer.get_state());
        thread::sleep(Duration::from_millis(1000));
        issuer
    }

    async fn accept_cred_proposal_1(issuer: &mut Issuer, faber: &mut Faber, connection: &Connection, rev_reg_id: Option<String>, tails_file: Option<String>) {
        faber.activate().await.unwrap();
        assert_eq!(IssuerState::OfferSent, issuer.get_state());
        issuer.update_state(connection).await.unwrap();
        assert_eq!(IssuerState::ProposalReceived, issuer.get_state());
        let proposal = issuer.get_proposal().unwrap();
        let offer_info = OfferInfo {
            credential_json: proposal.credential_proposal.to_string().unwrap(),
            cred_def_id: proposal.cred_def_id.clone(),
            rev_reg_id,
            tails_file,
        };
        issuer.build_credential_offer_msg(offer_info, Some("comment".into())).await.unwrap();
        issuer.send_credential_offer(connection.send_message_closure().unwrap()).await.unwrap();
        assert_eq!(IssuerState::OfferSent, issuer.get_state());
        thread::sleep(Duration::from_millis(1000));
    }

    async fn accept_offer(alice: &mut Alice, connection: &Connection, holder: &mut Holder) {
        alice.activate().await.unwrap();
        holder.update_state(connection).await.unwrap();
        assert_eq!(HolderState::OfferReceived, holder.get_state());
        assert!(holder.get_offer().is_ok());
        let my_pw_did = connection.pairwise_info().pw_did.to_string();
        holder.send_request(my_pw_did, connection.send_message_closure().unwrap()).await.unwrap();
        assert_eq!(HolderState::RequestSent, holder.get_state());
    }

    async fn decline_offer(alice: &mut Alice, connection: &Connection, holder: &mut Holder) {
        alice.activate().await.unwrap();
        holder.update_state(connection).await.unwrap();
        assert_eq!(HolderState::OfferReceived, holder.get_state());
        holder.decline_offer(Some("Have a nice day"), connection.send_message_closure().unwrap()).await.unwrap();
        assert_eq!(HolderState::Failed, holder.get_state());
    }

    async fn send_credential(consumer: &mut Alice, institution: &mut Faber, issuer_credential: &mut Issuer, issuer_to_consumer: &Connection, consumer_to_issuer: &Connection, holder_credential: &mut Holder, revokable: bool) {
        institution.activate().await.unwrap();
        info!("send_credential >>> getting offers");
        let thread_id = issuer_credential.get_thread_id().unwrap();
        assert_eq!(IssuerState::OfferSent, issuer_credential.get_state());
        assert_eq!(issuer_credential.is_revokable(), false);
        issuer_credential.update_state(issuer_to_consumer).await.unwrap();
        assert_eq!(IssuerState::RequestReceived, issuer_credential.get_state());
        assert_eq!(issuer_credential.is_revokable(), false);
        assert_eq!(thread_id, issuer_credential.get_thread_id().unwrap());

        info!("send_credential >>> sending credential");
        issuer_credential.send_credential(issuer_to_consumer.send_message_closure().unwrap()).await.unwrap();
        thread::sleep(Duration::from_millis(2000));
        assert_eq!(thread_id, issuer_credential.get_thread_id().unwrap());

        consumer.activate().await.unwrap();
        info!("send_credential >>> storing credential");
        assert_eq!(thread_id, holder_credential.get_thread_id().unwrap());
        assert_eq!(holder_credential.is_revokable().await.unwrap(), revokable);
        holder_credential.update_state(consumer_to_issuer).await.unwrap();
        assert_eq!(HolderState::Finished, holder_credential.get_state());
        assert_eq!(holder_credential.is_revokable().await.unwrap(), revokable);
        assert_eq!(thread_id, holder_credential.get_thread_id().unwrap());

        if revokable {
            thread::sleep(Duration::from_millis(2000));
            assert_eq!(holder_credential.get_tails_location().unwrap(), TEST_TAILS_URL.to_string());
        }
    }

    async fn send_proof_proposal(alice: &mut Alice, connection: &Connection, cred_def_id: &str) -> Prover {
        alice.activate().await.unwrap();
        let attrs = requested_attr_objects(cred_def_id);
        let mut proposal_data = PresentationProposalData::create();
        for attr in attrs.into_iter() {
            proposal_data = proposal_data.add_attribute(attr);
        }
        let mut prover = Prover::create("1").unwrap();
        prover.send_proposal(proposal_data, connection.send_message_closure().unwrap()).await.unwrap();
        assert_eq!(prover.get_state(), ProverState::PresentationProposalSent);
        thread::sleep(Duration::from_millis(1000));
        prover
    }

    async fn send_proof_proposal_1(alice: &mut Alice, prover: &mut Prover, connection: &Connection, cred_def_id: &str) {
        alice.activate().await.unwrap();
        prover.update_state(connection).await.unwrap();
        assert_eq!(prover.get_state(), ProverState::PresentationRequestReceived);
        let attrs = requested_attr_objects_1(cred_def_id);
        let mut proposal_data = PresentationProposalData::create();
        for attr in attrs.into_iter() {
            proposal_data = proposal_data.add_attribute(attr);
        }
        prover.send_proposal(proposal_data, connection.send_message_closure().unwrap()).await.unwrap();
        assert_eq!(prover.get_state(), ProverState::PresentationProposalSent);
        thread::sleep(Duration::from_millis(1000));
    }

    async fn accept_proof_proposal(faber: &mut Faber, verifier: &mut Verifier, connection: &Connection) {
        faber.activate().await.unwrap();
        verifier.update_state(connection).await.unwrap();
        assert_eq!(verifier.get_state(), VerifierState::PresentationProposalReceived);
        let proposal = verifier.get_presentation_proposal().unwrap();
        let attrs = proposal.presentation_proposal.attributes.into_iter().map(|attr| {
            AttrInfo {
                name: Some(attr.name.clone()),
                ..AttrInfo::default()
            }
        }).collect();
        let presentation_request_data =
            PresentationRequestData::create("request-1").await.unwrap()
                .set_requested_attributes_as_vec(attrs).unwrap();
        verifier.set_request(presentation_request_data, None).unwrap();
        verifier.send_presentation_request(connection.send_message_closure().unwrap()).await.unwrap();
    }

    async fn reject_proof_proposal(faber: &mut Faber, connection: &Connection) -> Verifier {
        faber.activate().await.unwrap();
        let mut verifier = Verifier::create("1").unwrap();
        verifier.update_state(connection).await.unwrap();
        assert_eq!(verifier.get_state(), VerifierState::PresentationProposalReceived);
        verifier.decline_presentation_proposal(connection.send_message_closure().unwrap(), "I don't like Alices").await.unwrap();
        assert_eq!(verifier.get_state(), VerifierState::Failed);
        verifier
    }

    async fn receive_proof_proposal_rejection(alice: &mut Alice, prover: &mut Prover, connection: &Connection) {
        alice.activate().await.unwrap();
        assert_eq!(prover.get_state(), ProverState::PresentationProposalSent);
        prover.update_state(connection).await.unwrap();
        assert_eq!(prover.get_state(), ProverState::Failed);
    }

    async fn send_proof_request(faber: &mut Faber, connection: &Connection, requested_attrs: &str, requested_preds: &str, revocation_interval: &str, request_name: Option<&str>) -> Verifier {
        faber.activate().await.unwrap();
        let presentation_request_data =
            PresentationRequestData::create(request_name.unwrap_or("name")).await.unwrap()
                .set_requested_attributes_as_string(requested_attrs.to_string()).unwrap()
                .set_requested_predicates_as_string(requested_preds.to_string()).unwrap()
                .set_not_revoked_interval(revocation_interval.to_string()).unwrap();
        let mut verifier = Verifier::create_from_request("1".to_string(), &presentation_request_data).unwrap();
        verifier.send_presentation_request(connection.send_message_closure().unwrap()).await.unwrap();
        thread::sleep(Duration::from_millis(2000));
        verifier
    }

    async fn create_proof_request(faber: &mut Faber, requested_attrs: &str, requested_preds: &str, revocation_interval: &str, request_name: Option<&str>) -> PresentationRequest {
        faber.activate().await.unwrap();
        let presentation_request =
            PresentationRequestData::create(request_name.unwrap_or("name")).await.unwrap()
                .set_requested_attributes_as_string(requested_attrs.to_string()).unwrap()
                .set_requested_predicates_as_string(requested_preds.to_string()).unwrap()
                .set_not_revoked_interval(revocation_interval.to_string()).unwrap();
        let verifier = Verifier::create_from_request("1".to_string(), &presentation_request).unwrap();
        verifier.get_presentation_request().unwrap()
    }

    async fn create_proof(alice: &mut Alice, connection: &Connection, request_name: Option<&str>) -> Prover {
        alice.activate().await.unwrap();
        info!("create_proof >>> getting proof request messages");
        let requests = {
            let _requests = get_proof_request_messages(connection).await.unwrap();
            info!("create_proof :: get proof request messages returned {}", _requests);
            match request_name {
                Some(request_name) => {
                    let filtered = filters::filter_proof_requests_by_name(&_requests, request_name).unwrap();
                    info!("create_proof :: proof request messages filtered by name {}: {}", request_name, filtered);
                    filtered
                }
                _ => _requests.to_string()
            }
        };
        let requests: Value = serde_json::from_str(&requests).unwrap();
        let requests = requests.as_array().unwrap();
        assert_eq!(requests.len(), 1);
        let request = serde_json::to_string(&requests[0]).unwrap();
        let presentation_request: PresentationRequest = serde_json::from_str(&request).unwrap();
        Prover::create_from_request(utils::constants::DEFAULT_PROOF_NAME, presentation_request).unwrap()
    }

    async fn generate_and_send_proof(alice: &mut Alice, prover: &mut Prover, connection: &Connection, selected_credentials: &str) {
        alice.activate().await.unwrap();
        let thread_id = prover.get_thread_id().unwrap();
        info!("generate_and_send_proof >>> generating proof using selected credentials {}", selected_credentials);
        prover.generate_presentation(selected_credentials.into(), "{}".to_string()).await.unwrap();
        assert_eq!(thread_id, prover.get_thread_id().unwrap());
        if ProverState::PresentationPrepared == prover.get_state() {
            info!("generate_and_send_proof :: proof generated, sending proof");
            prover.send_presentation(connection.send_message_closure().unwrap()).await.unwrap();
            info!("generate_and_send_proof :: proof sent");
            assert_eq!(thread_id, prover.get_thread_id().unwrap());
            thread::sleep(Duration::from_millis(5000));
        }
    }

    async fn verify_proof(institution: &mut Faber, verifier: &mut Verifier, connection: &Connection) {
        institution.activate().await.unwrap();
        verifier.update_state(&connection).await.unwrap();
        assert_eq!(verifier.get_state(), VerifierState::Finished);
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    async fn revoke_credential(faber: &mut Faber, issuer_credential: &Issuer, rev_reg_id: Option<String>) {
        faber.activate().await.unwrap();
        // GET REV REG DELTA BEFORE REVOCATION
        let (_, delta, timestamp) = libindy::utils::anoncreds::get_rev_reg_delta_json(&rev_reg_id.clone().unwrap(), None, None).await.unwrap();
        info!("revoking credential");
        issuer_credential.revoke_credential(true).await.unwrap();
        let (_, delta_after_revoke, _) = libindy::utils::anoncreds::get_rev_reg_delta_json(&rev_reg_id.unwrap(), Some(timestamp + 1), None).await.unwrap();
        assert_ne!(delta, delta_after_revoke);
    }

    async fn revoke_credential_local(faber: &mut Faber, issuer_credential: &Issuer, rev_reg_id: Option<String>) {
        faber.activate().await.unwrap();
        let (_, delta, timestamp) = libindy::utils::anoncreds::get_rev_reg_delta_json(&rev_reg_id.clone().unwrap(), None, None).await.unwrap();
        info!("revoking credential locally");
        issuer_credential.revoke_credential(false).await.unwrap();
        let (_, delta_after_revoke, _) = libindy::utils::anoncreds::get_rev_reg_delta_json(&rev_reg_id.unwrap(), Some(timestamp + 1), None).await.unwrap();
        assert_ne!(delta, delta_after_revoke); // They will not equal as we have saved the delta in cache
    }

    async fn rotate_rev_reg(faber: &mut Faber, cred_def: &mut CredentialDef) {
        faber.activate().await.unwrap();
        let revocation_details = RevocationDetailsBuilder::default()
            .support_revocation(true)
            .tails_dir(get_temp_dir_path(TAILS_DIR).to_str().unwrap())
            .max_creds(10 as u32)
            .build()
            .unwrap();
        cred_def.rotate_rev_reg(revocation_details).await.unwrap();
        cred_def.publish_revocation_primitives(TEST_TAILS_URL).await.unwrap();
    }

    async fn publish_revocation(institution: &mut Faber, rev_reg_id: String) {
        institution.activate().await.unwrap();
        libindy::utils::anoncreds::publish_local_revocations(rev_reg_id.as_str()).await.unwrap();
    }

    async fn _create_address_schema() -> (String, String, String, String, CredentialDef, Option<String>) {
        info!("test_real_proof_with_revocation >>> CREATE SCHEMA AND CRED DEF");
        let attrs_list = json!(["address1", "address2", "city", "state", "zip"]).to_string();
        create_and_store_credential_def(&attrs_list, true).await
    }

    async fn _exchange_credential(consumer: &mut Alice, institution: &mut Faber, credential_data: String, cred_def: &CredentialDef, consumer_to_issuer: &Connection, issuer_to_consumer: &Connection, comment: Option<&str>) -> Issuer {
        info!("Generated credential data: {}", credential_data);
        let mut issuer_credential = create_and_send_cred_offer(institution, cred_def, issuer_to_consumer, &credential_data, comment).await;
        info!("AS CONSUMER SEND CREDENTIAL REQUEST");
        let mut holder_credential = send_cred_req(consumer, consumer_to_issuer, comment).await;
        info!("AS INSTITUTION SEND CREDENTIAL");
        send_credential(consumer, institution, &mut issuer_credential, issuer_to_consumer, consumer_to_issuer, &mut holder_credential, true).await;
        issuer_credential
    }

    async fn _exchange_credential_with_proposal(consumer: &mut Alice, institution: &mut Faber, consumer_to_issuer: &Connection, issuer_to_consumer: &Connection, schema_id: &str, cred_def_id: &str, rev_reg_id: Option<String>, tails_file: Option<String>, comment: &str) -> (Holder, Issuer) {
        let mut holder = send_cred_proposal(consumer, consumer_to_issuer, schema_id, cred_def_id, comment).await;
        let mut issuer = accept_cred_proposal(institution, issuer_to_consumer, rev_reg_id, tails_file).await;
        accept_offer(consumer, consumer_to_issuer, &mut holder).await;
        send_credential(consumer, institution, &mut issuer, issuer_to_consumer, consumer_to_issuer, &mut holder, true).await;
        (holder, issuer)
    }

    async fn issue_address_credential(consumer: &mut Alice, institution: &mut Faber, consumer_to_institution: &Connection, institution_to_consumer: &Connection) -> (String, String, Option<String>, CredentialDef, Issuer) {
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg_id) = _create_address_schema().await;

        info!("test_real_proof_with_revocation :: AS INSTITUTION SEND CREDENTIAL OFFER");
        let (address1, address2, city, state, zip) = attr_names();
        let credential_data = json!({address1: "123 Main St", address2: "Suite 3", city: "Draper", state: "UT", zip: "84000"}).to_string();

        let credential_handle = _exchange_credential(consumer, institution, credential_data, &cred_def, consumer_to_institution, institution_to_consumer, None).await;
        (schema_id, cred_def_id, rev_reg_id, cred_def, credential_handle)
    }

    async fn verifier_create_proof_and_send_request(institution: &mut Faber, institution_to_consumer: &Connection, schema_id: &str, cred_def_id: &str, request_name: Option<&str>) -> Verifier {
        institution.activate().await.unwrap();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let _requested_attrs = requested_attrs(&institution_did, &schema_id, &cred_def_id, None, None);
        let requested_attrs_string = serde_json::to_string(&_requested_attrs).unwrap();
        send_proof_request(institution, institution_to_consumer, &requested_attrs_string, "[]", "{}", request_name).await
    }

    async fn prover_select_credentials(
        prover: &mut Prover,
        consumer: &mut Alice,
        connection: &Connection,
        requested_values: Option<&str>) -> String {
        consumer.activate().await.unwrap();
        prover.update_state(connection).await.unwrap();
        assert_eq!(prover.get_state(), ProverState::PresentationRequestReceived);
        let retrieved_credentials = prover.retrieve_credentials().await.unwrap();
        let selected_credentials_value = match requested_values {
            Some(requested_values) => {
                let credential_data = prover.presentation_request_data().unwrap();
                retrieved_to_selected_credentials_specific(&retrieved_credentials, requested_values, &credential_data, true)
            }
            _ => retrieved_to_selected_credentials_simple(&retrieved_credentials, true)
        };
        serde_json::to_string(&selected_credentials_value).unwrap()
    }

    async fn prover_select_credentials_and_send_proof_and_assert(
        consumer: &mut Alice,
        consumer_to_institution: &Connection,
        request_name: Option<&str>,
        requested_values: Option<&str>,
        expected_prover_state: ProverState,
    ) {
        consumer.activate().await.unwrap();
        let mut prover = create_proof(consumer, consumer_to_institution, request_name).await;
        let selected_credentials_str = prover_select_credentials(&mut prover, consumer, consumer_to_institution, requested_values).await;
        info!("Prover :: Retrieved credential converted to selected: {}", &selected_credentials_str);
        generate_and_send_proof(consumer, &mut prover, consumer_to_institution, &selected_credentials_str).await;
        assert_eq!(expected_prover_state, prover.get_state());
    }

    async fn prover_select_credentials_and_send_proof(consumer: &mut Alice, consumer_to_institution: &Connection, request_name: Option<&str>, requested_values: Option<&str>) {
        prover_select_credentials_and_send_proof_and_assert(consumer, consumer_to_institution, request_name, requested_values, ProverState::PresentationSent).await
    }

    async fn prover_select_credentials_and_fail_to_generate_proof(consumer: &mut Alice, consumer_to_institution: &Connection, request_name: Option<&str>, requested_values: Option<&str>) {
        prover_select_credentials_and_send_proof_and_assert(consumer, consumer_to_institution, request_name, requested_values, ProverState::PresentationPreparationFailed).await
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_proof_should_be_validated() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, cred_def_id, _rev_reg_id, _cred_def, _credential_handle) = issue_address_credential(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer).await;
        institution.activate().await.unwrap();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let requested_attrs_string = serde_json::to_string(&json!([
           {
               "name": "address1",
               "restrictions": [{
                 "issuer_did": institution_did,
                 "schema_id": schema_id,
                 "cred_def_id": cred_def_id,
               }]
           }])).unwrap();


        info!("test_proof_should_be_validated :: Going to seng proof request with attributes {}", &requested_attrs_string);
        let mut verifier = send_proof_request(&mut institution, &institution_to_consumer, &requested_attrs_string, "[]", "{}", None).await;

        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, None, None).await;

        info!("test_proof_should_be_validated :: verifier :: going to verify proof");
        institution.activate().await.unwrap();
        verifier.update_state(&institution_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_proof_with_predicates_should_be_validated() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        issue_address_credential(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer).await;
        institution.activate().await.unwrap();
        let requested_preds_string = serde_json::to_string(&json!([
           {
               "name": "zip",
               "p_type": ">=",
               "p_value": 83000
           }])).unwrap();

        info!("test_basic_proof :: Going to seng proof request with attributes {}", &requested_preds_string);
        let mut verifier = send_proof_request(&mut institution, &institution_to_consumer, "[]", &requested_preds_string, "{}", None).await;

        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, None, None).await;

        info!("test_basic_revocation :: verifier :: going to verify proof");
        institution.activate().await.unwrap();
        verifier.update_state(&institution_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
        info!("verifier received presentation!: {}", verifier.get_presentation_attachment().unwrap());
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_it_should_fail_to_select_credentials_for_predicate() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        issue_address_credential(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer).await;
        institution.activate().await.unwrap();
        let requested_preds_string = serde_json::to_string(&json!([
           {
               "name": "zip",
               "p_type": ">=",
               "p_value": 85000
           }])).unwrap();

        info!("test_basic_proof :: Going to seng proof request with attributes {}", &requested_preds_string);
        send_proof_request(&mut institution, &institution_to_consumer, "[]", &requested_preds_string, "{}", None).await;

        prover_select_credentials_and_fail_to_generate_proof(&mut consumer, &consumer_to_institution, None, None).await;
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_basic_revocation() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, cred_def_id, rev_reg_id, _cred_def, credential_handle) = issue_address_credential(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer).await;

        let time_before_revocation = time::get_time().sec as u64;
        info!("test_basic_revocation :: verifier :: Going to revoke credential");
        revoke_credential(&mut institution, &credential_handle, rev_reg_id).await;
        thread::sleep(Duration::from_millis(2000));
        let time_after_revocation = time::get_time().sec as u64;

        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let _requested_attrs = requested_attrs(&institution_did, &schema_id, &cred_def_id, None, Some(time_after_revocation));
        let interval = json!({"from": time_before_revocation - 100, "to": time_after_revocation}).to_string();
        let requested_attrs_string = serde_json::to_string(&_requested_attrs).unwrap();

        info!("test_basic_revocation :: Going to seng proof request with attributes {}", &requested_attrs_string);
        let mut verifier = send_proof_request(&mut institution, &institution_to_consumer, &requested_attrs_string, "[]", &interval, None).await;

        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, None, None).await;

        info!("test_basic_revocation :: verifier :: going to verify proof");
        institution.activate().await.unwrap();
        verifier.update_state(&institution_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofInvalid as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_local_revocation() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, cred_def_id, rev_reg_id, _cred_def, issuer_credential) = issue_address_credential(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer).await;

        revoke_credential_local(&mut institution, &issuer_credential, rev_reg_id.clone()).await;
        let request_name1 = Some("request1");
        let mut verifier = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer, &schema_id, &cred_def_id, request_name1).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name1, None).await;

        institution.activate().await.unwrap();
        verifier.update_state(&institution_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        publish_revocation(&mut institution, rev_reg_id.clone().unwrap()).await;
        let request_name2 = Some("request2");
        let mut verifier = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer, &schema_id, &cred_def_id, request_name2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name2, None).await;

        institution.activate().await.unwrap();
        verifier.update_state(&institution_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofInvalid as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_double_issuance_separate_issuer_and_consumers() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer1 = Alice::setup().await;
        let mut consumer2 = Alice::setup().await;
        let (consumer1_to_verifier, verifier_to_consumer1) = create_connected_connections(&mut consumer1, &mut verifier).await;
        let (consumer1_to_issuer, issuer_to_consumer1) = create_connected_connections(&mut consumer1, &mut issuer).await;
        let (consumer2_to_verifier, verifier_to_consumer2) = create_connected_connections(&mut consumer2, &mut verifier).await;
        let (consumer2_to_issuer, issuer_to_consumer2) = create_connected_connections(&mut consumer2, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, _rev_reg_id) = _create_address_schema().await;
        let (address1, address2, city, state, zip) = attr_names();
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let _credential_handle1 = _exchange_credential(&mut consumer1, &mut issuer, credential_data1, &cred_def, &consumer1_to_issuer, &issuer_to_consumer1, None).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = _exchange_credential(&mut consumer2, &mut issuer, credential_data2, &cred_def, &consumer2_to_issuer, &issuer_to_consumer2, None).await;

        let request_name1 = Some("request1");
        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer1, &schema_id, &cred_def_id, request_name1).await;
        prover_select_credentials_and_send_proof(&mut consumer1, &consumer1_to_verifier, None, None).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(&verifier_to_consumer1).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        let request_name2 = Some("request2");
        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer2, &schema_id, &cred_def_id, request_name2).await;
        prover_select_credentials_and_send_proof(&mut consumer2, &consumer2_to_verifier, None, None).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(&verifier_to_consumer2).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_double_issuance_separate_issuer() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, cred_def_id, _rev_reg_id, _cred_def, _credential_handle) = issue_address_credential(&mut consumer, &mut issuer, &consumer_to_issuer, &issuer_to_consumer).await;
        issuer.activate().await.unwrap();
        let request_name1 = Some("request1");
        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, request_name1).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, request_name1, None).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(&verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        let request_name2 = Some("request2");
        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, request_name2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, request_name2, None).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(&verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_double_issuance_issuer_is_verifier() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, _rev_reg_id) = _create_address_schema().await;
        let (address1, address, city, state, zip) = attr_names();
        let credential_data = json!({address1.clone(): "5th Avenue", address.clone(): "Suite 1234", city.clone(): "NYC", state.clone(): "NYS", zip.clone(): "84712"}).to_string();
        let _credential_handle = _exchange_credential(&mut consumer, &mut institution, credential_data, &cred_def, &consumer_to_institution, &institution_to_consumer, None).await;

        let request_name1 = Some("request1");
        let mut verifier = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer, &schema_id, &cred_def_id, request_name1).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name1, None).await;
        institution.activate().await.unwrap();
        verifier.update_state(&institution_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        let request_name2 = Some("request2");
        let mut verifier = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer, &schema_id, &cred_def_id, request_name2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_institution, request_name2, None).await;
        institution.activate().await.unwrap();
        verifier.update_state(&institution_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_batch_revocation() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer1 = Alice::setup().await;
        let mut consumer2 = Alice::setup().await;
        let mut consumer3 = Alice::setup().await;
        let (consumer_to_institution1, institution_to_consumer1) = create_connected_connections(&mut consumer1, &mut institution).await;
        let (consumer_to_institution2, institution_to_consumer2) = create_connected_connections(&mut consumer2, &mut institution).await;
        let (consumer_to_institution3, institution_to_consumer3) = create_connected_connections(&mut consumer3, &mut institution).await;
        // assert_ne!(institution_to_consumer1, institution_to_consumer2);
        // assert_ne!(institution_to_consumer1, institution_to_consumer3);
        // assert_ne!(institution_to_consumer2, institution_to_consumer3);
        // assert_ne!(consumer_to_institution1, consumer_to_institution2);
        // assert_ne!(consumer_to_institution1, consumer_to_institution3);
        // assert_ne!(consumer_to_institution2, consumer_to_institution3);

        // Issue and send three credentials of the same schema
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg_id) = _create_address_schema().await;
        let (address1, address2, city, state, zip) = attr_names();
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let credential_handle1 = _exchange_credential(&mut consumer1, &mut institution, credential_data1, &cred_def, &consumer_to_institution1, &institution_to_consumer1, None).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let credential_handle2 = _exchange_credential(&mut consumer2, &mut institution, credential_data2, &cred_def, &consumer_to_institution2, &institution_to_consumer2, None).await;
        let credential_data3 = json!({address1.clone(): "5th Avenue", address2.clone(): "Suite 1234", city.clone(): "NYC", state.clone(): "NYS", zip.clone(): "84712"}).to_string();
        let _credential_handle3 = _exchange_credential(&mut consumer3, &mut institution, credential_data3, &cred_def, &consumer_to_institution3, &institution_to_consumer3, None).await;

        revoke_credential_local(&mut institution, &credential_handle1, rev_reg_id.clone()).await;
        revoke_credential_local(&mut institution, &credential_handle2, rev_reg_id.clone()).await;

        // Revoke two locally and verify their are all still valid
        let request_name1 = Some("request1");
        let mut verifier1 = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer1, &schema_id, &cred_def_id, request_name1).await;
        prover_select_credentials_and_send_proof(&mut consumer1, &consumer_to_institution1, request_name1, None).await;
        let mut verifier2 = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer2, &schema_id, &cred_def_id, request_name1).await;
        prover_select_credentials_and_send_proof(&mut consumer2, &consumer_to_institution2, request_name1, None).await;
        let mut verifier3 = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer3, &schema_id, &cred_def_id, request_name1).await;
        prover_select_credentials_and_send_proof(&mut consumer3, &consumer_to_institution3, request_name1, None).await;

        institution.activate().await.unwrap();
        verifier1.update_state(&institution_to_consumer1).await.unwrap();
        verifier2.update_state(&institution_to_consumer2).await.unwrap();
        verifier3.update_state(&institution_to_consumer3).await.unwrap();
        assert_eq!(verifier1.get_presentation_status(), ProofStateType::ProofValidated as u32);
        assert_eq!(verifier2.get_presentation_status(), ProofStateType::ProofValidated as u32);
        assert_eq!(verifier3.get_presentation_status(), ProofStateType::ProofValidated as u32);

        // Publish revocations and verify the two are invalid, third still valid
        publish_revocation(&mut institution, rev_reg_id.clone().unwrap()).await;
        thread::sleep(Duration::from_millis(2000));
        let request_name2 = Some("request2");
        let mut verifier1 = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer1, &schema_id, &cred_def_id, request_name2).await;
        prover_select_credentials_and_send_proof(&mut consumer1, &consumer_to_institution1, request_name2, None).await;
        let mut verifier2 = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer2, &schema_id, &cred_def_id, request_name2).await;
        prover_select_credentials_and_send_proof(&mut consumer2, &consumer_to_institution2, request_name2, None).await;
        let mut verifier3 = verifier_create_proof_and_send_request(&mut institution, &institution_to_consumer3, &schema_id, &cred_def_id, request_name2).await;
        prover_select_credentials_and_send_proof(&mut consumer3, &consumer_to_institution3, request_name2, None).await;
        assert_ne!(verifier1, verifier2);
        assert_ne!(verifier1, verifier3);
        assert_ne!(verifier2, verifier3);

        institution.activate().await.unwrap();
        verifier1.update_state(&institution_to_consumer1).await.unwrap();
        verifier2.update_state(&institution_to_consumer2).await.unwrap();
        verifier3.update_state(&institution_to_consumer3).await.unwrap();
        assert_eq!(verifier1.get_presentation_status(), ProofStateType::ProofInvalid as u32);
        assert_eq!(verifier2.get_presentation_status(), ProofStateType::ProofInvalid as u32);
        assert_eq!(verifier3.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_revoked_credential_might_still_work() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, cred_def_id, rev_reg_id, _cred_def, credential_handle) = issue_address_credential(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer).await;

        thread::sleep(Duration::from_millis(1000));
        let time_before_revocation = time::get_time().sec as u64;
        thread::sleep(Duration::from_millis(2000));
        info!("test_revoked_credential_might_still_work :: verifier :: Going to revoke credential");
        revoke_credential(&mut institution, &credential_handle, rev_reg_id).await;
        thread::sleep(Duration::from_millis(2000));

        let from = time_before_revocation - 100;
        let to = time_before_revocation;
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let _requested_attrs = requested_attrs(&institution_did, &schema_id, &cred_def_id, Some(from), Some(to));
        let interval = json!({"from": from, "to": to}).to_string();
        let requested_attrs_string = serde_json::to_string(&_requested_attrs).unwrap();

        info!("test_revoked_credential_might_still_work :: Going to seng proof request with attributes {}", &requested_attrs_string);
        let mut verifier = send_proof_request(&mut institution, &institution_to_consumer, &requested_attrs_string, "[]", &interval, None).await;

        info!("test_revoked_credential_might_still_work :: Going to create proof");
        let mut prover = create_proof(&mut consumer, &consumer_to_institution, None).await;
        info!("test_revoked_credential_might_still_work :: retrieving matching credentials");

        let retrieved_credentials = prover.retrieve_credentials().await.unwrap();
        info!("test_revoked_credential_might_still_work :: prover :: based on proof, retrieved credentials: {}", &retrieved_credentials);

        let selected_credentials_value = retrieved_to_selected_credentials_simple(&retrieved_credentials, true);
        let selected_credentials_str = serde_json::to_string(&selected_credentials_value).unwrap();
        info!("test_revoked_credential_might_still_work :: prover :: retrieved credential converted to selected: {}", &selected_credentials_str);
        generate_and_send_proof(&mut consumer, &mut prover, &consumer_to_institution, &selected_credentials_str).await;
        assert_eq!(ProverState::PresentationSent, prover.get_state());

        info!("test_revoked_credential_might_still_work :: verifier :: going to verify proof");
        institution.activate().await.unwrap();
        verifier.update_state(&institution_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    fn retrieved_to_selected_credentials_simple(retrieved_credentials: &str, with_tails: bool) -> Value {
        info!("test_real_proof >>> retrieved matching credentials {}", retrieved_credentials);
        let data: Value = serde_json::from_str(retrieved_credentials).unwrap();
        let mut credentials_mapped: Value = json!({"attrs":{}});

        for (key, val) in data["attrs"].as_object().unwrap().iter() {
            let cred_array = val.as_array().unwrap();
            if cred_array.len() > 0 {
                let first_cred = &cred_array[0];
                credentials_mapped["attrs"][key]["credential"] = first_cred.clone();
                if with_tails {
                    credentials_mapped["attrs"][key]["tails_file"] = Value::from(get_temp_dir_path(TAILS_DIR).to_str().unwrap());
                }
            }
        }
        return credentials_mapped;
    }

    fn retrieved_to_selected_credentials_specific(retrieved_credentials: &str, requested_values: &str, credential_data: &str, with_tails: bool) -> Value {
        info!("test_real_proof >>> retrieved matching credentials {}", retrieved_credentials);
        let retrieved_credentials: Value = serde_json::from_str(retrieved_credentials).unwrap();
        let credential_data: Value = serde_json::from_str(credential_data).unwrap();
        let requested_values: Value = serde_json::from_str(requested_values).unwrap();
        let requested_attributes: &Value = &credential_data["requested_attributes"];
        let mut credentials_mapped: Value = json!({"attrs":{}});

        for (key, val) in retrieved_credentials["attrs"].as_object().unwrap().iter() {
            let filtered: Vec<&Value> = val.as_array().unwrap()
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
                credentials_mapped["attrs"][key]["tails_file"] = Value::from(get_temp_dir_path(TAILS_DIR).to_str().unwrap());
            }
        }
        return credentials_mapped;
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_real_proof() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;

        info!("test_real_proof >>>");
        let number_of_attributes = 10;

        info!("test_real_proof :: AS INSTITUTION SEND CREDENTIAL OFFER");
        let mut attrs_list: Value = serde_json::Value::Array(vec![]);
        for i in 1..number_of_attributes {
            attrs_list.as_array_mut().unwrap().push(json!(format!("key{}",i)));
        }
        let attrs_list = attrs_list.to_string();
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, _) = create_and_store_credential_def(&attrs_list, false).await;
        let mut credential_data = json!({});
        for i in 1..number_of_attributes {
            credential_data[format!("key{}", i)] = Value::String(format!("value{}", i));
        }
        info!("test_real_proof :: sending credential offer");
        let credential_data = credential_data.to_string();
        info!("test_real_proof :: generated credential data: {}", credential_data);
        let mut issuer_credential = create_and_send_cred_offer(&mut institution, &cred_def, &issuer_to_consumer, &credential_data, None).await;
        let issuance_thread_id = issuer_credential.get_thread_id().unwrap();

        info!("test_real_proof :: AS CONSUMER SEND CREDENTIAL REQUEST");
        let mut holder_credential = send_cred_req(&mut consumer, &consumer_to_issuer, None).await;

        info!("test_real_proof :: AS INSTITUTION SEND CREDENTIAL");
        send_credential(&mut consumer, &mut institution, &mut issuer_credential, &issuer_to_consumer, &consumer_to_issuer, &mut holder_credential, false).await;
        assert_eq!(issuance_thread_id, holder_credential.get_thread_id().unwrap());
        assert_eq!(issuance_thread_id, issuer_credential.get_thread_id().unwrap());

        info!("test_real_proof :: AS INSTITUTION SEND PROOF REQUEST");
        institution.activate().await.unwrap();

        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let restrictions = json!({ "issuer_did": institution_did, "schema_id": schema_id, "cred_def_id": cred_def_id, });
        let mut attrs: Value = serde_json::Value::Array(vec![]);
        for i in 1..number_of_attributes {
            attrs.as_array_mut().unwrap().push(json!({ "name":format!("key{}", i), "restrictions": [restrictions]}));
        }
        let requested_attrs = attrs.to_string();
        info!("test_real_proof :: Going to seng proof request with attributes {}", requested_attrs);
        let mut verifier = send_proof_request(&mut institution, &issuer_to_consumer, &requested_attrs, "[]", "{}", None).await;
        let presentation_thread_id = verifier.get_thread_id().unwrap();

        info!("test_real_proof :: Going to create proof");
        let mut prover = create_proof(&mut consumer, &consumer_to_issuer, None).await;
        info!("test_real_proof :: retrieving matching credentials");

        let retrieved_credentials = prover.retrieve_credentials().await.unwrap();
        let selected_credentials = retrieved_to_selected_credentials_simple(&retrieved_credentials, false);

        info!("test_real_proof :: generating and sending proof");
        generate_and_send_proof(&mut consumer, &mut prover, &consumer_to_issuer, &serde_json::to_string(&selected_credentials).unwrap()).await;
        assert_eq!(ProverState::PresentationSent, prover.get_state());
        assert_eq!(presentation_thread_id, prover.get_thread_id().unwrap());
        assert_eq!(presentation_thread_id, verifier.get_thread_id().unwrap());

        info!("test_real_proof :: AS INSTITUTION VALIDATE PROOF");
        institution.activate().await.unwrap();
        verifier.update_state(&issuer_to_consumer).await.unwrap();
        assert_eq!(verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
        assert_eq!(presentation_thread_id, verifier.get_thread_id().unwrap());
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_two_creds_one_rev_reg() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, _rev_reg_id) = _create_address_schema().await;
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let _credential_handle1 = _exchange_credential(&mut consumer, &mut issuer, credential_data1.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req1).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = _exchange_credential(&mut consumer, &mut issuer, credential_data2.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req2).await;

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req1).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(&verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(&verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_two_creds_one_rev_reg_revoke_first() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg_id) = _create_address_schema().await;
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let credential_handle1 = _exchange_credential(&mut consumer, &mut issuer, credential_data1.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req1).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = _exchange_credential(&mut consumer, &mut issuer, credential_data2.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req2).await;

        revoke_credential(&mut issuer, &credential_handle1, rev_reg_id).await;

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req1).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(&verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofInvalid as u32);

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(&verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_two_creds_one_rev_reg_revoke_second() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg_id) = _create_address_schema().await;
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let _credential_handle1 = _exchange_credential(&mut consumer, &mut issuer, credential_data1.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req1).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let credential_handle2 = _exchange_credential(&mut consumer, &mut issuer, credential_data2.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req2).await;

        revoke_credential(&mut issuer, &credential_handle2, rev_reg_id).await;

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req1).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(&verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(&verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofInvalid as u32);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_two_creds_two_rev_reg_id() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, mut cred_def, _rev_reg_id) = _create_address_schema().await;
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let _credential_handle1 = _exchange_credential(&mut consumer, &mut issuer, credential_data1.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req1).await;
        rotate_rev_reg(&mut issuer, &mut cred_def).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = _exchange_credential(&mut consumer, &mut issuer, credential_data2.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req2).await;

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req1).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(&verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(&verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_two_creds_two_rev_reg_id_revoke_first() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, mut cred_def, rev_reg_id) = _create_address_schema().await;
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let credential_handle1 = _exchange_credential(&mut consumer, &mut issuer, credential_data1.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req1).await;
        rotate_rev_reg(&mut issuer, &mut cred_def).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let _credential_handle2 = _exchange_credential(&mut consumer, &mut issuer, credential_data2.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req2).await;

        revoke_credential(&mut issuer, &credential_handle1, rev_reg_id).await;

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req1).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(&verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofInvalid as u32);

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(&verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_two_creds_two_rev_reg_id_revoke_second() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut issuer = Faber::setup().await;
        let mut verifier = Faber::setup().await;
        let mut consumer = Alice::setup().await;
        let (consumer_to_verifier, verifier_to_consumer) = create_connected_connections(&mut consumer, &mut verifier).await;
        let (consumer_to_issuer, issuer_to_consumer) = create_connected_connections(&mut consumer, &mut issuer).await;

        let (schema_id, _schema_json, cred_def_id, _cred_def_json, mut cred_def, rev_reg_id) = _create_address_schema().await;
        let (address1, address2, city, state, zip) = attr_names();
        let (req1, req2) = (Some("request1"), Some("request2"));
        let credential_data1 = json!({address1.clone(): "123 Main St", address2.clone(): "Suite 3", city.clone(): "Draper", state.clone(): "UT", zip.clone(): "84000"}).to_string();
        let _credential_handle1 = _exchange_credential(&mut consumer, &mut issuer, credential_data1.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req1).await;
        rotate_rev_reg(&mut issuer, &mut cred_def).await;
        let credential_data2 = json!({address1.clone(): "101 Tela Lane", address2.clone(): "Suite 1", city.clone(): "SLC", state.clone(): "WA", zip.clone(): "8721"}).to_string();
        let credential_handle2 = _exchange_credential(&mut consumer, &mut issuer, credential_data2.clone(), &cred_def, &consumer_to_issuer, &issuer_to_consumer, req2).await;

        revoke_credential(&mut issuer, &credential_handle2, rev_reg_id).await;

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req1).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req1, Some(&credential_data1)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(&verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofValidated as u32);

        let mut proof_verifier = verifier_create_proof_and_send_request(&mut verifier, &verifier_to_consumer, &schema_id, &cred_def_id, req2).await;
        prover_select_credentials_and_send_proof(&mut consumer, &consumer_to_verifier, req2, Some(&credential_data2)).await;
        verifier.activate().await.unwrap();
        proof_verifier.update_state(&verifier_to_consumer).await.unwrap();
        assert_eq!(proof_verifier.get_presentation_status(), ProofStateType::ProofInvalid as u32);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_establish_connection_via_public_invite() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections_via_public_invite(&mut consumer, &mut institution).await;

        institution_to_consumer.send_generic_message("Hello Alice, Faber here").await.unwrap();

        consumer.activate().await.unwrap();
        let consumer_msgs = consumer_to_institution.download_messages(Some(vec![MessageStatusCode::Received]), None).await.unwrap();
        assert_eq!(consumer_msgs.len(), 1);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_oob_connection_bootstrap() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        institution.activate().await.unwrap();
        let request_sender = create_proof_request(&mut institution, REQUESTED_ATTRIBUTES, "[]", "{}", None).await;

        let service = FullService::try_from(&institution.agent).unwrap();
        let oob_sender = OutOfBandSender::create()
            .set_label("test-label")
            .set_goal_code(&GoalCode::P2PMessaging)
            .set_goal("To exchange message")
            .append_service(&ServiceResolvable::FullService(service))
            .append_handshake_protocol(&HandshakeProtocol::ConnectionV1).unwrap()
            .append_a2a_message(request_sender.to_a2a_message()).unwrap();
        let oob_msg = oob_sender.to_a2a_message();

        consumer.activate().await.unwrap();
        let oob_receiver = OutOfBandReceiver::create_from_a2a_msg(&oob_msg).unwrap();
        let conns = vec![];
        let conn = oob_receiver.connection_exists(&conns).await.unwrap();
        assert!(conn.is_none());
        let mut conn_receiver = oob_receiver.build_connection(true).await.unwrap();
        conn_receiver.connect().await.unwrap();
        conn_receiver.update_state().await.unwrap();
        assert_eq!(ConnectionState::Invitee(InviteeState::Requested), conn_receiver.get_state());
        assert_eq!(oob_sender.oob.id.0, oob_receiver.oob.id.0);

        let conn_sender = connect_using_request_sent_to_public_agent(&mut consumer, &mut institution, &mut conn_receiver).await;

        let (conn_receiver_pw1, _conn_sender_pw1) = create_connected_connections(&mut consumer, &mut institution).await;
        let (conn_receiver_pw2, _conn_sender_pw2) = create_connected_connections(&mut consumer, &mut institution).await;

        let conns = vec![&conn_receiver, &conn_receiver_pw1, &conn_receiver_pw2];
        let conn = oob_receiver.connection_exists(&conns).await.unwrap();
        assert!(conn.is_some());
        assert!(*conn.unwrap() == conn_receiver);

        let conns = vec![&conn_receiver_pw1, &conn_receiver_pw2];
        let conn = oob_receiver.connection_exists(&conns).await.unwrap();
        assert!(conn.is_none());

        let a2a_msg = oob_receiver.extract_a2a_message().unwrap().unwrap();
        assert!(matches!(a2a_msg, A2AMessage::PresentationRequest(..)));
        if let A2AMessage::PresentationRequest(request_receiver) = a2a_msg {
            assert_eq!(request_receiver.request_presentations_attach, request_sender.request_presentations_attach);
        }

        conn_sender.send_generic_message("Hello oob receiver, from oob sender").await.unwrap();
        consumer.activate().await.unwrap();
        conn_receiver.send_generic_message("Hello oob sender, from oob receiver").await.unwrap();
        institution.activate().await.unwrap();
        let sender_msgs = conn_sender.download_messages(None, None).await.unwrap();
        consumer.activate().await.unwrap();
        let receiver_msgs = conn_receiver.download_messages(None, None).await.unwrap();
        assert_eq!(sender_msgs.len(), 2);
        assert_eq!(receiver_msgs.len(), 2);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_oob_connection_reuse() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections_via_public_invite(&mut consumer, &mut institution).await;

        institution.activate().await.unwrap();
        let service = FullService::try_from(&institution.agent).unwrap();
        let oob_sender = OutOfBandSender::create()
            .set_label("test-label")
            .set_goal_code(&GoalCode::P2PMessaging)
            .set_goal("To exchange message")
            .append_service(&ServiceResolvable::FullService(service));
        let oob_msg = oob_sender.to_a2a_message();

        consumer.activate().await.unwrap();
        let oob_receiver = OutOfBandReceiver::create_from_a2a_msg(&oob_msg).unwrap();
        let conns = vec![&consumer_to_institution];
        let conn = oob_receiver.connection_exists(&conns).await.unwrap();
        assert!(conn.is_some());
        conn.unwrap().send_generic_message("Hello oob sender, from oob receiver").await.unwrap();

        institution.activate().await.unwrap();
        let msgs = institution_to_consumer.download_messages(None, None).await.unwrap();
        assert_eq!(msgs.len(), 2);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    async fn test_oob_connection_handshake_reuse() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (mut consumer_to_institution, mut institution_to_consumer) = create_connected_connections_via_public_invite(&mut consumer, &mut institution).await;

        institution.activate().await.unwrap();
        let service = FullService::try_from(&institution.agent).unwrap();
        let oob_sender = OutOfBandSender::create()
            .set_label("test-label")
            .set_goal_code(&GoalCode::P2PMessaging)
            .set_goal("To exchange message")
            .append_service(&ServiceResolvable::FullService(service));
        let sender_oob_id = oob_sender.get_id();
        let oob_msg = oob_sender.to_a2a_message();

        consumer.activate().await.unwrap();
        let oob_receiver = OutOfBandReceiver::create_from_a2a_msg(&oob_msg).unwrap();
        let conns = vec![&consumer_to_institution];
        let conn = oob_receiver.connection_exists(&conns).await.unwrap();
        assert!(conn.is_some());
        let receiver_oob_id = oob_receiver.get_id();
        let receiver_msg = serde_json::to_string(&oob_receiver.to_a2a_message()).unwrap();
        conn.unwrap().send_handshake_reuse(&receiver_msg).await.unwrap();

        institution.activate().await.unwrap();
        let mut msgs = institution_to_consumer.download_messages(Some(vec![MessageStatusCode::Received]), None).await.unwrap();
        assert_eq!(msgs.len(), 1);
        let reuse_msg = match serde_json::from_str::<A2AMessage>(&msgs.pop().unwrap().decrypted_msg.unwrap()).unwrap() {
            A2AMessage::OutOfBandHandshakeReuse(ref a2a_msg) => {
                assert_eq!(sender_oob_id, a2a_msg.thread.pthid.as_ref().unwrap().to_string());
                assert_eq!(receiver_oob_id, a2a_msg.thread.pthid.as_ref().unwrap().to_string());
                assert_eq!(a2a_msg.id.0, a2a_msg.thread.thid.as_ref().unwrap().to_string());
                a2a_msg.clone()
            }
            _ => { panic!("Expected OutOfBandHandshakeReuse message type"); }
        };
        institution_to_consumer.update_state_with_message(&A2AMessage::OutOfBandHandshakeReuse(reuse_msg.clone())).await.unwrap();

        consumer.activate().await.unwrap();
        let mut msgs = consumer_to_institution.download_messages(Some(vec![MessageStatusCode::Received]), None).await.unwrap();
        assert_eq!(msgs.len(), 1);
        let reuse_ack_msg = match serde_json::from_str::<A2AMessage>(&msgs.pop().unwrap().decrypted_msg.unwrap()).unwrap() {
            A2AMessage::OutOfBandHandshakeReuseAccepted(ref a2a_msg) => {
                assert_eq!(sender_oob_id, a2a_msg.thread.pthid.as_ref().unwrap().to_string());
                assert_eq!(receiver_oob_id, a2a_msg.thread.pthid.as_ref().unwrap().to_string());
                assert_eq!(reuse_msg.id.0, a2a_msg.thread.thid.as_ref().unwrap().to_string());
                a2a_msg.clone()
            }
            _ => { panic!("Expected OutOfBandHandshakeReuseAccepted message type"); }
        };
        consumer_to_institution.update_state_with_message(&A2AMessage::OutOfBandHandshakeReuseAccepted(reuse_ack_msg)).await.unwrap();
        consumer_to_institution.update_state().await.unwrap();
        assert_eq!(consumer_to_institution.download_messages(Some(vec![MessageStatusCode::Received]), None).await.unwrap().len(), 0);
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    pub async fn test_two_enterprise_connections() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer1 = Alice::setup().await;

        let (_faber, _alice) = create_connected_connections(&mut consumer1, &mut institution).await;
        let (_faber, _alice) = create_connected_connections(&mut consumer1, &mut institution).await;
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    pub async fn test_credential_exchange_via_proposal() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg_id) = _create_address_schema().await;
        let tails_file = cred_def.get_tails_dir().unwrap();

        _exchange_credential_with_proposal(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer, &schema_id, &cred_def_id, rev_reg_id, Some(tails_file), "comment").await;
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    pub async fn test_credential_exchange_via_proposal_failed() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg_id) = _create_address_schema().await;
        let tails_file = cred_def.get_tails_dir().unwrap();

        let mut holder = send_cred_proposal(&mut consumer, &consumer_to_institution, &schema_id, &cred_def_id, "comment").await;
        let mut issuer = accept_cred_proposal(&mut institution, &institution_to_consumer, rev_reg_id, Some(tails_file)).await;
        decline_offer(&mut consumer, &consumer_to_institution, &mut holder).await;
        institution.activate().await.unwrap();
        assert_eq!(IssuerState::OfferSent, issuer.get_state());
        issuer.update_state(&institution_to_consumer).await.unwrap();
        assert_eq!(IssuerState::Failed, issuer.get_state());
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    pub async fn test_credential_exchange_via_proposal_with_negotiation() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg_id) = _create_address_schema().await;
        let tails_file = cred_def.get_tails_dir().unwrap();

        let mut holder = send_cred_proposal(&mut consumer, &consumer_to_institution, &schema_id, &cred_def_id, "comment").await;
        let mut issuer = accept_cred_proposal(&mut institution, &institution_to_consumer, rev_reg_id.clone(), Some(tails_file.clone())).await;
        send_cred_proposal_1(&mut holder, &mut consumer, &consumer_to_institution, &schema_id, &cred_def_id, "comment").await;
        accept_cred_proposal_1(&mut issuer, &mut institution, &institution_to_consumer, rev_reg_id, Some(tails_file)).await;
        accept_offer(&mut consumer, &consumer_to_institution, &mut holder).await;
        send_credential(&mut consumer, &mut institution, &mut issuer, &institution_to_consumer, &consumer_to_institution, &mut holder, true).await;
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    pub async fn test_presentation_via_proposal() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg_id) = _create_address_schema().await;
        let tails_file = cred_def.get_tails_dir().unwrap();

        _exchange_credential_with_proposal(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer, &schema_id, &cred_def_id, rev_reg_id, Some(tails_file), "comment").await;
        let mut prover = send_proof_proposal(&mut consumer, &consumer_to_institution, &cred_def_id).await;
        let mut verifier = Verifier::create("1").unwrap();
        accept_proof_proposal(&mut institution, &mut verifier, &institution_to_consumer).await;
        let selected_credentials_str = prover_select_credentials(&mut prover, &mut consumer, &consumer_to_institution, None).await;
        generate_and_send_proof(&mut consumer, &mut prover, &consumer_to_institution, &selected_credentials_str).await;
        verify_proof(&mut institution, &mut verifier, &institution_to_consumer).await;
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    pub async fn test_presentation_via_proposal_with_rejection() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg_id) = _create_address_schema().await;
        let tails_file = cred_def.get_tails_dir().unwrap();

        _exchange_credential_with_proposal(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer, &schema_id, &cred_def_id, rev_reg_id, Some(tails_file), "comment").await;
        let mut prover = send_proof_proposal(&mut consumer, &consumer_to_institution, &cred_def_id).await;
        reject_proof_proposal(&mut institution, &institution_to_consumer).await;
        receive_proof_proposal_rejection(&mut consumer, &mut prover, &consumer_to_institution).await;
    }

    #[tokio::test]
    #[cfg(feature = "agency_pool_tests")]
    pub async fn test_presentation_via_proposal_with_negotiation() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (consumer_to_institution, institution_to_consumer) = create_connected_connections(&mut consumer, &mut institution).await;
        let (schema_id, _schema_json, cred_def_id, _cred_def_json, cred_def, rev_reg_id) = _create_address_schema().await;
        let tails_file = cred_def.get_tails_dir().unwrap();

        _exchange_credential_with_proposal(&mut consumer, &mut institution, &consumer_to_institution, &institution_to_consumer, &schema_id, &cred_def_id, rev_reg_id, Some(tails_file), "comment").await;
        let mut prover = send_proof_proposal(&mut consumer, &consumer_to_institution, &cred_def_id).await;
        let mut verifier = Verifier::create("1").unwrap();
        accept_proof_proposal(&mut institution, &mut verifier, &institution_to_consumer).await;
        send_proof_proposal_1(&mut consumer, &mut prover, &consumer_to_institution, &cred_def_id).await;
        accept_proof_proposal(&mut institution, &mut verifier, &institution_to_consumer).await;
        let selected_credentials_str = prover_select_credentials(&mut prover, &mut consumer, &consumer_to_institution, None).await;
        generate_and_send_proof(&mut consumer, &mut prover, &consumer_to_institution, &selected_credentials_str).await;
        verify_proof(&mut institution, &mut verifier, &institution_to_consumer).await;
    }

    pub struct Pool {}

    impl Pool {
        pub fn open() -> Pool {
            futures::executor::block_on(libindy::utils::pool::test_utils::open_test_pool());
            Pool {}
        }
    }

    impl Drop for Pool {
        fn drop(&mut self) {
            futures::executor::block_on(libindy::utils::pool::close()).unwrap();
            futures::executor::block_on(libindy::utils::pool::test_utils::delete_test_pool());
        }
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn aries_demo() {
        let _setup = SetupEmpty::init();
        let _pool = Pool::open();

        let mut faber = Faber::setup().await;
        let mut alice = Alice::setup().await;

        // Publish Schema and Credential Definition
        faber.create_schema().await;

        std::thread::sleep(std::time::Duration::from_secs(2));

        faber.create_credential_definition().await;

        // Connection
        let invite = faber.create_invite().await;
        alice.accept_invite(&invite).await;

        faber.update_state(3).await;
        alice.update_state(4).await;
        faber.update_state(4).await;

        // Credential issuance
        faber.offer_credential().await;
        alice.accept_offer().await;
        faber.send_credential().await;
        alice.accept_credential().await;

        // Credential Presentation
        faber.request_presentation().await;
        alice.send_presentation().await;
        faber.verify_presentation().await;
        alice.ensure_presentation_verified().await;
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn aries_demo_handle_connection_related_messages() {
        let _setup = SetupEmpty::init();
        let _pool = Pool::open();

        let mut faber = Faber::setup().await;
        let mut alice = Alice::setup().await;

        // Publish Schema and Credential Definition
        faber.create_schema().await;

        std::thread::sleep(std::time::Duration::from_secs(2));

        faber.create_credential_definition().await;

        // Connection
        let invite = faber.create_invite().await;
        alice.accept_invite(&invite).await;

        faber.update_state(3).await;
        alice.update_state(4).await;
        faber.update_state(4).await;

        // Ping
        faber.ping().await;

        alice.update_state(4).await;

        faber.update_state(4).await;

        let faber_connection_info = faber.connection_info().await;
        assert!(faber_connection_info["their"]["protocols"].as_array().is_none());

        // Discovery Features
        faber.discovery_features().await;

        alice.update_state(4).await;

        faber.update_state(4).await;

        let faber_connection_info = faber.connection_info().await;
        assert!(faber_connection_info["their"]["protocols"].as_array().unwrap().len() > 0);
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn aries_demo_create_with_message_id_flow() {
        let _setup = SetupEmpty::init();
        let _pool = Pool::open();

        let mut faber = Faber::setup().await;
        let mut alice = Alice::setup().await;

        // Publish Schema and Credential Definition
        faber.create_schema().await;

        std::thread::sleep(std::time::Duration::from_secs(2));

        faber.create_credential_definition().await;

        // Connection
        let invite = faber.create_invite().await;
        alice.accept_invite(&invite).await;

        faber.update_state(3).await;
        alice.update_state(4).await;
        faber.update_state(4).await;

        /*
         Create with message id flow
        */

        // Credential issuance
        faber.offer_credential().await;

        // Alice creates Credential object with message id
        {
            let message = alice.download_message(PayloadKinds::CredOffer).await.unwrap();
            let cred_offer = alice.get_credential_offer_by_msg_id(&message.uid).await.unwrap();
            alice.credential = Holder::create_from_offer("test", cred_offer).unwrap();

            let pw_did = alice.connection.pairwise_info().pw_did.to_string();
            alice.credential.send_request(pw_did, alice.connection.send_message_closure().unwrap()).await.unwrap();
            assert_eq!(HolderState::RequestSent, alice.credential.get_state());
        }

        faber.send_credential().await;
        alice.accept_credential().await;

        // Credential Presentation
        faber.request_presentation().await;

        // Alice creates Presentation object with message id
        {
            let message = alice.download_message(PayloadKinds::ProofRequest).await.unwrap();
            let presentation_request = alice.get_proof_request_by_msg_id(&message.uid).await.unwrap();
            alice.prover = Prover::create_from_request("test", presentation_request).unwrap();

            let credentials = alice.get_credentials_for_presentation().await;

            alice.prover.generate_presentation(credentials.to_string(), String::from("{}")).await.unwrap();
            assert_eq!(ProverState::PresentationPrepared, alice.prover.get_state());

            alice.prover.send_presentation(alice.connection.send_message_closure().unwrap()).await.unwrap();
            assert_eq!(ProverState::PresentationSent, alice.prover.get_state());
        }

        faber.verify_presentation().await;
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn aries_demo_download_message_flow() {
        SetupEmpty::init();
        let _pool = Pool::open();

        let mut faber = Faber::setup().await;
        let mut alice = Alice::setup().await;

        // Publish Schema and Credential Definition
        faber.create_schema().await;

        std::thread::sleep(std::time::Duration::from_secs(2));

        faber.create_credential_definition().await;

        // Connection
        let invite = faber.create_invite().await;
        alice.accept_invite(&invite).await;

        faber.update_state(3).await;
        alice.update_state(4).await;
        faber.update_state(4).await;

        /*
         Create with message flow
        */

        // Credential issuance
        faber.offer_credential().await;

        // Alice creates Credential object with Offer
        {
            let message = alice.download_message(PayloadKinds::CredOffer).await.unwrap();

            let cred_offer: CredentialOffer = serde_json::from_str(&message.decrypted_msg).unwrap();
            alice.credential = Holder::create_from_offer("test", cred_offer).unwrap();

            alice.connection.update_message_status(&message.uid).await.unwrap();

            let pw_did = alice.connection.pairwise_info().pw_did.to_string();
            alice.credential.send_request(pw_did, alice.connection.send_message_closure().unwrap()).await.unwrap();
            assert_eq!(HolderState::RequestSent, alice.credential.get_state());
        }

        faber.send_credential().await;
        alice.accept_credential().await;

        // Credential Presentation
        faber.request_presentation().await;

        // Alice creates Presentation object with Proof Request
        {
            let agency_msg = alice.download_message(PayloadKinds::ProofRequest).await.unwrap();

            let presentation_request: PresentationRequest = serde_json::from_str(&agency_msg.decrypted_msg).unwrap();
            alice.prover = Prover::create_from_request("test", presentation_request).unwrap();

            alice.connection.update_message_status(&agency_msg.uid).await.unwrap();

            let credentials = alice.get_credentials_for_presentation().await;

            alice.prover.generate_presentation(credentials.to_string(), String::from("{}")).await.unwrap();
            assert_eq!(ProverState::PresentationPrepared, alice.prover.get_state());

            alice.prover.send_presentation(alice.connection.send_message_closure().unwrap()).await.unwrap();
            assert_eq!(ProverState::PresentationSent, alice.prover.get_state());
        }

        faber.verify_presentation().await;
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_deserialize_connection_inviter_completed() {
        let _setup = SetupMocks::init();

        let connection = Connection::from_string(CONNECTION_SM_INVITER_COMPLETED).unwrap();
        let _second_string = connection.to_string();

        assert_eq!(connection.pairwise_info().pw_did, "2ZHFFhzA2XtTD6hJqzL7ux");
        assert_eq!(connection.pairwise_info().pw_vk, "rCw3x5h1jS6gPo7rRrt3EYbXXe5nNjnGbdf1jAwUxuj");
        assert_eq!(connection.cloud_agent_info().agent_did, "EZrZyu4bfydm4ByNm56kPP");
        assert_eq!(connection.cloud_agent_info().agent_vk, "8Ps2WosJ9AV1eXPoJKsEJdM3NchPhSyS8qFt6LQUTKv2");
        assert_eq!(connection.get_state(), ConnectionState::Inviter(InviterState::Completed));
    }

    fn test_deserialize_and_serialize(sm_serialized: &str) {
        let original_object: Value = serde_json::from_str(sm_serialized).unwrap();
        let connection = Connection::from_string(sm_serialized).unwrap();
        let reserialized = connection.to_string().unwrap();
        let reserialized_object: Value = serde_json::from_str(&reserialized).unwrap();

        assert_eq!(original_object, reserialized_object);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_deserialize_and_serialize_should_produce_the_same_object() {
        let _setup = SetupMocks::init();

        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_INVITED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_REQUESTED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITEE_COMPLETED);
        test_deserialize_and_serialize(CONNECTION_SM_INVITER_COMPLETED);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_serialize_deserialize() {
        let _setup = SetupMocks::init();

        let connection = Connection::create("test_serialize_deserialize", true).await.unwrap();
        let first_string = connection.to_string().unwrap();

        let connection2 = Connection::from_string(&first_string).unwrap();
        let second_string = connection2.to_string().unwrap();

        assert_eq!(first_string, second_string);
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_serialize_deserialize_serde() {
        let _setup = SetupMocks::init();

        let connection = Connection::create("test_serialize_deserialize", true).await.unwrap();
        let first_string = serde_json::to_string(&connection).unwrap();

        let connection: Connection = serde_json::from_str(&first_string).unwrap();
        let second_string = serde_json::to_string(&connection).unwrap();
        assert_eq!(first_string, second_string);
    }

    pub async fn create_connected_connections(consumer: &mut Alice, institution: &mut Faber) -> (Connection, Connection) {
        debug!("Institution is going to create connection.");
        institution.activate().await.unwrap();
        let mut institution_to_consumer = Connection::create("consumer", true).await.unwrap();
        institution_to_consumer.connect().await.unwrap();
        let details = institution_to_consumer.get_invite_details().unwrap();

        consumer.activate().await.unwrap();
        debug!("Consumer is going to accept connection invitation.");
        let mut consumer_to_institution = Connection::create_with_invite("institution", details.clone(), true).await.unwrap();

        consumer_to_institution.connect().await.unwrap();
        consumer_to_institution.update_state().await.unwrap();

        let thread_id = consumer_to_institution.get_thread_id();

        debug!("Institution is going to process connection request.");
        institution.activate().await.unwrap();
        thread::sleep(Duration::from_millis(500));
        institution_to_consumer.update_state().await.unwrap();
        assert_eq!(ConnectionState::Inviter(InviterState::Responded), institution_to_consumer.get_state());
        assert_eq!(thread_id, institution_to_consumer.get_thread_id());

        debug!("Consumer is going to complete the connection protocol.");
        consumer.activate().await.unwrap();
        consumer_to_institution.update_state().await.unwrap();
        assert_eq!(ConnectionState::Invitee(InviteeState::Completed), consumer_to_institution.get_state());
        assert_eq!(thread_id, consumer_to_institution.get_thread_id());

        debug!("Institution is going to complete the connection protocol.");
        institution.activate().await.unwrap();
        thread::sleep(Duration::from_millis(500));
        institution_to_consumer.update_state().await.unwrap();
        assert_eq!(ConnectionState::Inviter(InviterState::Completed), institution_to_consumer.get_state());
        assert_eq!(thread_id, consumer_to_institution.get_thread_id());

        (consumer_to_institution, institution_to_consumer)
    }

    pub async fn connect_using_request_sent_to_public_agent(consumer: &mut Alice, institution: &mut Faber, consumer_to_institution: &mut Connection) -> Connection {
        institution.activate().await.unwrap();
        thread::sleep(Duration::from_millis(500));
        let mut conn_requests = institution.agent.download_connection_requests(None).await.unwrap();
        assert_eq!(conn_requests.len(), 1);
        let mut institution_to_consumer = Connection::create_with_request(conn_requests.pop().unwrap(), &institution.agent).await.unwrap();
        assert_eq!(ConnectionState::Inviter(InviterState::Requested), institution_to_consumer.get_state());
        institution_to_consumer.update_state().await.unwrap();
        assert_eq!(ConnectionState::Inviter(InviterState::Responded), institution_to_consumer.get_state());

        consumer.activate().await.unwrap();
        consumer_to_institution.update_state().await.unwrap();
        assert_eq!(ConnectionState::Invitee(InviteeState::Completed), consumer_to_institution.get_state());

        institution.activate().await.unwrap();
        thread::sleep(Duration::from_millis(500));
        institution_to_consumer.update_state().await.unwrap();
        assert_eq!(ConnectionState::Inviter(InviterState::Completed), institution_to_consumer.get_state());

        assert_eq!(institution_to_consumer.get_thread_id(), consumer_to_institution.get_thread_id());

        institution_to_consumer
    }

    pub async fn create_connected_connections_via_public_invite(consumer: &mut Alice, institution: &mut Faber) -> (Connection, Connection) {
        institution.activate().await.unwrap();
        let public_invite_json = institution.create_public_invite().unwrap();
        let public_invite: Invitation = serde_json::from_str(&public_invite_json).unwrap();

        consumer.activate().await.unwrap();
        let mut consumer_to_institution = Connection::create_with_invite("institution", public_invite, true).await.unwrap();
        consumer_to_institution.connect().await.unwrap();
        consumer_to_institution.update_state().await.unwrap();

        let institution_to_consumer = connect_using_request_sent_to_public_agent(consumer, institution, &mut consumer_to_institution).await;
        (consumer_to_institution, institution_to_consumer)
    }

    #[cfg(feature = "agency_pool_tests")]
    #[tokio::test]
    async fn test_send_and_download_messages() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer = Alice::setup().await;

        let (alice_to_faber, faber_to_alice) = create_connected_connections(&mut consumer, &mut institution).await;

        institution.activate().await.unwrap();
        faber_to_alice.send_generic_message("Hello Alice").await.unwrap();
        faber_to_alice.send_generic_message("How are you Alice?").await.unwrap();

        consumer.activate().await.unwrap();
        alice_to_faber.send_generic_message("Hello Faber").await.unwrap();

        let alice_pw_did = alice_to_faber.pairwise_info().pw_did.clone();
        let faber_remote_did = faber_to_alice.remote_did().unwrap();
        assert_eq!(alice_pw_did, faber_remote_did);

        thread::sleep(Duration::from_millis(1000));

        let all_messages = download_messages_noauth(Some(vec![alice_pw_did.clone()]), None, None).await.unwrap();
        assert_eq!(all_messages.len(), 1);
        assert_eq!(all_messages[0].msgs.len(), 3);
        assert!(all_messages[0].msgs[0].decrypted_msg.is_some());
        assert!(all_messages[0].msgs[1].decrypted_msg.is_some());

        let received = download_messages_noauth(Some(vec![alice_pw_did.clone()]), Some(vec![MessageStatusCode::Received.to_string()]), None).await.unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].msgs.len(), 2);
        assert!(received[0].msgs[0].decrypted_msg.is_some());
        assert_eq!(received[0].msgs[0].status_code, MessageStatusCode::Received);
        assert!(received[0].msgs[1].decrypted_msg.is_some());

        // there should be messages in "Reviewed" status connections/1.0/response from Aries-Faber connection protocol
        let reviewed = download_messages_noauth(Some(vec![alice_pw_did.clone()]), Some(vec![MessageStatusCode::Reviewed.to_string()]), None).await.unwrap();
        assert_eq!(reviewed.len(), 1);
        assert_eq!(reviewed[0].msgs.len(), 1);
        assert!(reviewed[0].msgs[0].decrypted_msg.is_some());
        assert_eq!(reviewed[0].msgs[0].status_code, MessageStatusCode::Reviewed);

        let rejected = download_messages_noauth(Some(vec![alice_pw_did.clone()]), Some(vec![MessageStatusCode::Rejected.to_string()]), None).await.unwrap();
        assert_eq!(rejected.len(), 1);
        assert_eq!(rejected[0].msgs.len(), 0);

        let specific = download_messages_noauth(Some(vec![alice_pw_did.clone()]), None, Some(vec![received[0].msgs[0].uid.clone()])).await.unwrap();
        assert_eq!(specific.len(), 1);
        assert_eq!(specific[0].msgs.len(), 1);
        let msg = specific[0].msgs[0].decrypted_msg.clone().unwrap();
        let msg_aries_value: Value = serde_json::from_str(&msg).unwrap();
        assert!(msg_aries_value.is_object());
        assert!(msg_aries_value["@id"].is_string());
        assert!(msg_aries_value["@type"].is_string());
        assert!(msg_aries_value["content"].is_string());

        let unknown_did = "CmrXdgpTXsZqLQtGpX5Yee".to_string();
        let empty = download_messages_noauth(Some(vec![unknown_did]), None, None).await.unwrap();
        assert_eq!(empty.len(), 0);
    }

    #[tokio::test]
    #[cfg(feature = "agency_v2")]
    async fn test_connection_send_works() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut faber = Faber::setup().await;
        let mut alice = Alice::setup().await;

        let invite = faber.create_invite().await;
        alice.accept_invite(&invite).await;

        faber.update_state(3).await;
        alice.update_state(4).await;
        faber.update_state(4).await;

        let uid: String;
        let message = _ack();

        info!("test_connection_send_works:: Test if Send Message works");
        {
            faber.activate().await.unwrap();
            faber.connection.send_message_closure().unwrap()(message.to_a2a_message()).await.unwrap();
        }

        {
            info!("test_connection_send_works:: Test if Get Messages works");
            alice.activate().await.unwrap();

            let messages = alice.connection.get_messages().await.unwrap();
            assert_eq!(1, messages.len());

            uid = messages.keys().next().unwrap().clone();
            let received_message = messages.values().next().unwrap().clone();

            match received_message {
                A2AMessage::Ack(received_message) => assert_eq!(message, received_message.clone()),
                _ => assert!(false)
            }
        }

        info!("test_connection_send_works:: Test if Get Message by id works");
        {
            alice.activate().await.unwrap();

            let message = alice.connection.get_message_by_id(&uid.clone()).await.unwrap();

            match message {
                A2AMessage::Ack(ack) => assert_eq!(_ack(), ack),
                _ => assert!(false)
            }
        }

        info!("test_connection_send_works:: Test if Update Message Status works");
        {
            alice.activate().await.unwrap();

            alice.connection.update_message_status(&uid).await.unwrap();
            let messages = alice.connection.get_messages().await.unwrap();
            assert_eq!(0, messages.len());
        }

        info!("test_connection_send_works:: Test if Send Basic Message works");
        {
            faber.activate().await.unwrap();

            let basic_message = r#"Hi there"#;
            faber.connection.send_generic_message(basic_message).await.unwrap();

            alice.activate().await.unwrap();

            let messages = alice.connection.get_messages().await.unwrap();
            assert_eq!(1, messages.len());

            let uid = messages.keys().next().unwrap().clone();
            let message = messages.values().next().unwrap().clone();

            match message {
                A2AMessage::BasicMessage(message) => assert_eq!(basic_message, message.content),
                _ => assert!(false)
            }
            alice.connection.update_message_status(&uid).await.unwrap();
        }

        info!("test_connection_send_works:: Test if Download Messages");
        {
            use aries_vcx::agency_client::get_message::{MessageByConnection, Message};

            let credential_offer = aries_vcx::messages::issuance::credential_offer::test_utils::_credential_offer();

            faber.activate().await.unwrap();
            faber.connection.send_message_closure().unwrap()(credential_offer.to_a2a_message()).await.unwrap();

            alice.activate().await.unwrap();

            let msgs = alice.connection.download_messages(Some(vec![MessageStatusCode::Received]), None).await.unwrap();
            let message: Message = msgs[0].clone();
            let decrypted_msg = message.decrypted_msg.unwrap();
            let _payload: aries_vcx::messages::issuance::credential_offer::CredentialOffer = serde_json::from_str(&decrypted_msg).unwrap();

            alice.connection.update_message_status(&message.uid).await.unwrap()
        }
    }

    #[cfg(feature = "agency_v2")]
    #[tokio::test]
    async fn test_download_messages() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer1 = Alice::setup().await;
        let mut consumer2 = Alice::setup().await;
        let (consumer1_to_institution, institution_to_consumer1) = create_connected_connections(&mut consumer1, &mut institution).await;
        let (consumer2_to_institution, institution_to_consumer2) = create_connected_connections(&mut consumer2, &mut institution).await;

        consumer1.activate().await.unwrap();
        consumer1_to_institution.send_generic_message("Hello Institution from consumer1").await.unwrap();
        consumer2.activate().await.unwrap();
        consumer2_to_institution.send_generic_message("Hello Institution from consumer2").await.unwrap();

        institution.activate().await.unwrap();

        let consumer1_msgs = institution_to_consumer1.download_messages(None, None).await.unwrap();
        assert_eq!(consumer1_msgs.len(), 2);

        let consumer2_msgs = institution_to_consumer2.download_messages(None, None).await.unwrap();
        assert_eq!(consumer2_msgs.len(), 2);

        let consumer1_received_msgs = institution_to_consumer1.download_messages(Some(vec![MessageStatusCode::Received]), None).await.unwrap();
        assert_eq!(consumer1_received_msgs.len(), 1);

        let consumer1_reviewed_msgs = institution_to_consumer1.download_messages(Some(vec![MessageStatusCode::Reviewed]), None).await.unwrap();
        assert_eq!(consumer1_reviewed_msgs.len(), 1);
    }

    #[cfg(feature = "agency_v2")]
    #[tokio::test]
    async fn test_update_agency_messages() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer1 = Alice::setup().await;
        let (alice_to_faber, faber_to_alice) = create_connected_connections(&mut consumer1, &mut institution).await;

        faber_to_alice.send_generic_message("Hello 1").await.unwrap();
        faber_to_alice.send_generic_message("Hello 2").await.unwrap();
        faber_to_alice.send_generic_message("Hello 3").await.unwrap();

        thread::sleep(Duration::from_millis(1000));
        consumer1.activate().await.unwrap();

        let received = alice_to_faber.download_messages(Some(vec![MessageStatusCode::Received]), None).await.unwrap();
        assert_eq!(received.len(), 3);
        let uid = received[0].uid.clone();

        let reviewed = alice_to_faber.download_messages(Some(vec![MessageStatusCode::Reviewed]), None).await.unwrap();
        let reviewed_count_before = reviewed.len();

        let pairwise_did = alice_to_faber.pairwise_info().pw_did.clone();
        let message = serde_json::to_string(&vec![UIDsByConn { pairwise_did: pairwise_did.clone(), uids: vec![uid.clone()] }]).unwrap();
        update_agency_messages("MS-106", &message).await.unwrap();

        let received = alice_to_faber.download_messages(Some(vec![MessageStatusCode::Received]), None).await.unwrap();
        assert_eq!(received.len(), 2);

        let reviewed = alice_to_faber.download_messages(Some(vec![MessageStatusCode::Reviewed]), None).await.unwrap();
        let reviewed_count_after = reviewed.len();
        assert_eq!(reviewed_count_after, reviewed_count_before + 1);

        let specific_review = alice_to_faber.download_messages(Some(vec![MessageStatusCode::Reviewed]), Some(vec![uid.clone()])).await.unwrap();
        assert_eq!(specific_review[0].uid, uid);
    }

    #[cfg(feature = "agency_v2")]
    #[tokio::test]
    async fn test_download_messages_from_multiple_connections() {
        let _setup = SetupLibraryAgencyV2::init().await;
        let mut institution = Faber::setup().await;
        let mut consumer1 = Alice::setup().await;
        let mut consumer2 = Alice::setup().await;
        let (consumer1_to_institution, institution_to_consumer1) = create_connected_connections(&mut consumer1, &mut institution).await;
        let (consumer2_to_institution, institution_to_consumer2) = create_connected_connections(&mut consumer2, &mut institution).await;

        consumer1.activate().await.unwrap();
        consumer1_to_institution.send_generic_message("Hello Institution from consumer1").await.unwrap();
        consumer2.activate().await.unwrap();
        consumer2_to_institution.send_generic_message("Hello Institution from consumer2").await.unwrap();

        institution.activate().await.unwrap();
        let consumer1_msgs = institution_to_consumer1.download_messages(None, None).await.unwrap();
        assert_eq!(consumer1_msgs.len(), 2);

        let consumer2_msgs = institution_to_consumer2.download_messages(None, None).await.unwrap();
        assert_eq!(consumer2_msgs.len(), 2);
    }

    #[cfg(feature = "pool_tests")]
    #[tokio::test]
    async fn test_get_credential_def() {
        let _setup = SetupWithWalletAndAgency::init().await;
        let (_, _, cred_def_id, cred_def_json, _, _) = create_and_store_credential_def(utils::constants::DEFAULT_SCHEMA_ATTRS, false).await;

        let (id, r_cred_def_json) = libindy::utils::anoncreds::get_cred_def_json(&cred_def_id).await.unwrap();

        assert_eq!(id, cred_def_id);
        let def1: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let def2: serde_json::Value = serde_json::from_str(&r_cred_def_json).unwrap();
        assert_eq!(def1, def2);
    }
}
