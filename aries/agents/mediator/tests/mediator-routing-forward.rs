mod common;

use aries_vcx::utils::encryption_envelope::EncryptionEnvelope;
use mediator::aries_agent::client::transports::AriesTransport;
use messages::msg_fields::protocols::basic_message::{
    BasicMessage, BasicMessageContent, BasicMessageDecorators,
};

use crate::common::{
    agent_and_transport_utils::{
        gen_and_register_recipient_key, gen_mediator_connected_agent, get_mediator_grant_data,
    },
    prelude::*,
    test_setup::setup_env_logging,
};

static LOGGING_INIT: std::sync::Once = std::sync::Once::new();

#[tokio::test]
async fn test_forward_flow() -> Result<()> {
    LOGGING_INIT.call_once(setup_env_logging);
    // prepare receiver connection parameters
    let (mut agent, mut agent_aries_transport, agent_verkey, mediator_diddoc) =
        gen_mediator_connected_agent().await?;
    // setup receiver routing
    let grant_data = get_mediator_grant_data(
        &agent,
        &mut agent_aries_transport,
        &agent_verkey,
        &mediator_diddoc,
    )
    .await;
    agent
        .init_service(grant_data.routing_keys, grant_data.endpoint.parse()?)
        .await?;
    // register recipient key with mediator
    let (_agent_recipient_key, agent_diddoc) = gen_and_register_recipient_key(
        &mut agent,
        &mut agent_aries_transport,
        &agent_verkey,
        &mediator_diddoc,
    )
    .await?;
    // Prepare forwarding agent transport
    let mut agent_f_aries_transport = reqwest::Client::new();
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
    info!(
        "Prepared message {:?}, proceeding to anoncrypt wrap",
        serde_json::to_string(&message).unwrap()
    );
    let EncryptionEnvelope(packed_message) = EncryptionEnvelope::create_from_legacy(
        agent.get_wallet_ref().as_ref(),
        &serde_json::to_vec(&message)?,
        None,
        &agent_diddoc,
    )
    .await?;
    // Send forward message to provided endpoint
    let packed_json = serde_json::from_slice(&packed_message)?;
    let response = agent_f_aries_transport
        .send_aries_envelope(packed_json, &agent_diddoc)
        .await?;
    info!("Response of forward{:?}", response);

    Ok(())
}
