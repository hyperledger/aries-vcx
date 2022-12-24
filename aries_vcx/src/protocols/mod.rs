use futures::future::BoxFuture;
use messages::a2a::A2AMessage;
use messages::diddoc::aries::diddoc::AriesDidDoc;

use crate::errors::error::VcxResult;

pub mod connection;
pub mod issuance;
pub mod oob;
pub mod proof_presentation;
pub mod trustping;
pub mod common;
pub mod revocation_notification;

pub type SendClosure = Box<dyn FnOnce(A2AMessage) -> BoxFuture<'static, VcxResult<()>> + Send + Sync>;
pub type SendClosureConnection = Box<dyn FnOnce(A2AMessage, String, AriesDidDoc) -> BoxFuture<'static, VcxResult<()>> + Send + Sync>;
