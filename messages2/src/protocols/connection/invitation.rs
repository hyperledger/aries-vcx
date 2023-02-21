use messages_macros::Message;
use serde::{Deserialize, Serialize};

use crate::{message_type::message_family::connection::ConnectionV1_0};

use crate::protocols::traits::ConcreteMessage;

#[derive(Clone, Debug, Deserialize, Serialize, Message)]
#[message(kind = "ConnectionV1_0::Invitation")]
pub struct Invitation {
    #[serde(rename = "@id")]
    pub id: String,
    pub label: String,
    pub did: String,
}
