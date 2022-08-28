use futures::future::BoxFuture;

use crate::error::VcxResult;
use crate::messages::a2a::A2AMessage;

pub mod connection;
pub mod issuance;
pub mod proof_presentation;
pub mod oob;
pub mod trustping;

pub type SendClosure = Box<dyn Fn(A2AMessage) -> BoxFuture<'static, VcxResult<()>> + Send + Sync>;
