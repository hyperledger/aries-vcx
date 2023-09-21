use diddoc_legacy::aries::diddoc::AriesDidDoc;
use futures::future::BoxFuture;
use messages::AriesMessage;

use crate::errors::error::VcxResult;

pub mod common;
pub mod connection;
pub mod issuance;
pub mod mediated_connection;
pub mod oob;
pub mod proof_presentation;
pub mod revocation_notification;
pub mod trustping;

pub type SendClosure =
    Box<dyn FnOnce(AriesMessage) -> BoxFuture<'static, VcxResult<()>> + Send + Sync>;
pub type SendClosureConnection = Box<
    dyn FnOnce(AriesMessage, String, AriesDidDoc) -> BoxFuture<'static, VcxResult<()>>
        + Send
        + Sync,
>;
