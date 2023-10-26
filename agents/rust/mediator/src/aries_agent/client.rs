use aries_vcx::{
    handlers::util::AnyInvitation,
    protocols::{
        connection::invitee::{
            states::{
                completed::Completed, initial::Initial as ClientInit,
                requested::Requested as ClientRequestSent,
            },
            InviteeConnection,
        },
        mediated_connection::pairwise_info::PairwiseInfo,
    },
    utils::encryption_envelope::EncryptionEnvelope,
};
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use mediation::storage::MediatorPersistence;
use messages::{
    msg_fields::protocols::{
        connection::{response::Response, Connection},
        out_of_band::invitation::Invitation as OOBInvitation,
    },
    AriesMessage,
};
use test_utils::mockdata::mock_ledger::MockLedger;

use super::{transports::AriesTransport, Agent};
use crate::{aries_agent::utils::oob2did, utils::prelude::*};

// client role utilities
impl<T: BaseWallet + 'static, P: MediatorPersistence> Agent<T, P> {
    /// Starting from a new connection object, tries to create connection request object for the
    /// specified OOB invite endpoint
    pub async fn gen_connection_request(
        &self,
        oob_invite: OOBInvitation,
    ) -> Result<(InviteeConnection<ClientRequestSent>, EncryptionEnvelope), String> {
        // Generate keys
        let (pw_did, pw_vk) = self
            .wallet
            .create_and_store_my_did(None, None)
            .await
            .unwrap();
        // Create fresh connection object and transform step wise to requested state
        let mock_ledger = MockLedger {}; // not good. to be dealt later
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
        // Extract the actual connection request message to be sent
        let msg_connection_request = client_conn.get_request().clone();
        info!("Client Connection Request: {:#?}", msg_connection_request);
        let req_msg = client_conn.get_request();
        debug!(
            "Connection Request: {},",
            serde_json::to_string_pretty(&req_msg).unwrap()
        );
        // encrypt/pack the connection request message
        let EncryptionEnvelope(packed_aries_msg_bytes) = client_conn
            .encrypt_message(
                self.wallet.as_ref(),
                &AriesMessage::Connection(Connection::Request(req_msg.clone())),
            )
            .await
            .unwrap();

        Ok((client_conn, EncryptionEnvelope(packed_aries_msg_bytes)))
    }

    pub async fn handle_connection_response(
        &self,
        state: InviteeConnection<ClientRequestSent>,
        response: Response,
    ) -> Result<InviteeConnection<Completed>, String> {
        state
            .handle_response(self.wallet.as_ref(), response)
            .await
            .map_err(|err| err.to_string())
    }
    pub async fn save_completed_connection_as_contact(
        &self,
        state: &InviteeConnection<Completed>,
    ) -> Result<(), String> {
        let their_vk = state.remote_vk().map_err(|e| e.to_string())?;
        let our_vk = &state.pairwise_info().pw_vk;
        self.create_account(&their_vk, our_vk, state.their_did_doc())
            .await?;
        Ok(())
    }

    pub async fn list_contacts(&self) -> Result<Vec<(String, String)>, String> {
        self.persistence.list_accounts().await
    }
    /// Workflow method to establish DidComm connection with Aries peer, given OOB invite.
    pub async fn establish_connection(
        &self,
        oob_invite: OOBInvitation,
        aries_transport: &mut impl AriesTransport,
    ) -> Result<InviteeConnection<Completed>, anyhow::Error> {
        let (state, EncryptionEnvelope(packed_aries_msg_bytes)) = self
            .gen_connection_request(oob_invite.clone())
            .await
            .map_err(|err| GenericStringError { msg: err })?;
        let packed_aries_msg_json = serde_json::from_slice(&packed_aries_msg_bytes)?;
        info!(
            "Sending Connection Request Envelope: {},",
            serde_json::to_string_pretty(&packed_aries_msg_json).unwrap()
        );
        aries_transport
            .push_aries_envelope(packed_aries_msg_json, &oob2did(oob_invite))
            .await?;
        let response_envelope = aries_transport.pop_aries_envelope()?;
        info!(
            "Received Response envelope {:#?}, unpacking",
            serde_json::to_string_pretty(&response_envelope).unwrap()
        );
        let response_envelope_bytes = serde_json::to_vec(&response_envelope)?;
        let response_unpacked = self
            .unpack_didcomm(&response_envelope_bytes)
            .await
            .map_err(|err| GenericStringError { msg: err })?;
        let response_message: AriesMessage = serde_json::from_str(&response_unpacked.message)?;
        let AriesMessage::Connection(Connection::Response(connection_response)) = response_message
        else {
            return Err(GenericStringError {
                msg: format!("Expected connection response, got {:?}", response_message),
            }
            .into());
        };
        let state = self
            .handle_connection_response(state, connection_response)
            .await
            .map_err(|err| GenericStringError { msg: err })?;
        info!(
            "Completed Connection {:?}, saving as contact",
            serde_json::to_value(&state).unwrap()
        );
        self.save_completed_connection_as_contact(&state)
            .await
            .map_err(|err| GenericStringError { msg: err })?;
        Ok(state)
    }
}
