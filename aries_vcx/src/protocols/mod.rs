use futures::future::BoxFuture;

use crate::error::VcxResult;
use messages::{a2a::A2AMessage, did_doc::DidDoc};

pub mod connection;
pub mod issuance;
pub mod oob;
pub mod proof_presentation;
pub mod trustping;
pub mod common;
pub mod revocation_notification;

pub type SendClosure = Box<dyn FnOnce(A2AMessage) -> BoxFuture<'static, VcxResult<()>> + Send + Sync>;
pub type SendClosureConnection = Box<dyn FnOnce(A2AMessage, String, DidDoc) -> BoxFuture<'static, VcxResult<()>> + Send + Sync>;
