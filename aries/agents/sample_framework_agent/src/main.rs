#![deny(clippy::unwrap_used)]

use std::{error::Error, thread};

use aries_framework_vcx::{
    aries_vcx::handlers::out_of_band::receiver::OutOfBandReceiver,
    connection_service::ConnectionServiceConfig,
    framework::{
        AriesFrameworkVCX, EventEmitter, FrameworkConfig, DEFAULT_ASKAR_KEY_METHOD,
        DEFAULT_WALLET_PROFILE, IN_MEMORY_DB_URL,
    },
    AskarWalletConfig, Url, VCXFrameworkError,
};
use log::{debug, info};

#[tokio::main]
async fn main() -> Result<(), VCXFrameworkError> {
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
    debug!(
        "Created Out Of Band Invitation: {:?}",
        oob_invitation.invitation_to_url("http://localhost:8010")
    );

    let invitation_receiver = OutOfBandReceiver::create_from_url_encoded_oob(
        &oob_invitation
            .invitation_to_url("https://present-bengal-logical.ngrok-free.app?oob=eyJAdHlwZSI6ICJodHRwczovL2RpZGNvbW0ub3JnL291dC1vZi1iYW5kLzEuMS9pbnZpdGF0aW9uIiwgIkBpZCI6ICI1ZGRlMTZmNy1hNjY4LTQ3ZmItOTE4OC01NGU4Mjg5YmU1MmMiLCAibGFiZWwiOiAiOS0xMi0yNCIsICJoYW5kc2hha2VfcHJvdG9jb2xzIjogWyJodHRwczovL2RpZGNvbW0ub3JnL2RpZGV4Y2hhbmdlLzEuMCJdLCAiYWNjZXB0IjogWyJkaWRjb21tL2FpcDEiLCAiZGlkY29tbS9haXAyO2Vudj1yZmMxOSJdLCAic2VydmljZXMiOiBbImRpZDpwZWVyOjR6UW1QeWpiUmlTV0xYVGppTWJwclJuMzZ6NVVLRFB1WWd4eU03dkNQYm03R2ZHTjp6MllXS1VEZ2hnQWtVR2FIWU5WSE1EQndjUG5YVEVCcXpFclp6MW5nNEdGcWY2c2hadXlLVHNiN2FmMWduWnZ2UUw4cUdvd3JpWFM4U25pNzJobmN3Q3hvUFpCSk1IOTNVb0VvdktvR0hNMlZieVNBQ1dvUlBHWUg0NG5ic3ZxWndhUkRkQmtkbUpoaW1IcGdtTlYydFlMM2J4MXRZSnBNUnRSU1liRTd3M2JyWXlrZHlxdkxCdXR4VXkyNXd4VkR2Wm5oZHJaM0N4TVFzaHJpWGhvYjlLTDFjWnV2QTZYTEpUTThZaG9FaFJoQmF1Yno5VmFCY01zS3hLaHJNQ2o2aHF4cFNrWEZ0dTNYbWlMYmVnQkVBR0pGbjRmcXNqdURtU3hFOVFXRTlyU1MyMVZNbUhra3c4aW9jYmk4MzhUR0d4QXdjczlzMndSQUM4MXhQTGY5azJQSjVCemk4c2NIWHpGMm9VVnR0YjFtSkVpOVFlVG92eFd4QjgyYXNMWVk3Q0NQRGNtV21lVjVuU0FDWG1pNGlmcE1WdjFpN2Q5dmpRdkRmWlBzWDlvc1g4MnE3eE5HbVdVQWFKSzFVbWVkOERYdVZjNFc3cGZnbXhHNUo5NDRBWkNhUTZRRVdmaWdjdllZYUZmM3pNUWNTaWZ3VzdZODJSeENTdW9nWVVpVXVUSkFGeEZLTTV5a0Q0ZjVyUDVWSGE4QkpIZVdFa3dSZVp6UExuMlBrM0NMblMzcENNZHRwUFpianY2Vk5NdzhXM3ZQb2VlMnd3VlhBN0tvZFZtQlVMUnE0N2RaZTVBNzZ6REtadU1iYnhwIl19")?
            .to_string(),
    )?;

    let connection_record = framework
        .connection_service
        .lock()
        .expect("unpoisoned mutex")
        .request_connection(invitation_receiver)
        .await?;

    Ok(())
}
