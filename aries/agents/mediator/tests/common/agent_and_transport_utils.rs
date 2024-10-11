use aries_vcx::{
    protocols::{
        connection::invitee::{states::completed::Completed, InviteeConnection},
        oob::oob_invitation_to_legacy_did_doc,
    },
    utils::encryption_envelope::EncryptionEnvelope,
};
use aries_vcx_wallet::wallet::{askar::AskarWallet, base_wallet::BaseWallet};
use diddoc_legacy::aries::diddoc::AriesDidDoc;
use mediator::{
    aries_agent::{client::transports::AriesTransport, Agent},
    persistence::MediatorPersistence,
    utils::{structs::VerKey, GenericStringError},
};
use messages::{
    msg_fields::protocols::{
        coordinate_mediation::{
            keylist_update::{KeylistUpdateItem, KeylistUpdateItemAction},
            CoordinateMediation, KeylistUpdate, KeylistUpdateContent, MediateGrantContent,
            MediateRequest, MediateRequestContent,
        },
        out_of_band::invitation::Invitation as OOBInvitation,
    },
    AriesMessage,
};
use test_utils::mockdata::mock_ledger::MockLedger;

use super::prelude::*;

const ENDPOINT_ROOT: &str = "http://localhost:8005";

pub async fn didcomm_connection(
    agent: &Agent<impl BaseWallet, impl MediatorPersistence>,
    aries_transport: &mut impl AriesTransport,
) -> Result<InviteeConnection<Completed>> {
    let client = reqwest::Client::new();
    let base: Url = ENDPOINT_ROOT.parse().unwrap();
    let endpoint_register = base.join("invitation").unwrap();

    let oobi: OOBInvitation = client
        .get(endpoint_register)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    info!("Got invitation {:?}", oobi);

    let state: InviteeConnection<Completed> =
        agent.establish_connection(oobi, aries_transport).await?;

    Ok(state)
}

/// Returns agent, aries transport for agent, agent's verkey, and mediator's diddoc.
pub async fn gen_mediator_connected_agent() -> Result<(
    Agent<impl BaseWallet, impl MediatorPersistence>,
    impl AriesTransport,
    VerKey,
    AriesDidDoc,
)> {
    let agent = mediator::aries_agent::AgentBuilder::<AskarWallet>::new_demo_agent().await?;
    let mut aries_transport = reqwest::Client::new();
    let completed_connection = didcomm_connection(&agent, &mut aries_transport).await?;
    let our_verkey: VerKey = completed_connection.pairwise_info().pw_vk.clone();
    let their_diddoc = completed_connection.their_did_doc().clone();
    Ok((agent, aries_transport, our_verkey, their_diddoc))
}

/// Sends message over didcomm connection and returns unpacked response message
pub async fn send_message_and_pop_response_message(
    message_bytes: &[u8],
    agent: &Agent<impl BaseWallet, impl MediatorPersistence>,
    aries_transport: &mut impl AriesTransport,
    our_verkey: &VerKey,
    their_diddoc: &AriesDidDoc,
) -> Result<String> {
    // Wrap message in encrypted envelope
    let EncryptionEnvelope(packed_message) = agent
        .pack_didcomm(message_bytes, our_verkey, their_diddoc)
        .await
        .map_err(|e| GenericStringError { msg: e })?;
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
/// Register recipient keys with mediator
pub async fn gen_and_register_recipient_key(
    agent: &mut Agent<impl BaseWallet, impl MediatorPersistence>,
    agent_aries_transport: &mut impl AriesTransport,
    agent_verkey: &VerKey,
    mediator_diddoc: &AriesDidDoc,
) -> Result<(VerKey, AriesDidDoc)> {
    let agent_invite: OOBInvitation = agent
        .get_oob_invite()
        .map_err(|e| GenericStringError { msg: e })?;
    let mock_ledger = MockLedger {};
    let agent_diddoc = oob_invitation_to_legacy_did_doc(&mock_ledger, &agent_invite)
        .await
        .unwrap();
    let agent_recipient_key = agent_diddoc
        .recipient_keys()
        .unwrap()
        .first()
        .unwrap()
        .clone();
    // register recipient key with mediator
    let key_update = KeylistUpdate::builder()
        .content(
            KeylistUpdateContent::builder()
                .updates(vec![KeylistUpdateItem {
                    recipient_key: agent_recipient_key.clone(),
                    action: KeylistUpdateItemAction::Add,
                }])
                .build(),
        )
        .id("register-key-with-mediator".to_owned())
        .build();
    let message = AriesMessage::CoordinateMediation(CoordinateMediation::KeylistUpdate(key_update));
    info!("Sending {:?}", serde_json::to_string(&message).unwrap());
    let message_bytes = serde_json::to_vec(&message)?;
    let _response_message = send_message_and_pop_response_message(
        &message_bytes,
        agent,
        agent_aries_transport,
        agent_verkey,
        mediator_diddoc,
    )
    .await?;
    Ok((agent_recipient_key, agent_diddoc))
}

pub async fn get_mediator_grant_data(
    agent: &Agent<impl BaseWallet, impl MediatorPersistence>,
    agent_aries_transport: &mut impl AriesTransport,
    agent_verkey: &VerKey,
    mediator_diddoc: &AriesDidDoc,
) -> MediateGrantContent {
    // prepare request message
    let message = AriesMessage::CoordinateMediation(CoordinateMediation::MediateRequest(
        MediateRequest::builder()
            .content(MediateRequestContent::default())
            .id("mediate-requets".to_owned())
            .build(),
    ));
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
    if let AriesMessage::CoordinateMediation(CoordinateMediation::MediateGrant(grant_data)) =
        serde_json::from_str(&response_message).unwrap()
    {
        info!("Grant Data {:?}", grant_data);
        grant_data.content
    } else {
        panic!(
            "Should get response that is of type Mediator Grant. Found {:?}",
            response_message
        )
    }
}
