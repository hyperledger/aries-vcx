mod common;

use aries_vcx::utils::encryption_envelope::EncryptionEnvelope;
use aries_vcx_wallet::wallet::askar::AskarWallet;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use mediator::aries_agent::client::transports::AriesTransport;
use messages::{
    decorators::attachment::AttachmentType,
    msg_fields::protocols::{
        basic_message::{BasicMessage, BasicMessageContent, BasicMessageDecorators},
        pickup::{
            DeliveryRequest, DeliveryRequestContent, DeliveryRequestDecorators, Pickup,
            StatusRequest, StatusRequestContent, StatusRequestDecorators,
        },
    },
    AriesMessage,
};

use crate::common::{
    agent_and_transport_utils::{
        gen_and_register_recipient_key, gen_mediator_connected_agent, get_mediator_grant_data,
        send_message_and_pop_response_message,
    },
    prelude::*,
    test_setup::setup_env_logging,
};

static LOGGING_INIT: std::sync::Once = std::sync::Once::new();

async fn forward_basic_anoncrypt_message(
    agent_diddoc: &AriesDidDoc,
    message_text: &str,
) -> Result<()> {
    // Prepare forwarding agent
    let agent_f = mediator::aries_agent::AgentBuilder::<AskarWallet>::new_demo_agent().await?;
    // Prepare forwarding agent transport
    let mut agent_f_aries_transport = reqwest::Client::new();
    // Prepare message and wrap into anoncrypt forward message
    let message: BasicMessage = BasicMessage::builder()
        .content(
            BasicMessageContent::builder()
                .content(message_text.to_string())
                .sent_time(chrono::DateTime::default())
                .build(),
        )
        .decorators(BasicMessageDecorators::default())
        .id("JustHello".to_string())
        .build();

    let EncryptionEnvelope(packed_message) = EncryptionEnvelope::create_from_legacy(
        agent_f.get_wallet_ref().as_ref(),
        &serde_json::to_vec(&message)?,
        None,
        agent_diddoc,
    )
    .await?;
    // Send forward message to provided endpoint
    let packed_json = serde_json::from_slice(&packed_message)?;
    info!("Sending anoncrypt packed message{}", packed_json);
    let response_envelope = agent_f_aries_transport
        .send_aries_envelope(packed_json, agent_diddoc)
        .await?;
    info!("Response of forward{:?}", response_envelope);
    Ok(())
}

#[tokio::test]
async fn test_pickup_flow() -> Result<()> {
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
    // forward some messages.
    forward_basic_anoncrypt_message(&agent_diddoc, "Hi, from AgentF").await?;
    forward_basic_anoncrypt_message(&agent_diddoc, "Hi again, from AgentF").await?;
    // Pickup flow
    // // Status
    let pickup_status_req = Pickup::StatusRequest(
        StatusRequest::builder()
            .content(StatusRequestContent::builder().build())
            .decorators(StatusRequestDecorators::default())
            .id("request-status".to_owned())
            .build(),
    );
    let aries_message = AriesMessage::Pickup(pickup_status_req);
    let message_bytes = serde_json::to_vec(&aries_message)?;
    // send message and get response
    let response_message = send_message_and_pop_response_message(
        &message_bytes,
        &agent,
        &mut agent_aries_transport,
        &agent_verkey,
        &mediator_diddoc,
    )
    .await?;
    // Verify expected
    if let AriesMessage::Pickup(Pickup::Status(status)) = serde_json::from_str(&response_message)? {
        info!("Received status as expected {:?}", status);
        assert_eq!(status.content.message_count, 2)
    } else {
        panic!(
            "Expected status with message count = 2, received {:?}",
            response_message
        )
    }
    // // Delivery
    let pickup_delivery_req = Pickup::DeliveryRequest(
        DeliveryRequest::builder()
            .content(DeliveryRequestContent::builder().limit(10).build())
            .decorators(DeliveryRequestDecorators::builder().build())
            .id("request-delivery".to_owned())
            .build(),
    );
    let aries_message = AriesMessage::Pickup(pickup_delivery_req);
    let message_bytes = serde_json::to_vec(&aries_message)?;
    // send message and get response
    let response_message = send_message_and_pop_response_message(
        &message_bytes,
        &agent,
        &mut agent_aries_transport,
        &agent_verkey,
        &mediator_diddoc,
    )
    .await?;
    // Verify expected
    let delivery = if let AriesMessage::Pickup(Pickup::Delivery(delivery)) =
        serde_json::from_str(&response_message)?
    {
        info!("Received delivery as expected {:?}", delivery);
        assert_eq!(delivery.content.attach.len(), 2);
        delivery
    } else {
        panic!(
            "Expected delivery with num_attachment = 2, received {:?}",
            response_message
        )
    };
    // verify valid attachment
    if let AttachmentType::Base64(base64message) =
        &delivery.content.attach.first().unwrap().data.content
    {
        let encrypted_message_bytes = URL_SAFE_NO_PAD.decode(base64message)?;
        info!(
            "Decoding attachment to packed_message {}",
            String::from_utf8(message_bytes.clone())?
        );
        let unpack = agent.unpack_didcomm(&encrypted_message_bytes).await;
        info!("Decoded attachment 1 {:?}", unpack);
    }

    Ok(())
}
