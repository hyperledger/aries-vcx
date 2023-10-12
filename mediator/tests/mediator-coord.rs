mod common;
use std::collections::VecDeque;

use aries_vcx::{
    handlers::util::AnyInvitation,
    protocols::{
        connection::invitee::{
            states::{
                completed::Completed, initial::Initial as ClientInit,
                requested::Requested as ClientRequestSent,
            },
            InviteeConnection,
        },
        mediated_connection::pairwise_info::PairwiseInfo,
    },
    utils::{encryption_envelope::EncryptionEnvelope, mockdata::profile::mock_ledger::MockLedger},
};
use aries_vcx_core::wallet::{base_wallet::BaseWallet, indy::IndySdkWallet};
use common::{prelude::*, test_setup::OneTimeInit};
use mediator::{
    agent::{
        transports::{AriesReqwest, AriesTransport},
        Agent,
    },
    utils::{structs::VeriKey, GenericStringError},
};
use messages::{
    msg_fields::protocols::{
        connection::{request::Request, response::Response, Connection},
        out_of_band::invitation::Invitation as OOBInvitation,
    },
    AriesMessage,
};
use reqwest::header::ACCEPT;
use sqlx::MySqlPool;
use xum_test_server::{
    didcomm_types::mediator_coord_structs::MediatorCoordMsgEnum, storage::MediatorPersistence,
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
    let agent = mediator::agent::AgentMaker::new_demo_agent().await.unwrap();
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
    let agent = mediator::agent::AgentMaker::new_demo_agent().await?;
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
    // let EncryptionEnvelope(packed) = agent.pack_didcomm(&message_bytes, &our_verikey, &their_diddoc).await.unwrap();
    // let packed_val =  serde_json::from_slice::<serde_json::Value>(&packed).unwrap();
    // info!("Packed: {:?}", serde_json::to_string(&packed_val).unwrap());
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
    let unpacked_response = agent.unpack_didcomm(&serde_json::to_vec(&response).unwrap()).await.unwrap();
    info!("response message: {:?}", unpacked_response.message);

    Ok(())
}
