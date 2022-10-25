use futures::future::BoxFuture;

use crate::error::VcxResult;
use messages::a2a::A2AMessage;

pub mod common;
pub mod connection;
pub mod issuance;
pub mod oob;
pub mod proof_presentation;
pub mod trustping;

pub type SendClosure = Box<dyn Fn(A2AMessage) -> BoxFuture<'static, VcxResult<()>> + Send + Sync>;
