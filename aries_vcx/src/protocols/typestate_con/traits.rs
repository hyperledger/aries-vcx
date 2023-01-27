use async_trait::async_trait;
use messages::diddoc::aries::diddoc::AriesDidDoc;

use crate::errors::error::VcxResult;

/// Trait used for implementing common [`super::Connection`] behavior based
/// on states implementing it.
pub trait TheirDidDoc {
    fn their_did_doc(&self) -> &AriesDidDoc;
}

/// Trait used for implementing a mechanism to send a message, used by [`super::Connection`].
#[async_trait]
pub trait Transport {
    async fn send_message(&self, msg: Vec<u8>, service_endpoint: &str) -> VcxResult<()>;
}
