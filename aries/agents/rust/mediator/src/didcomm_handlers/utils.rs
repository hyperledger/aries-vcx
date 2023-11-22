pub mod prelude {
    pub use std::sync::Arc;

    pub use aries_vcx::utils::encryption_envelope::EncryptionEnvelope;
    pub use aries_vcx_core::wallet::base_wallet::BaseWallet;

    pub use crate::{aries_agent::ArcAgent, persistence::MediatorPersistence, utils::prelude::*};
}
