use std::sync::Arc;

// use aries_vcx::protocols::connection::initiation_type::Invitee;
use aries_vcx::protocols::connection::invitee::states::initial::Initial as ClientInit;
// use aries_vcx::protocols::connection::invitee::states::invited::Invited;
use aries_vcx::protocols::connection::invitee::states::requested::Requested as ClientRequestSent;
// use aries_vcx::protocols::oob;
use aries_vcx::utils::mockdata::profile::mock_ledger::MockLedger;
use aries_vcx::{
    handlers::util::AnyInvitation,
    protocols::{
        connection::invitee::{states::completed::Completed, InviteeConnection},
        mediated_connection::pairwise_info::PairwiseInfo,
    },
    utils::encryption_envelope::EncryptionEnvelope,
};
use aries_vcx_core::ledger::base_ledger::IndyLedgerRead;
use messages::{
    msg_fields::protocols::{
        connection::{response::Response, Connection},
        out_of_band::invitation::Invitation as OOBInvitation,
    },
    AriesMessage,
};

// use diddoc_legacy::aries::service::AriesService;
use super::Agent;
use crate::utils::prelude::*;
// client role utilities
impl Agent {
    /// Starts a new connection object and tries to create request to the specified OOB invite
    /// endpoint
    pub async fn gen_client_connect_req(
        &self,
        oob_invite: OOBInvitation,
    ) -> Result<(InviteeConnection<ClientRequestSent>, EncryptionEnvelope), String> {
        let (pw_did, pw_vk) = self
            .wallet
            .create_and_store_my_did(None, None)
            .await
            .unwrap();

        let mock_ledger: Arc<dyn IndyLedgerRead> = Arc::new(MockLedger {}); // not good. will be dealt later. (can see brutish attempt above)
        let client_conn = InviteeConnection::<ClientInit>::new_invitee(
            "foo".into(),
            PairwiseInfo { pw_did, pw_vk },
        )
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
                &self.wallet,
                &AriesMessage::Connection(Connection::Request(req_msg.clone())),
            )
            .await
            .unwrap();

        Ok((client_conn, EncryptionEnvelope(packed_aries_msg_bytes)))
    }

    pub async fn handle_response(
        &self,
        state: InviteeConnection<ClientRequestSent>,
        response: Response,
    ) -> Result<InviteeConnection<Completed>, String> {
        state
            .handle_response(&self.get_wallet_ref(), response)
            .await
            .map_err(|err| err.to_string())
    }
    pub async fn save_completed_as_contact(
        &self,
        state: &InviteeConnection<Completed>,
    ) -> Result<(), String> {
        let their_vk = state.remote_vk().map_err(|e| e.to_string())?;
        let our_vk = &state.pairwise_info().pw_vk;
        self.create_account(&their_vk, &our_vk, state.their_did_doc())
            .await?;
        Ok(())
    }
}
