use messages_macros::Message;
use serde::{Deserialize, Serialize};

use crate::{decorators::{Thread, Timing}, message_type::message_family::out_of_band::OutOfBandV1_1, protocols::traits::ConcreteMessage};

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "OutOfBandV1_1::HandshakeReuseAccepted")]
pub struct HandshakeReuseAccepted {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "~thread")]
    pub thread: Thread,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
