#![deny(clippy::unwrap_used)]

use std::{error::Error, thread};

use aries_framework_vcx::{
    connection_service::ConnectionServiceConfig,
    framework::{
        AriesFrameworkVCX, EventEmitter, FrameworkConfig, DEFAULT_ASKAR_KEY_METHOD,
        DEFAULT_WALLET_PROFILE, IN_MEMORY_DB_URL,
    },
    AskarWalletConfig, Url,
};
use log::{debug, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    info!("Sample Agent Init");

    let host = "localhost";
    let port = 8000;
    let agent_endpoint = Url::parse(&format!("http://{}:{}", host, port))?;

    let framework_config = FrameworkConfig {
        wallet_config: AskarWalletConfig {
            db_url: IN_MEMORY_DB_URL.to_string(),
            key_method: DEFAULT_ASKAR_KEY_METHOD,
            pass_key: "sample_pass_key".to_string(),
            profile: DEFAULT_WALLET_PROFILE.to_string(),
        },
        connection_service_config: ConnectionServiceConfig::default(),
        agent_endpoint,
        agent_label: "Sample Aries Framework VCX Agent".to_string(),
    };

    let framework = AriesFrameworkVCX::initialize(framework_config).await?;

    let rx = framework
        .invitation_service
        .lock()
        .expect("an unpoisoned mutex")
        .register_event_receiver();
    thread::spawn(move || {
        let received = rx.recv().expect("Expected a valid InvitationEvent");
        debug!("Received OutOfBandEvent {:?}", received);
    });

    let oob_invitation = framework
        .invitation_service
        .lock()
        .expect("an unpoisoned mutex")
        .create_invitation()
        .await?;
    debug!("Created Out Of Band Invitation: {:?}", oob_invitation);

    let connection_record = framework
        .connection_service
        .lock()
        .expect("unpoisoned mutex")
        .handle_request_and_await("1234567890")
        .await;

    Ok(())
}
