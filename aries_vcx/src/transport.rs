use async_trait::async_trait;

use crate::errors::error::VcxResult;

/// Trait used for implementing a mechanism to send a message, used by
/// [`crate::protocols::connection::Connection`].
#[async_trait]
pub trait Transport: Send + Sync {
    async fn send_message(&self, msg: Vec<u8>, service_endpoint: &str) -> VcxResult<()>;
}

// While in many cases the auto-dereferencing does the trick,
// this implementation aids in using things such as a trait object
// when a generic parameter is expected.
#[async_trait]
impl<T> Transport for &T
where
    T: Transport + ?Sized,
{
    async fn send_message(&self, msg: Vec<u8>, service_endpoint: &str) -> VcxResult<()> {
        self.send_message(msg, service_endpoint).await
    }
}
