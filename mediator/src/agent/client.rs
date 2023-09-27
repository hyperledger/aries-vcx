use std::sync::Arc;

use aries_vcx::handlers::util::AnyInvitation;
// use aries_vcx::protocols::connection::initiation_type::Invitee;
use aries_vcx::protocols::connection::invitee::states::initial::Initial as ClientInit;
// use aries_vcx::protocols::connection::invitee::states::invited::Invited;
use aries_vcx::protocols::connection::invitee::states::requested::Requested as ClientRequestSent;
use aries_vcx::protocols::connection::invitee::InviteeConnection;

use aries_vcx::protocols::mediated_connection::pairwise_info::PairwiseInfo;
use aries_vcx::utils::encryption_envelope::EncryptionEnvelope;
// use aries_vcx::protocols::oob;
use aries_vcx::utils::mockdata::profile::mock_ledger::MockLedger;
use aries_vcx_core::ledger::base_ledger::IndyLedgerRead;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use aries_vcx_core::wallet::indy::IndySdkWallet;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::msg_fields::protocols::connection::Connection;
use messages::AriesMessage;
// use diddoc_legacy::aries::service::AriesService;
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;
use messages::msg_fields::protocols::out_of_band::invitation::OobService;
use serde_json::Value;

use super::Agent;
use crate::utils::prelude::*;

pub fn oob2did(oob: OOBInvitation) -> AriesDidDoc {
    let mut did_doc: AriesDidDoc = AriesDidDoc::default();
    did_doc.set_id(oob.id.clone());
    let oob_service = oob.content.services.first().expect("OOB needs a service").clone();

    match oob_service {
        OobService::AriesService(service) => {
            did_doc.set_service_endpoint(service.service_endpoint);
            did_doc.set_recipient_keys(service.recipient_keys);
            did_doc.set_routing_keys(service.routing_keys);
        }
        _ => panic!("Assuming fully clean AriesService variant only"),
    }
    did_doc
}

// client role utilities
impl Agent<IndySdkWallet> {
    /// Starts a new connection object and tries to create request to the specified OOB invite endpoint
    pub async fn client_connect_req(
        &self,
        oob_invite: OOBInvitation,
    ) -> Result<(InviteeConnection<ClientRequestSent>, Value), String> {
        let (pw_did, pw_vk) = self.wallet.create_and_store_my_did(None, None).await.unwrap();

        let mock_ledger: Arc<dyn IndyLedgerRead> = Arc::new(MockLedger {}); // not good. will be dealt later. (can see brutish attempt above)
        let client_conn = InviteeConnection::<ClientInit>::new_invitee("foo".into(), PairwiseInfo { pw_did, pw_vk })
            .accept_invitation(&mock_ledger, AnyInvitation::Oob(oob_invite.clone()))
            .await
            .unwrap();

        let client_conn = client_conn
            .prepare_request("http://response.http.alt".parse().unwrap(), vec![])
            .await
            .unwrap();

        let msg_connection_request = client_conn.get_request().clone();
        info!("Client Connection Request: {:#?}", msg_connection_request);
        let req_msg = client_conn.get_request();
        debug!(
            "Connection Request: {},",
            serde_json::to_string_pretty(&req_msg).unwrap()
        );
        // encrypt/pack connection request
        let EncryptionEnvelope(packed_aries_msg_bytes) = client_conn
            .encrypt_message(
                &self.wallet_ref,
                &AriesMessage::Connection(Connection::Request(req_msg.clone())),
            )
            .await
            .unwrap();
        let packed_aries_msg_json: Value =
            serde_json::from_slice(&packed_aries_msg_bytes[..]).expect("Envelope content should be serializable json");
        info!(
            "Sending Connection Request Envelope: {},",
            serde_json::to_string_pretty(&packed_aries_msg_json).unwrap()
        );
        let oob_invited_endpoint = oob2did(oob_invite).get_endpoint().expect("Service needs an endpoint");
        let http_client = reqwest::Client::new();
        let res = http_client
            .post(oob_invited_endpoint)
            .json(&packed_aries_msg_json)
            .send()
            .await
            .expect("Something went wrong while sending/receiving");
        debug!("Received response {:#?}", res);
        let Ok(_res_ref) = res.error_for_status_ref() else {
            return Err(format!("{:#?} {:#?}", res.status().as_u16(), res.text().await));
        };
        let res_status = res.status().as_u16();
        let res_body = res
            .text()
            .await
            .expect("Reading response body is a trivial expectation");
        info!("Response {:#?} {:#?}", res_status, res_body);
        let Ok(res_json) = serde_json::from_str::<Value>(&res_body) else {
            return Err(format!("Couldn't decode response body to json, got {:#?}", res_body));
        };
        debug!(
            "Received response json: {},",
            serde_json::to_string_pretty(&res_json).unwrap()
        );
        Ok((client_conn, res_json))
    }
}
