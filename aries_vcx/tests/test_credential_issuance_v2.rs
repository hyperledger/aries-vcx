use std::collections::HashMap;

use aries_vcx::{
    common::test_utils::{create_and_write_test_cred_def, create_and_write_test_schema},
    protocols::{
        issuance_v2::{
            formats::{
                holder::hyperledger_indy::{
                    HyperledgerIndyCreateProposalInput, HyperledgerIndyCreateRequestInput,
                    HyperledgerIndyCredentialFilterBuilder,
                    HyperledgerIndyHolderCredentialIssuanceFormat,
                    HyperledgerIndyStoreCredentialInput,
                },
                issuer::hyperledger_indy::{
                    HyperledgerIndyCreateCredentialInput, HyperledgerIndyCreateOfferInput,
                    HyperledgerIndyIssuerCredentialIssuanceFormat,
                },
            },
            holder::{states::ProposalPrepared, HolderV2},
            issuer::{states::ProposalReceived, IssuerV2},
        },
        mediated_connection::pairwise_info::PairwiseInfo,
    },
    utils::devsetup::SetupProfile,
};
use messages::msg_fields::protocols::cred_issuance::{
    common::CredentialAttr, v2::CredentialPreviewV2,
};

#[tokio::test]
#[ignore]
async fn test_hlindy_non_revocable_credential_issuance_v2_from_proposal() {
    SetupProfile::run(|setup| async move {
        let anoncreds = setup.profile.inject_anoncreds();
        let ledger_read = setup.profile.inject_anoncreds_ledger_read();

        let schema = create_and_write_test_schema(
            &anoncreds,
            &setup.profile.inject_anoncreds_ledger_write(),
            &setup.institution_did,
            aries_vcx::utils::constants::DEFAULT_SCHEMA_ATTRS,
        )
        .await;
        let cred_def = create_and_write_test_cred_def(
            &anoncreds,
            &ledger_read,
            &setup.profile.inject_anoncreds_ledger_write(),
            &setup.institution_did,
            &schema.schema_id,
            false,
        ).await;

        let proposal_input = HyperledgerIndyCreateProposalInput {
            cred_filter: HyperledgerIndyCredentialFilterBuilder::default().cred_def_id(cred_def.get_cred_def_id()).build().unwrap()
        };
        let proposal_preview = CredentialPreviewV2::new(
            vec![CredentialAttr::builder().name(String::from("address")).value(String::from("123 Main St")).build()]
        );
        let holder = HolderV2::<ProposalPrepared<HyperledgerIndyHolderCredentialIssuanceFormat>>::with_proposal(
            &proposal_input, Some(proposal_preview)
        ).await.unwrap();

        let proposal_msg = holder.get_proposal().clone();


        let issuer = IssuerV2::<ProposalReceived<HyperledgerIndyIssuerCredentialIssuanceFormat>>::from_proposal(proposal_msg);

        // TODO - would be good if issuer had an easy way to view what was proposed

        let offer_data = HyperledgerIndyCreateOfferInput { anoncreds: &anoncreds, cred_def_id: cred_def.get_cred_def_id() };
        let offer_preview = CredentialPreviewV2::new(
            vec![
                CredentialAttr::builder().name(String::from("address1")).value(String::from("123 Main St")).build(),
                CredentialAttr::builder().name(String::from("address2")).value(String::from("Suite 3")).build(),
                CredentialAttr::builder().name(String::from("city")).value(String::from("Draper")).build(),
                CredentialAttr::builder().name(String::from("state")).value(String::from("UT")).build(),
                CredentialAttr::builder().name(String::from("zip")).value(String::from("84000")).build(),
            ]
        );
        let issuer = issuer.prepare_offer(&offer_data, offer_preview, None).await.unwrap();

        let offer_msg = issuer.get_offer().clone();


        let holder = holder.receive_offer(offer_msg).unwrap();

        // usually this would be the DID from the connection, but does not really matter
        let pw = PairwiseInfo::create(&setup.profile.inject_wallet()).await.unwrap();
        let request_input = HyperledgerIndyCreateRequestInput { my_pairwise_did: pw.pw_did, ledger: &ledger_read, anoncreds: &anoncreds };

        let holder = holder.prepare_credential_request(&request_input).await.unwrap();

        let request_msg = holder.get_request().clone();


        let issuer = issuer.receive_request(request_msg);

        let cred_data = HyperledgerIndyCreateCredentialInput { anoncreds: &anoncreds, credential_attributes: HashMap::from([
            (String::from("address1"), String::from("123 Main St")),
            (String::from("address2"), String::from("Suite 3")),
            (String::from("city"), String::from("Draper")),
            (String::from("state"), String::from("UT")),
            (String::from("zip"), String::from("84000")),
        ]), revocation_info: None };

        let issuer = issuer.prepare_credential(&cred_data, Some(true), None).await.unwrap();
        let issuer_cred_metadata = issuer.get_credential_creation_metadata().clone();

        let cred_msg = issuer.get_credential().clone();

        let receive_input = HyperledgerIndyStoreCredentialInput { ledger: &ledger_read, anoncreds: &anoncreds };

        let holder = holder.receive_credential(cred_msg, &receive_input).await.unwrap();
        let holder_cred_metadata = holder.get_stored_credential_metadata().clone();
        let holder = holder.prepare_ack_if_required();

        let ack_msg = holder.get_ack().unwrap().clone();

        let _issuer = issuer.complete_with_ack(ack_msg);

        // check final states
        assert!(issuer_cred_metadata.credential_revocation_id.is_none());

        let holder_cred_id = holder_cred_metadata.credential_id;
        let cred = anoncreds.prover_get_credential(&holder_cred_id).await.unwrap();
        assert!(!cred.is_empty());

    }).await
}
