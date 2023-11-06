use std::collections::VecDeque;

use aries_vcx::{
    protocols::connection::invitee::{states::completed::Completed, InviteeConnection},
    utils::encryption_envelope::EncryptionEnvelope,
};
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use mediation::{
    didcomm_types::mediator_coord_structs::{MediateGrantData, MediatorCoordMsgEnum},
    storage::MediatorPersistence,
};
use mediator::{
    aries_agent::{
        client::transports::{AriesReqwest, AriesTransport},
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
    // Wrap message in encrypted envelope
    let EncryptionEnvelope(packed_message) = agent
        .pack_didcomm(message_bytes, our_verkey, their_diddoc)
        .await
        .map_err(|e| GenericStringError { msg: e.to_string() })?;
    let packed_json = serde_json::from_slice(&packed_message)?;
    // Send serialized envelope over transport
    let response_envelope = aries_transport
        .send_aries_envelope(packed_json, their_diddoc)
        .await?;
    // unpack
    let unpacked_response = agent
        .unpack_didcomm(&serde_json::to_vec(&response_envelope).unwrap())
        .await
        .unwrap();
    Ok(unpacked_response.message)
}

pub async fn get_mediator_grant_data(
    agent: &Agent<impl BaseWallet + 'static, impl MediatorPersistence>,
    agent_aries_transport: &mut impl AriesTransport,
    agent_verkey: &VerKey,
    mediator_diddoc: &AriesDidDoc,
) -> MediateGrantData {
    // prepare request message
    let message = MediatorCoordMsgEnum::MediateRequest;
    let message_bytes = serde_json::to_vec(&message).unwrap();
    // send message and get response
    let response_message = send_message_and_pop_response_message(
        &message_bytes,
        agent,
        agent_aries_transport,
        agent_verkey,
        mediator_diddoc,
    )
    .await
    .unwrap();
    // extract routing parameters
    if let MediatorCoordMsgEnum::MediateGrant(grant_data) =
        serde_json::from_str(&response_message).unwrap()
    {
        info!("Grant Data {:?}", grant_data);
        grant_data
    } else {
        panic!(
            "Should get response that is of type Mediator Grant. Found {:?}",
            response_message
        )
    }
}
