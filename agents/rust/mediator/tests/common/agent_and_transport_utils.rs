use std::collections::VecDeque;

use aries_vcx::protocols::connection::invitee::{states::completed::Completed, InviteeConnection};
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use mediation::storage::MediatorPersistence;
use mediator::{
    aries_agent::{
        transports::{AriesReqwest, AriesTransport},
        Agent,
    },
    utils::{structs::VerKey, GenericStringError},
};
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;
use reqwest::header::ACCEPT;

use super::prelude::*;

const ENDPOINT_ROOT: &str = "http://localhost:8005";

pub async fn didcomm_connection(
    agent: &Agent<impl BaseWallet + 'static, impl MediatorPersistence>,
    aries_transport: &mut impl AriesTransport,
) -> Result<InviteeConnection<Completed>> {
    let client = reqwest::Client::new();
    let base: Url = ENDPOINT_ROOT.parse().unwrap();
    let endpoint_register = base.join("register").unwrap();

    let oobi: OOBInvitation = client
        .get(endpoint_register)
        .header(ACCEPT, "application/json")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    info!("Got invitation from register endpoint {:?}", oobi);

    let state: InviteeConnection<Completed> =
        agent.establish_connection(oobi, aries_transport).await?;

    Ok(state)
}

/// Returns agent, aries transport for agent, agent's verkey, and mediator's diddoc.
pub async fn gen_mediator_connected_agent() -> Result<(
    Agent<impl BaseWallet + 'static, impl MediatorPersistence>,
    impl AriesTransport,
    VerKey,
    AriesDidDoc,
)> {
    let agent = mediator::aries_agent::AgentBuilder::new_demo_agent().await?;
    let mut aries_transport = AriesReqwest {
        response_queue: VecDeque::new(),
        client: reqwest::Client::new(),
    };
    let completed_connection = didcomm_connection(&agent, &mut aries_transport).await?;
    let our_verkey: VerKey = completed_connection.pairwise_info().pw_vk.clone();
    let their_diddoc = completed_connection.their_did_doc().clone();
    Ok((agent, aries_transport, our_verkey, their_diddoc))
}

/// Sends message over didcomm connection and returns unpacked response message
pub async fn send_message_and_pop_response_message(
    message_bytes: &[u8],
    agent: &Agent<impl BaseWallet + 'static, impl MediatorPersistence>,
    aries_transport: &mut impl AriesTransport,
    our_verkey: &VerKey,
    their_diddoc: &AriesDidDoc,
) -> Result<String> {
    agent
        .pack_and_send_didcomm(message_bytes, our_verkey, their_diddoc, aries_transport)
        .await
        .map_err(|err| GenericStringError { msg: err })?;
    // unpack
    let response = aries_transport.pop_aries_envelope()?;
    let unpacked_response = agent
        .unpack_didcomm(&serde_json::to_vec(&response).unwrap())
        .await
        .unwrap();
    Ok(unpacked_response.message)
}
