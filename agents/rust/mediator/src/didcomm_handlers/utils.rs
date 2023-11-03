pub mod prelude {
    pub use aries_vcx::utils::encryption_envelope::EncryptionEnvelope;
    pub use aries_vcx_core::wallet::base_wallet::BaseWallet;
    pub use mediation::storage::MediatorPersistence;

    pub use crate::{aries_agent::ArcAgent, utils::prelude::*};
}
