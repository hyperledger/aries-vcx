mod common;
use std::collections::VecDeque;

use aries_vcx::protocols::connection::invitee::{states::completed::Completed, InviteeConnection};
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use common::{prelude::*, test_setup::OneTimeInit};
use mediator::{
    aries_agent::{
        transports::{AriesReqwest, AriesTransport},
        Agent,
    },
    utils::{structs::VeriKey, GenericStringError},
};
use messages::msg_fields::protocols::out_of_band::invitation::Invitation as OOBInvitation;
use reqwest::header::ACCEPT;
use xum_test_server::{
    didcomm_types::mediator_coord_structs::{
        KeylistData, KeylistQueryData, KeylistUpdateItem, KeylistUpdateItemAction,
        KeylistUpdateRequestData, MediatorCoordMsgEnum,
    },
    storage::MediatorPersistence,
};

const ENDPOINT_ROOT: &str = "http://localhost:8005";

struct TestSetupAries;
impl OneTimeInit for TestSetupAries {
    fn one_time_setup_code(&self) {
        fn setup_logging() {
            let env = env_logger::Env::default().default_filter_or("info");
            env_logger::init_from_env(env);
        }
        fn load_dot_env() {
            let _ = dotenvy::dotenv();
        }
        load_dot_env();
        setup_logging();
    }
}
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

#[tokio::test]
#[ignore]
async fn test_init() {
    TestSetupAries.init();
    let agent = mediator::aries_agent::AgentMaker::new_demo_agent().await.unwrap();
    let mut aries_transport = AriesReqwest {
        response_queue: VecDeque::new(),
        client: reqwest::Client::new(),
    };
    let _ = didcomm_connection(&agent, &mut aries_transport).await;
    let _ = didcomm_connection(&agent, &mut aries_transport).await;
}

#[tokio::test]
async fn test_mediate_grant() -> Result<()> {
    TestSetupAries.init();
    let agent = mediator::aries_agent::AgentMaker::new_demo_agent().await?;
    let mut aries_transport = AriesReqwest {
        response_queue: VecDeque::new(),
        client: reqwest::Client::new(),
    };
    let completed_connection = didcomm_connection(&agent, &mut aries_transport).await?;
    let our_verikey: VeriKey = completed_connection.pairwise_info().pw_vk.clone();
    let their_diddoc = completed_connection.their_did_doc();
    let message = MediatorCoordMsgEnum::MediateRequest;
    let message_bytes = serde_json::to_vec(&message)?;
    // info!("Message: {:?}", serde_json::to_string(&message).unwrap());
    // info!("Sending: {:?}", message_bytes);
    // let EncryptionEnvelope(packed) = agent.pack_didcomm(&message_bytes, &our_verikey,
    // &their_diddoc).await.unwrap(); let packed_val =
    // serde_json::from_slice::<serde_json::Value>(&packed).unwrap(); info!("Packed: {:?}",
    // serde_json::to_string(&packed_val).unwrap());
    agent
        .pack_and_send_didcomm(
            &message_bytes,
            &our_verikey,
            their_diddoc,
            &mut aries_transport,
        )
        .await
        .map_err(|err| GenericStringError { msg: err })?;
    // let k = aries_transport.push_aries_envelope(packed_val, their_diddoc).await.unwrap();
    // info!("Pushed: {:?}", k);
    let response = aries_transport.pop_aries_envelope()?;
    let unpacked_response = agent
        .unpack_didcomm(&serde_json::to_vec(&response).unwrap())
        .await
        .unwrap();
    if let MediatorCoordMsgEnum::MediateGrant(grant_data) =
        serde_json::from_str(&unpacked_response.message).unwrap()
    {
        info!("Grant Data {:?}", grant_data);
    } else if let MediatorCoordMsgEnum::MediateDeny(deny_data) =
        serde_json::from_str(&unpacked_response.message).unwrap()
    {
        info!("Deny Data {:?}", deny_data);
    } else {
        panic!(
            "Should get response that is of type Mediator Grant / Deny. Found {:?}",
            unpacked_response.message
        )
    };

    Ok(())
}

#[tokio::test]
async fn test_mediate_keylist_update_add() -> Result<()> {
    TestSetupAries.init();
    // ready agent, http client
    let agent = mediator::aries_agent::AgentMaker::new_demo_agent().await?;
    let mut aries_transport = AriesReqwest {
        response_queue: VecDeque::new(),
        client: reqwest::Client::new(),
    };
    // connection and parameters
    let completed_connection = didcomm_connection(&agent, &mut aries_transport).await?;
    let our_verikey: VeriKey = completed_connection.pairwise_info().pw_vk.clone();
    let their_diddoc = completed_connection.their_did_doc();
    // test feature
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
    agent
        .pack_and_send_didcomm(
            &message_bytes,
            &our_verikey,
            their_diddoc,
            &mut aries_transport,
        )
        .await
        .map_err(|err| GenericStringError { msg: err })?;
    // verify response
    let response = aries_transport.pop_aries_envelope()?;
    let unpacked_response = agent
        .unpack_didcomm(&serde_json::to_vec(&response).unwrap())
        .await
        .unwrap();
    if let MediatorCoordMsgEnum::KeylistUpdateResponse(update_response_data) =
        serde_json::from_str(&unpacked_response.message)?
    {
        info!("Received update response {:?}", update_response_data);
    } else {
        panic!(
            "Expected message of type KeylistUpdateResponse. Found {:?}",
            unpacked_response.message
        )
    }

    Ok(())
}

#[tokio::test]
async fn test_mediate_keylist_query() -> Result<()> {
    TestSetupAries.init();
    // ready agent, http client
    let agent = mediator::aries_agent::AgentMaker::new_demo_agent().await?;
    let mut aries_transport = AriesReqwest {
        response_queue: VecDeque::new(),
        client: reqwest::Client::new(),
    };
    // connection and parameters
    let completed_connection = didcomm_connection(&agent, &mut aries_transport).await?;
    let our_verikey: VeriKey = completed_connection.pairwise_info().pw_vk.clone();
    let their_diddoc = completed_connection.their_did_doc();
    // test feature part1: add key
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
    agent
        .pack_and_send_didcomm(
            &message_bytes,
            &our_verikey,
            their_diddoc,
            &mut aries_transport,
        )
        .await
        .map_err(|err| GenericStringError { msg: err })?;
    let response = aries_transport.pop_aries_envelope()?;
    let unpacked_response = agent
        .unpack_didcomm(&serde_json::to_vec(&response).unwrap())
        .await
        .unwrap();
    if let MediatorCoordMsgEnum::KeylistUpdateResponse(update_response_data) =
        serde_json::from_str(&unpacked_response.message)?
    {
        info!(
            "Recieved update response {:?}, proceeding to keylist query",
            update_response_data
        );
    } else {
        panic!(
            "Expected message of type KeylistUpdateResponse. Found {:?}",
            unpacked_response.message
        )
    }

    //test feature part2: list keys
    let message = MediatorCoordMsgEnum::KeylistQuery(KeylistQueryData {});
    info!("Sending {:?}", serde_json::to_string(&message).unwrap());
    let message_bytes = serde_json::to_vec(&message)?;
    agent
        .pack_and_send_didcomm(
            &message_bytes,
            &our_verikey,
            their_diddoc,
            &mut aries_transport,
        )
        .await
        .map_err(|err| GenericStringError { msg: err })?;
    // verify response
    let response = aries_transport.pop_aries_envelope()?;
    let unpacked_response = agent
        .unpack_didcomm(&serde_json::to_vec(&response).unwrap())
        .await
        .unwrap();
    if let MediatorCoordMsgEnum::Keylist(KeylistData { keys }) =
        serde_json::from_str(&unpacked_response.message)?
    {
        info!("Keylist mediator sent {:?}", keys)
    } else {
        panic!(
            "Expected message of type Keylist. Found {:?}",
            unpacked_response.message
        )
    }

    Ok(())
}

#[tokio::test]
async fn test_mediate_keylist_update_remove() -> Result<()> {
    TestSetupAries.init();
    // ready agent, http client
    let agent = mediator::aries_agent::AgentMaker::new_demo_agent().await?;
    let mut aries_transport = AriesReqwest {
        response_queue: VecDeque::new(),
        client: reqwest::Client::new(),
    };
    // connection and parameters
    let completed_connection = didcomm_connection(&agent, &mut aries_transport).await?;
    let our_verikey: VeriKey = completed_connection.pairwise_info().pw_vk.clone();
    let their_diddoc = completed_connection.their_did_doc();
    // test feature part1: add key
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
    info!("Sending {:?}", serde_json::to_string(&message).unwrap());
    let message_bytes = serde_json::to_vec(&message)?;
    agent
        .pack_and_send_didcomm(
            &message_bytes,
            &our_verikey,
            their_diddoc,
            &mut aries_transport,
        )
        .await
        .map_err(|err| GenericStringError { msg: err })?;
    let response = aries_transport.pop_aries_envelope()?;
    let unpacked_response = agent
        .unpack_didcomm(&serde_json::to_vec(&response).unwrap())
        .await
        .unwrap();
    if let MediatorCoordMsgEnum::KeylistUpdateResponse(update_response_data) =
        serde_json::from_str(&unpacked_response.message)?
    {
        info!(
            "Recieved update response {:?}, proceeding to delete",
            update_response_data
        );
    } else {
        panic!(
            "Expected message of type KeylistUpdateResponse. Found {:?}",
            unpacked_response.message
        )
    }
    // test feature part2: delete key
    let message = MediatorCoordMsgEnum::KeylistUpdateRequest(KeylistUpdateRequestData {
        updates: vec![KeylistUpdateItem {
            recipient_key: new_vk,
            action: KeylistUpdateItemAction::Remove,
            result: None,
        }],
    });
    info!("Sending {:?}", serde_json::to_string(&message).unwrap());
    let message_bytes = serde_json::to_vec(&message)?;
    agent
        .pack_and_send_didcomm(
            &message_bytes,
            &our_verikey,
            their_diddoc,
            &mut aries_transport,
        )
        .await
        .map_err(|err| GenericStringError { msg: err })?;
    let response = aries_transport.pop_aries_envelope()?;
    let unpacked_response = agent
        .unpack_didcomm(&serde_json::to_vec(&response).unwrap())
        .await
        .unwrap();
    if let MediatorCoordMsgEnum::KeylistUpdateResponse(update_response_data) =
        serde_json::from_str(&unpacked_response.message)?
    {
        info!("Recieved update response {:?}", update_response_data);
    } else {
        panic!(
            "Expected message of type KeylistUpdateResponse. Found {:?}",
            unpacked_response.message
        )
    }
    Ok(())
}
