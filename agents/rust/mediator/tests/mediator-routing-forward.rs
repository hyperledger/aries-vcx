mod common;
use std::collections::VecDeque;

use aries_vcx::utils::encryption_envelope::EncryptionEnvelope;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use mediation::{
    didcomm_types::mediator_coord_structs::{
        KeylistUpdateItem, KeylistUpdateItemAction, KeylistUpdateRequestData, MediateGrantData,
        MediatorCoordMsgEnum,
    },
    storage::MediatorPersistence,
};
use mediator::{
    aries_agent::{
        transports::{AriesReqwest, AriesTransport},
        utils::oob2did,
        Agent,
    },
    utils::{structs::VeriKey, GenericStringError},
};
use messages::msg_fields::protocols::{
    basic_message::{BasicMessage, BasicMessageContent, BasicMessageDecorators},
    out_of_band::invitation::Invitation as OOBInvitation,
};

use crate::common::{
    agent_and_transport_utils::{
        gen_mediator_connected_agent, send_message_and_pop_response_message,
    },
    prelude::*,
    test_setup::OneTimeInit,
};

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

async fn get_mediator_grant_data(
    agent: &Agent<impl BaseWallet + 'static, impl MediatorPersistence>,
    agent_aries_transport: &mut impl AriesTransport,
    agent_verikey: &VeriKey,
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
        agent_verikey,
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

#[tokio::test]
async fn test_forward_flow() -> Result<()> {
    TestSetupAries.init();
    // prepare receiver connection parameters
    let (mut agent, mut agent_aries_transport, agent_verikey, mediator_diddoc) =
        gen_mediator_connected_agent().await?;
    // setup receiver routing
    let grant_data = get_mediator_grant_data(
        &agent,
        &mut agent_aries_transport,
        &agent_verikey,
        &mediator_diddoc,
    )
    .await;
    agent
        .init_service(grant_data.routing_keys, grant_data.endpoint.parse()?)
        .await?;
    let agent_invite: OOBInvitation = agent
        .get_oob_invite()
        .map_err(|e| GenericStringError { msg: e.to_string() })?;
    let agent_diddoc = oob2did(agent_invite);
    let agent_recipient_key = agent_diddoc
        .recipient_keys()
        .unwrap()
        .first()
        .unwrap()
        .clone();
    // register recipient key with mediator
    let message = MediatorCoordMsgEnum::KeylistUpdateRequest(KeylistUpdateRequestData {
        updates: vec![KeylistUpdateItem {
            recipient_key: agent_recipient_key,
            action: KeylistUpdateItemAction::Add,
            result: None,
        }],
    });
    info!("Sending {:?}", serde_json::to_string(&message).unwrap());
    let message_bytes = serde_json::to_vec(&message)?;
    let _response_message = send_message_and_pop_response_message(
        &message_bytes,
        &agent,
        &mut agent_aries_transport,
        &agent_verikey,
        &mediator_diddoc,
    )
    .await?;
    // Prepare forwarding agent transport
    let mut agent_f_aries_transport = AriesReqwest {
        response_queue: VecDeque::new(),
        client: reqwest::Client::new(),
    };
    // Prepare message and wrap into anoncrypt forward message
    let message: BasicMessage = BasicMessage::builder()
        .content(
            BasicMessageContent::builder()
                .content("Hi, for AgentF".to_string())
                .sent_time(chrono::DateTime::default())
                .build(),
        )
        .decorators(BasicMessageDecorators::default())
        .id("JustHello".to_string())
        .build();

    let EncryptionEnvelope(packed_message) = EncryptionEnvelope::create(
        &*agent.get_wallet_ref(),
        &serde_json::to_vec(&message)?,
        None,
        &agent_diddoc,
    )
    .await?;
    // Send forward message to provided endpoint
    let packed_json = serde_json::from_slice(&packed_message)?;
    info!("Sending anoncrypt packed message{}", packed_json);
    agent_f_aries_transport
        .push_aries_envelope(packed_json, &agent_diddoc)
        .await?;
    info!(
        "Response of forward{:?}",
        agent_f_aries_transport.pop_aries_envelope()?
    );

    Ok(())
}
