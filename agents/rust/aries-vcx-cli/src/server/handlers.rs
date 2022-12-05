use std::sync::RwLockWriteGuard;

use anyhow::anyhow;

use aries_vcx_agent::aries_vcx::utils::encryption_envelope::EncryptionEnvelope;

use crate::agent::CliAriesAgent;

pub async fn handle_message(mut agent: RwLockWriteGuard<'_, CliAriesAgent>, payload: Vec<u8>) -> anyhow::Result<()> {
    let (message, _) = EncryptionEnvelope::anon_unpack(&agent.agent().profile().inject_wallet(), payload)
        .await
        .map_err(|err| anyhow!("Failed to unpack message: {}", err))?;
    info!("Received message: {:?}", message);
    agent.push_message(message);
    Ok(())
}
