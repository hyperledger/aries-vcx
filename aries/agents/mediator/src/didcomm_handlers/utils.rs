pub mod prelude {
    pub use aries_vcx::utils::encryption_envelope::EncryptionEnvelope;

    pub use crate::{aries_agent::ArcAgent, persistence::MediatorPersistence, utils::prelude::*};
}
