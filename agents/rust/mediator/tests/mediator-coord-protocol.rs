mod common;
use std::collections::VecDeque;

use aries_vcx::{
    protocols::connection::invitee::{states::completed::Completed, InviteeConnection},
    utils::encryption_envelope::EncryptionEnvelope,
};
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use mediation::{
    didcomm_types::mediator_coord_structs::{
        KeylistData, KeylistQueryData, KeylistUpdateItem, KeylistUpdateItemAction,
        KeylistUpdateRequestData, MediatorCoordMsgEnum,
    },
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

use crate::common::{prelude::*, test_setup::setup_env_logging};

static LOGGING_INIT: std::sync::Once = std::sync::Once::new();

const ENDPOINT_ROOT: &str = "http://localhost:8005";

async fn didcomm_connection(
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
async fn gen_mediator_connected_agent() -> Result<(
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
async fn send_message_and_pop_response_message(
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
    info!(
        "Packed: {:?}, sending",
        serde_json::to_string(&packed_json).unwrap()
    );
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

#[tokio::test]
#[ignore]
async fn test_init() {
    LOGGING_INIT.call_once(setup_env_logging);
    let agent = mediator::aries_agent::AgentBuilder::new_demo_agent()
        .await
        .unwrap();
    let mut aries_transport = AriesReqwest {
        response_queue: VecDeque::new(),
        client: reqwest::Client::new(),
    };
    let _ = didcomm_connection(&agent, &mut aries_transport).await;
    let _ = didcomm_connection(&agent, &mut aries_transport).await;
}

#[tokio::test]
async fn test_mediate_grant() -> Result<()> {
    LOGGING_INIT.call_once(setup_env_logging);
    // prepare connection parameters
    let (agent, mut aries_transport, our_verkey, their_diddoc) =
        gen_mediator_connected_agent().await?;
    // prepare request message
    let message = MediatorCoordMsgEnum::MediateRequest;
    let message_bytes = serde_json::to_vec(&message)?;
    // send message and get response
    let response_message = send_message_and_pop_response_message(
        &message_bytes,
        &agent,
        &mut aries_transport,
        &our_verkey,
        &their_diddoc,
    )
    .await?;
    // verify response
    if let MediatorCoordMsgEnum::MediateGrant(grant_data) =
        serde_json::from_str(&response_message).unwrap()
    {
        info!("Grant Data {:?}", grant_data);
    } else if let MediatorCoordMsgEnum::MediateDeny(deny_data) =
        serde_json::from_str(&response_message).unwrap()
    {
        info!("Deny Data {:?}", deny_data);
    } else {
        panic!(
            "Should get response that is of type Mediator Grant / Deny. Found {:?}",
            response_message
        )
    };

    Ok(())
}

#[tokio::test]
async fn test_mediate_keylist_update_add() -> Result<()> {
    LOGGING_INIT.call_once(setup_env_logging);
    // prepare connection parameters
    let (agent, mut aries_transport, our_verkey, their_diddoc) =
        gen_mediator_connected_agent().await?;
    // prepare request message
    let (_, new_vk) = agent
        .get_wallet_ref()
        .create_and_store_my_did(None, None)
        .await?;
    let message = MediatorCoordMsgEnum::KeylistUpdateRequest(KeylistUpdateRequestData {
        updates: vec![KeylistUpdateItem {
            recipient_key: new_vk,
            action: KeylistUpdateItemAction::Add,
            result: None,
        }],
    });
    info!("Sending {:?}", serde_json::to_string(&message).unwrap());
    let message_bytes = serde_json::to_vec(&message)?;
    // send message and get response
    let response_message = send_message_and_pop_response_message(
        &message_bytes,
        &agent,
        &mut aries_transport,
        &our_verkey,
        &their_diddoc,
    )
    .await?;
    // verify response
    if let MediatorCoordMsgEnum::KeylistUpdateResponse(update_response_data) =
        serde_json::from_str(&response_message)?
    {
        info!("Received update response {:?}", update_response_data);
    } else {
        panic!(
            "Expected message of type KeylistUpdateResponse. Found {:?}",
            response_message
        )
    }

    Ok(())
}

#[tokio::test]
async fn test_mediate_keylist_query() -> Result<()> {
    LOGGING_INIT.call_once(setup_env_logging);
    // prepare connection parameters
    let (agent, mut aries_transport, our_verkey, their_diddoc) =
        gen_mediator_connected_agent().await?;
    // prepare request message: add key
    let (_, new_vk) = agent
        .get_wallet_ref()
        .create_and_store_my_did(None, None)
        .await?;
    let message = MediatorCoordMsgEnum::KeylistUpdateRequest(KeylistUpdateRequestData {
        updates: vec![KeylistUpdateItem {
            recipient_key: new_vk,
            action: KeylistUpdateItemAction::Add,
            result: None,
        }],
    });
    let message_bytes = serde_json::to_vec(&message)?;
    // send message and get response
    let _ = send_message_and_pop_response_message(
        &message_bytes,
        &agent,
        &mut aries_transport,
        &our_verkey,
        &their_diddoc,
    )
    .await?;
    info!("Proceeding to keylist query");
    //prepare request message: list keys
    let message = MediatorCoordMsgEnum::KeylistQuery(KeylistQueryData {});
    info!("Sending {:?}", serde_json::to_string(&message).unwrap());
    let message_bytes = serde_json::to_vec(&message)?;
    // send message and get response
    let response_message = send_message_and_pop_response_message(
        &message_bytes,
        &agent,
        &mut aries_transport,
        &our_verkey,
        &their_diddoc,
    )
    .await?;
    // verify
    if let MediatorCoordMsgEnum::Keylist(KeylistData { keys }) =
        serde_json::from_str(&response_message)?
    {
        info!("Keylist mediator sent {:?}", keys)
    } else {
        panic!(
            "Expected message of type Keylist. Found {:?}",
            response_message
        )
    }

    Ok(())
}

#[tokio::test]
async fn test_mediate_keylist_update_remove() -> Result<()> {
    LOGGING_INIT.call_once(setup_env_logging);
    // prepare connection parameters
    let (agent, mut aries_transport, our_verkey, their_diddoc) =
        gen_mediator_connected_agent().await?;
    // prepare request message: add key
    let (_, new_vk) = agent
        .get_wallet_ref()
        .create_and_store_my_did(None, None)
        .await?;
    let message = MediatorCoordMsgEnum::KeylistUpdateRequest(KeylistUpdateRequestData {
        updates: vec![KeylistUpdateItem {
            recipient_key: new_vk.clone(),
            action: KeylistUpdateItemAction::Add,
            result: None,
        }],
    });
    let message_bytes = serde_json::to_vec(&message)?;
    // send message and get response
    let _ = send_message_and_pop_response_message(
        &message_bytes,
        &agent,
        &mut aries_transport,
        &our_verkey,
        &their_diddoc,
    )
    .await?;
    info!("Proceeding to delete");
    // prepare request message: delete key
    let message = MediatorCoordMsgEnum::KeylistUpdateRequest(KeylistUpdateRequestData {
        updates: vec![KeylistUpdateItem {
            recipient_key: new_vk,
            action: KeylistUpdateItemAction::Remove,
            result: None,
        }],
    });
    info!("Sending {:?}", serde_json::to_string(&message).unwrap());
    let message_bytes = serde_json::to_vec(&message)?;
    // send message and get response
    let response_message = send_message_and_pop_response_message(
        &message_bytes,
        &agent,
        &mut aries_transport,
        &our_verkey,
        &their_diddoc,
    )
    .await?;
    if let MediatorCoordMsgEnum::KeylistUpdateResponse(update_response_data) =
        serde_json::from_str(&response_message)?
    {
        info!("Received update response {:?}", update_response_data);
    } else {
        panic!(
            "Expected message of type KeylistUpdateResponse. Found {:?}",
            response_message
        )
    }
    Ok(())
}
