#[cfg(test)]
mod demo_test {
    use std::collections::HashMap;

    use messages::{
        decorators::thread::Thread,
        msg_fields::protocols::{
            cred_issuance::{CredentialAttr, CredentialPreview},
            notification::ack::{Ack, AckContent, AckDecorators, AckStatus},
        },
    };

    use crate::{
        core::profile::profile::Profile,
        protocols::issuance_v2::{
            formats::issuer::{
                anoncreds::{
                    AnoncredsCreateCredentialInput, AnoncredsCreateCredentialRevocationInfoInput,
                    AnoncredsCreateOfferInput, AnoncredsIssuerCredentialIssuanceFormat,
                },
                ld_proof_vc::LdProofIssuerCredentialIssuanceFormat,
            },
            issuer::{states::CredentialPrepared, IssuerV2},
            messages::{ProposeCredentialV2, RequestCredentialV2},
        },
        utils::mockdata::profile::mock_profile::MockProfile,
    };

    fn dummy_ack() -> Ack {
        Ack::builder()
            .id(String::new())
            .decorators(
                AckDecorators::builder()
                    .thread(Thread::builder().thid(String::from("thid")).build())
                    .build(),
            )
            .content(AckContent::builder().status(AckStatus::Ok).build())
            .build()
    }

    #[tokio::test]
    async fn classic_anoncreds_demo() {
        // ----- setup
        let profile = MockProfile;
        let anoncreds = profile.inject_anoncreds();

        let (cred_def_id, _) = anoncreds
            .issuer_create_and_store_credential_def("issuer_did", "schema_json", "tag", None, "config_json")
            .await
            .unwrap();

        let tails_dir = "dir";
        let (rev_reg_id, _, _) = anoncreds
            .issuer_create_and_store_revoc_reg("issuer_did", "cred_def_id", tails_dir, 100, "tag")
            .await
            .unwrap();

        // ------ receive incoming proposal

        let proposal = ProposeCredentialV2;

        let issuer = IssuerV2::from_proposal(proposal);

        // ------ respond with offer

        let offer_data = AnoncredsCreateOfferInput {
            anoncreds: &anoncreds,
            cred_def_id,
        };

        let cred_preview = CredentialPreview::new(vec![CredentialAttr {
            name: String::from("dob"),
            value: String::from("19742110"),
            mime_type: None,
        }]);

        let issuer = issuer
            .prepare_offer::<AnoncredsIssuerCredentialIssuanceFormat>(&offer_data, None, Some(cred_preview))
            .await
            .unwrap();

        let _offer = issuer.get_offer();
        // send_msg(offer.into())

        // ------- receive request

        let request = RequestCredentialV2;

        let issuer = issuer.receive_request(request);

        // ------- respond with credential

        let prep_cred_data = AnoncredsCreateCredentialInput {
            anoncreds: &anoncreds,
            credential_attributes: HashMap::from([(String::from("dob"), String::from("19742110"))]),
            revocation_info: Some(AnoncredsCreateCredentialRevocationInfoInput {
                registry_id: rev_reg_id,
                tails_directory: tails_dir.to_owned(),
            }),
        };

        let issuer = issuer
            .prepare_credential::<AnoncredsIssuerCredentialIssuanceFormat>(&prep_cred_data, None, Some(true))
            .await
            .unwrap();

        let _credential = issuer.get_credential();
        // send_msg(credential.into())

        // ------ receive ack
        let ack = dummy_ack();

        let _issuer = issuer.complete_with_ack(ack);
    }

    #[tokio::test]
    async fn ld_proof_vc_demo() {
        // ------ receive incoming proposal

        let proposal = ProposeCredentialV2;

        let issuer = IssuerV2::from_proposal(proposal);

        // ------ respond with offer

        let issuer = issuer
            .prepare_offer::<LdProofIssuerCredentialIssuanceFormat>(&(), None, None)
            .await
            .unwrap();

        let _offer = issuer.get_offer();
        // send_msg(offer.into())

        // ------- receive request

        let request = RequestCredentialV2;

        let issuer = issuer.receive_request(request);

        // ------- respond with credential

        let issuer = issuer
            .prepare_credential::<LdProofIssuerCredentialIssuanceFormat>(&(), None, None)
            .await
            .unwrap();

        let _credential = issuer.get_credential();
        // send_msg(credential.into())

        // ------ receive ack
        let ack = dummy_ack();

        let _issuer = issuer.complete_with_ack(ack);
    }

    #[tokio::test]
    async fn multi_cred_ld_proof_vc_from_request_demo() {
        // ------ initialize with request received
        let request = RequestCredentialV2;

        let issuer = IssuerV2::from_request::<LdProofIssuerCredentialIssuanceFormat>(request).unwrap();

        // ------ respond with first cred, and show intent to issue `4` more creds

        let mut issuer = issuer
            .prepare_credential::<LdProofIssuerCredentialIssuanceFormat>(&(), Some(4), Some(true))
            .await
            .unwrap();

        let _credential = issuer.get_credential();
        // send_msg(credential.into())

        // ------- iterate until all creds are sent

        async fn send_another_cred(issuer: IssuerV2<CredentialPrepared>) -> IssuerV2<CredentialPrepared> {
            // receive request message
            let request = RequestCredentialV2;

            let requested_issuer = issuer.receive_request_for_more(request).unwrap();
            let issuer = requested_issuer
                .prepare_credential::<LdProofIssuerCredentialIssuanceFormat>(&(), None, None)
                .await
                .unwrap();

            let _credential = issuer.get_credential();
            // send_msg(credential.into())
            issuer
        }

        loop {
            if issuer.remaining_credentials_to_issue() == 0 {
                break;
            }

            issuer = send_another_cred(issuer).await;
        }

        // ------- finish and receive ack

        let ack = dummy_ack();

        let _issuer = issuer.complete_with_ack(ack);
    }
}
