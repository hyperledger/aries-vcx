use serde::{Deserialize, Serialize};

/// The roles an agent can have in a protocol.
/// These are mainly for use in the [discover features](https://github.com/hyperledger/aries-rfcs/blob/main/features/0031-discover-features/README.md) protocol.
#[derive(Clone, Deserialize, Debug, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
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
}
