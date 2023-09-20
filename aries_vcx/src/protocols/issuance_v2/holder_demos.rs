#[cfg(test)]
pub mod demo_test {
    use messages::msg_fields::protocols::cred_issuance::{CredentialAttr, CredentialPreview};

    use crate::{
        core::profile::profile::Profile,
        protocols::issuance_v2::{
            formats::{
                holder::anoncreds::{
                    AnoncredsCreateProposalInput, AnoncredsCreateRequestInput, AnoncredsCredentialFilter,
                    AnoncredsHolderCredentialIssuanceFormat, AnoncredsStoreCredentialInput,
                },
                holder::ld_proof_vc::LdProofHolderCredentialIssuanceFormat,
            },
            holder::{
                states::{CredentialReceived, RequestPrepared},
                HolderV2,
            },
            messages::{IssueCredentialV2, OfferCredentialV2},
        },
        utils::mockdata::profile::mock_profile::MockProfile,
    };

    #[tokio::test]
    async fn classic_anoncreds_demo() {
        let profile = MockProfile;
        let anoncreds = profile.inject_anoncreds();
        let ledger_read = profile.inject_anoncreds_ledger_read();

        // ----- create proposal

        let my_proposal_data = AnoncredsCreateProposalInput {
            cred_filter: AnoncredsCredentialFilter {
                issuer_id: Some(String::from("cool-issuer-1")),
                ..Default::default()
            },
        };
        let cred_preview = CredentialPreview::new(vec![CredentialAttr {
            name: String::from("dob"),
            value: String::from("19742110"),
            mime_type: None,
        }]);

        let holder =
            HolderV2::with_proposal::<AnoncredsHolderCredentialIssuanceFormat>(&my_proposal_data, Some(cred_preview))
                .await
                .unwrap();

        let _proposal_message = holder.get_proposal().to_owned();
        // send_msg(proposal_message.into());

        // ------- receive back offer and make request

        let offer_message = OfferCredentialV2;
        let holder = holder.receive_offer(offer_message).unwrap();

        let prep_request_data = AnoncredsCreateRequestInput {
            entropy: String::from("blah-blah-blah"),
            ledger: &ledger_read,
            anoncreds: &anoncreds,
        };
        let holder = holder
            .prepare_credential_request::<AnoncredsHolderCredentialIssuanceFormat>(&prep_request_data)
            .await
            .unwrap();

        let _request_message = holder.get_request().to_owned();
        // send_msg(request_message.into());

        // ------- receive back issuance and finalize

        let issue_message = IssueCredentialV2;

        let receive_cred_data = AnoncredsStoreCredentialInput {
            ledger: &ledger_read,
            anoncreds: &anoncreds,
        };
        let holder = holder
            .receive_credential(issue_message, &receive_cred_data)
            .await
            .unwrap();

        // ------- finish and send ack if required

        let holder = holder.prepare_ack_if_required();

        if let Some(_ack) = holder.get_ack() {
            // send_msg(ack.into())
        }
    }

    #[tokio::test]
    async fn ld_proof_vc_demo() {
        // ----- create proposal

        let holder = HolderV2::with_proposal::<LdProofHolderCredentialIssuanceFormat>(&(), None)
            .await
            .unwrap();

        let _proposal_message = holder.get_proposal().to_owned();
        // send_msg(proposal_message.into());

        // ------- receive back offer and make request

        let offer_message = OfferCredentialV2;
        let holder = holder.receive_offer(offer_message).unwrap();

        let holder = holder
            .prepare_credential_request::<LdProofHolderCredentialIssuanceFormat>(&())
            .await
            .unwrap();

        let _request_message = holder.get_request().to_owned();
        // send_msg(request_message.into());

        // ------- receive back issuance and finalize

        let issue_message = IssueCredentialV2;

        let holder = holder.receive_credential(issue_message, &()).await.unwrap();

        // ------- finish and send ack if required

        let holder = holder.prepare_ack_if_required();

        if let Some(_ack) = holder.get_ack() {
            // send_msg(ack.into())
        }
    }

    #[tokio::test]
    async fn multi_cred_ld_proof_vc_from_request_demo() {
        // ----- initialize with request

        let requested_holder: HolderV2<RequestPrepared<LdProofHolderCredentialIssuanceFormat>> =
            HolderV2::with_request(&()).await.unwrap();

        let _request_message = requested_holder.get_request().to_owned();
        // send_msg(request_message.into());

        // ------- receive back issuance and iterate

        let issue_message = IssueCredentialV2;

        let mut holder = requested_holder.receive_credential(issue_message, &()).await.unwrap();

        async fn receive_another_cred(
            holder: HolderV2<CredentialReceived<LdProofHolderCredentialIssuanceFormat>>,
        ) -> HolderV2<CredentialReceived<LdProofHolderCredentialIssuanceFormat>> {
            let requested_holder = holder.prepare_request_for_next_credential(&()).await.unwrap();

            let _request_message = requested_holder.get_request().to_owned();
            // send_msg(request_message.into());

            // receive another issuance msg
            let issue_message = IssueCredentialV2;

            requested_holder.receive_credential(issue_message, &()).await.unwrap()
        }

        loop {
            if !holder.is_more_credential_available() {
                break;
            }

            holder = receive_another_cred(holder).await;
        }

        // ------- finish and send ack if required

        let holder = holder.prepare_ack_if_required();

        if let Some(_ack) = holder.get_ack() {
            // send_msg(ack.into())
        }
    }
}
