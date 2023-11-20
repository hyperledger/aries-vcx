use diddoc_legacy::aries::diddoc::AriesDidDoc;
use futures::future::BoxFuture;
use messages::AriesMessage;

use crate::errors::error::VcxResult;

pub mod common;
pub mod connection;
pub mod did_exchange;
pub mod issuance;
pub mod mediated_connection;
pub mod oob;
pub mod proof_presentation;
pub mod revocation_notification;
pub mod trustping;

pub type SendClosure<'a> =
    Box<dyn FnOnce(AriesMessage) -> BoxFuture<'a, VcxResult<()>> + Send + Sync + 'a>;
pub type SendClosureConnection<'a> = Box<
    dyn FnOnce(AriesMessage, String, AriesDidDoc) -> BoxFuture<'a, VcxResult<()>>
        + Send
        + Sync
        + 'a,
>;
