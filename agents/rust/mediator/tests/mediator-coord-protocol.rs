mod common;

use aries_vcx_core::wallet::base_wallet::BaseWallet;
use mediation::didcomm_types::mediator_coord_structs::{
    KeylistData, KeylistQueryData, KeylistUpdateItem, KeylistUpdateItemAction,
    KeylistUpdateRequestData, MediatorCoordMsgEnum,
};

use crate::common::{
    agent_and_transport_utils::{
        gen_mediator_connected_agent, send_message_and_pop_response_message,
    },
    prelude::*,
    test_setup::setup_env_logging,
};

static LOGGING_INIT: std::sync::Once = std::sync::Once::new();

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
