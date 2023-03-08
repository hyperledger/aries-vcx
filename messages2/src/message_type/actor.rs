use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
#[serde(untagged)]
pub enum Actor {
    Inviter,
    Invitee,
    Issuer,
    Holder,
    Prover,
    Verifier,
    Sender,
    Receiver,
    Requester,
    Responder,
    Notified,
    Notifier,
    Mediator,
    #[serde(serialize_with = "String::serialize")]
    #[serde(deserialize_with = "String::deserialize")]
    Custom(String),
}
