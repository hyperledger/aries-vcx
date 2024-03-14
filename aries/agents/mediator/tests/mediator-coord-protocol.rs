mod common;

use aries_vcx_wallet::wallet::base_wallet::did_wallet::DidWallet;
use messages::{
    msg_fields::protocols::coordinate_mediation::{
        keylist_update::{KeylistUpdateItem, KeylistUpdateItemAction},
        CoordinateMediation, KeylistQuery, KeylistQueryContent, KeylistUpdate,
        KeylistUpdateContent, MediateRequest, MediateRequestContent,
    },
    AriesMessage,
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
    let mediate_request = CoordinateMediation::MediateRequest(
        MediateRequest::builder()
            .content(MediateRequestContent::default())
            .id("mediate-request-test".to_owned())
            .build(),
    );
    let message_bytes = serde_json::to_vec(&AriesMessage::CoordinateMediation(mediate_request))?;
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
    if let AriesMessage::CoordinateMediation(CoordinateMediation::MediateGrant(grant_data)) =
        serde_json::from_str(&response_message).unwrap()
    {
        info!("Grant Data {:?}", grant_data);
    } else if let AriesMessage::CoordinateMediation(CoordinateMediation::MediateDeny(deny_data)) =
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
    let did_data = agent
        .get_wallet_ref()
        .create_and_store_my_did(None, None)
        .await?;
    let keylist_update_request = KeylistUpdate::builder()
        .content(KeylistUpdateContent {
            updates: vec![KeylistUpdateItem {
                recipient_key: did_data.verkey().base58(),
                action: KeylistUpdateItemAction::Add,
            }],
        })
        .id("key-add".to_owned())
        .build();

    let message = AriesMessage::CoordinateMediation(CoordinateMediation::KeylistUpdate(
        keylist_update_request,
    ));
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
    if let AriesMessage::CoordinateMediation(CoordinateMediation::KeylistUpdateResponse(
        update_response_data,
    )) = serde_json::from_str(&response_message)?
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
    let did_data = agent
        .get_wallet_ref()
        .create_and_store_my_did(None, None)
        .await?;
    let keylist_update_request = KeylistUpdate::builder()
        .content(KeylistUpdateContent {
            updates: vec![KeylistUpdateItem {
                recipient_key: did_data.verkey().base58(),
                action: KeylistUpdateItemAction::Add,
            }],
        })
        .id("key-add".to_owned())
        .build();

    let message = AriesMessage::CoordinateMediation(CoordinateMediation::KeylistUpdate(
        keylist_update_request,
    ));
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
    let keylist_query = KeylistQuery::builder()
        .content(KeylistQueryContent::default())
        .id("keylist-query".to_owned())
        .build();
    let message =
        AriesMessage::CoordinateMediation(CoordinateMediation::KeylistQuery(keylist_query));
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
    if let AriesMessage::CoordinateMediation(CoordinateMediation::Keylist(keylist)) =
        serde_json::from_str(&response_message)?
    {
        info!("Keylist mediator sent {:?}", keylist.content)
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
    let did_data = agent
        .get_wallet_ref()
        .create_and_store_my_did(None, None)
        .await?;
    let keylist_update_request = KeylistUpdate::builder()
        .content(KeylistUpdateContent {
            updates: vec![KeylistUpdateItem {
                recipient_key: did_data.verkey().base58(),
                action: KeylistUpdateItemAction::Add,
            }],
        })
        .id("key-add".to_owned())
        .build();

    let message = AriesMessage::CoordinateMediation(CoordinateMediation::KeylistUpdate(
        keylist_update_request,
    ));
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
    let keylist_update_request = KeylistUpdate::builder()
        .content(KeylistUpdateContent {
            updates: vec![KeylistUpdateItem {
                recipient_key: did_data.verkey().base58(),
                action: KeylistUpdateItemAction::Remove,
            }],
        })
        .id("key-remove".to_owned())
        .build();

    let message = AriesMessage::CoordinateMediation(CoordinateMediation::KeylistUpdate(
        keylist_update_request,
    ));
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
    if let AriesMessage::CoordinateMediation(CoordinateMediation::KeylistUpdateResponse(
        update_response_data,
    )) = serde_json::from_str(&response_message)?
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
