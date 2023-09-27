use std::{collections::HashMap, sync::Arc, time::Duration};

use agency_client::httpclient::post_message;
use aries_vcx::{
    common::{
        signing::unpack_message_to_string,
        test_utils::{create_and_write_test_cred_def, create_and_write_test_schema},
    },
    core::profile::{
        ledger::VcxPoolConfig, modular_libs_profile::ModularLibsProfile, profile::Profile,
    },
    errors::error::VcxResult,
    global::settings,
    protocols::{
        connection::Connection,
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
            holder::{
                states::{offer_received::OfferReceived, proposal_prepared::ProposalPrepared},
                HolderV2,
            },
            issuer::{states::proposal_received::ProposalReceived, IssuerV2},
        },
        mediated_connection::pairwise_info::PairwiseInfo,
    },
    transport::Transport,
    utils::{devsetup::SetupProfile, encryption_envelope::EncryptionEnvelope},
};
use aries_vcx_core::wallet::{
    base_wallet::BaseWallet,
    indy::{wallet::create_and_open_wallet, IndySdkWallet, WalletConfig, WalletConfigBuilder},
};
use async_trait::async_trait;
use messages::{
    msg_fields::protocols::{
        connection::response::Response,
        cred_issuance::{
            common::CredentialAttr,
            v2::{
                issue_credential::IssueCredentialV2, offer_credential::OfferCredentialV2,
                CredentialPreviewV2,
            },
        },
    },
    AriesMessage,
};
use serde::{de::DeserializeOwned, Deserializer};
use serde_json::{json, Value};
use url::Url;

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
            &proposal_input, Some(proposal_preview.clone())
        ).await.unwrap();

        let proposal_msg = holder.get_proposal().clone();


        let issuer = IssuerV2::<ProposalReceived<HyperledgerIndyIssuerCredentialIssuanceFormat>>::from_proposal(proposal_msg);

        // issuer checks details of the proposal
        let (received_filter, received_proposal_preview) = issuer.get_proposal_details().unwrap();
        assert_eq!(received_filter, proposal_input.cred_filter);
        assert_eq!(received_proposal_preview.unwrap(), &proposal_preview);

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
        let issuer = issuer.prepare_offer(&offer_data, offer_preview.clone(), None).await.unwrap();

        let offer_msg = issuer.get_offer().clone();


        let holder = holder.receive_offer(offer_msg).unwrap();

        // holder checks details of the offer
        let (received_offer_details, received_offer_preview) = holder.get_offer_details().unwrap();
        assert_eq!(received_offer_details.cred_def_id, cred_def.get_cred_def_id());
        assert_eq!(received_offer_details.schema_id, cred_def.get_schema_id());
        assert_eq!(received_offer_preview, &offer_preview);

        // usually this would be the DID from the connection, but does not really matter
        let pw = PairwiseInfo::create(&setup.profile.inject_wallet()).await.unwrap();
        let request_input = HyperledgerIndyCreateRequestInput { my_pairwise_did: pw.pw_did, ledger: &ledger_read, anoncreds: &anoncreds };

        let holder = holder.prepare_credential_request(&request_input).await.unwrap();

        let request_msg = holder.get_request().clone();


        let issuer = issuer.receive_request(request_msg).unwrap();

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

// TODO -DELETE BELOW
#[tokio::test]
#[ignore]
async fn manual_test_holder_against_acapy() {
    let relay_external_endpoint =
        String::from("https://fa5b-203-123-120-210.ngrok-free.app/send_user_message/user-123");
    let relay_internal_endpoint =
        String::from("https://fa5b-203-123-120-210.ngrok-free.app/pop_user_message/user-123");

    fn fix_malformed_thread_decorator(msg: &mut Value) {
        // remove thread decorator if it is empty (acapy sends it empty)
        let Some(thread) = msg.get_mut("~thread") else {
            return;
        };

        if thread.as_object().unwrap().is_empty() {
            thread.take();
        }
    }

    async fn get_next_aries_msg<T: DeserializeOwned>(
        relay: &str,
        wallet: &Arc<dyn BaseWallet>,
    ) -> VcxResult<T> {
        let enc_bytes = reqwest::get(relay)
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap()
            .to_vec();

        let unpacked = wallet.unpack_message(&enc_bytes).await?;
        let unpacked = serde_json::from_slice::<Value>(&unpacked)?;
        let msg_str = unpacked["message"].as_str().unwrap().clone();
        let mut msg = serde_json::from_str(msg_str)?;
        fix_malformed_thread_decorator(&mut msg);
        Ok(serde_json::from_value(msg)?)
    }

    async fn await_next_aries_msg<T: DeserializeOwned>(
        relay: &str,
        wallet: &Arc<dyn BaseWallet>,
    ) -> T {
        loop {
            match get_next_aries_msg(relay, wallet).await {
                Ok(data) => return data,
                Err(e) => println!("failed to fetch msg, trying again: {e:?}"),
            }

            std::thread::sleep(Duration::from_millis(500))
        }
    }

    let config_wallet = WalletConfigBuilder::default()
        .wallet_name("wallet1")
        .wallet_key(settings::DEFAULT_WALLET_KEY)
        .wallet_key_derivation(settings::WALLET_KDF_RAW)
        .build()
        .unwrap();

    let wh = create_and_open_wallet(&config_wallet).await.unwrap();
    let wallet = Arc::new(IndySdkWallet::new(wh));
    let vcx_pool_config = VcxPoolConfig {
        genesis_file_path: String::from("/Users/gmulhearne/Documents/dev/rust/aries-vcx/testnet"),
        indy_vdr_config: None,
        response_cache_config: None,
    };
    let profile = ModularLibsProfile::init(wallet, vcx_pool_config).unwrap();
    let wallet = profile.inject_wallet();
    let indy_read = profile.inject_indy_ledger_read();
    let anoncreds_read = profile.inject_anoncreds_ledger_read();
    let anoncreds = profile.inject_anoncreds();

    anoncreds
        .prover_create_link_secret(settings::DEFAULT_LINK_SECRET_ALIAS)
        .await
        .ok();

    let pairwise_info = PairwiseInfo::create(&wallet).await.unwrap();
    let inviter = Connection::new_invitee(String::from("Mr Vcx"), pairwise_info.clone());

    // acccept invite
    let invitation_json = json!(
        {
            "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/invitation",
            "@id": "ade68e30-6880-47e7-9dae-b5588e41b815",
            "label": "Bob3",
            "recipientKeys": [
              "Ab2L1WaK5rhTqZFCb2RHyjQjVzygf6xAo3jAayH1r8XM"
            ],
            "serviceEndpoint": "http://localhost:8200"
          }
    );
    let invitation = serde_json::from_value(invitation_json).unwrap();

    let inviter = inviter
        .accept_invitation(&indy_read, invitation)
        .await
        .unwrap();

    let inviter = inviter
        .prepare_request(relay_external_endpoint.parse().unwrap(), vec![])
        .await
        .unwrap();
    let request_msg = inviter.get_request().clone();
    inviter
        .send_message(&wallet, &request_msg.into(), &HttpClient)
        .await
        .unwrap();

    // get and accept response
    let response = await_next_aries_msg::<Response>(&relay_internal_endpoint, &wallet).await;
    let conn = inviter
        .handle_response(&wallet, response.try_into().unwrap(), &HttpClient)
        .await
        .unwrap();

    // send back an ack
    conn.send_message(&wallet, &conn.get_ack().into(), &HttpClient)
        .await
        .unwrap();

    println!("CONN ESTABLISHED");

    // start the credential fun :)

    // get offer
    println!("WAITING FOR CRED OFFER, GO DO IT");

    let offer = await_next_aries_msg::<OfferCredentialV2>(&relay_internal_endpoint, &wallet).await;
    println!("{offer:?}");
    println!("{}", serde_json::to_string(&offer).unwrap());

    let holder =
        HolderV2::<OfferReceived<HyperledgerIndyHolderCredentialIssuanceFormat>>::from_offer(offer);

    println!("{:?}", holder.get_offer_details().unwrap());

    // send request

    let holder = holder
        .prepare_credential_request(&HyperledgerIndyCreateRequestInput {
            my_pairwise_did: pairwise_info.pw_did,
            ledger: &anoncreds_read,
            anoncreds: &anoncreds,
        })
        .await
        .unwrap();

    let msg = holder.get_request().to_owned().into();
    conn.send_message(&wallet, &msg, &HttpClient).await.unwrap();

    // get cred
    let cred = await_next_aries_msg::<IssueCredentialV2>(&relay_internal_endpoint, &wallet).await;
    println!("{cred:?}");
    println!("{}", serde_json::to_string(&cred).unwrap());

    let holder = holder
        .receive_credential(
            cred,
            &HyperledgerIndyStoreCredentialInput {
                ledger: &anoncreds_read,
                anoncreds: &anoncreds,
            },
        )
        .await
        .unwrap();

    println!("{:?}", holder.get_stored_credential_metadata());

    // check cred made in wallet!
    let stored_cred = anoncreds
        .prover_get_credential(&holder.get_stored_credential_metadata().credential_id)
        .await
        .unwrap();

    println!("{stored_cred}");
}

// TODO - DELETE ME, for acapy test
pub struct HttpClient;
#[async_trait]
impl Transport for HttpClient {
    async fn send_message(&self, msg: Vec<u8>, service_endpoint: Url) -> VcxResult<()> {
        post_message(msg, service_endpoint).await?;
        Ok(())
    }
}
