use serde::{Deserialize, Serialize, Serializer};

/// The roles an agent can have in a protocol.
/// These are mainly for use in the [discover features](https://github.com/hyperledger/aries-rfcs/blob/main/features/0031-discover-features/README.md) protocol.
//
// Unfortunately, we can't really derive a lot of things here.
// Since it is plausible to expect other agent implementations to use other protocols too, we need the
// catch-all [`Other`] variant, and that doesn't play well with `serde` or `strum` crates yet.
//
// I opened [this](https://github.com/Peternator7/strum/issues/258) issue since it shouldn't be hard to implement it in `strum` and that would make this easier to write & maintain.
#[derive(Clone, Deserialize, Debug)]
#[serde(from = "&str")]
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
    Other(String),
}

impl Actor {
    const INVITER: &str = "inviter";
    const INVITEE: &str = "invitee";
    const ISSUER: &str = "issuer";
    const HOLDER: &str = "holder";
    const PROVER: &str = "prover";
    const VERIFIER: &str = "verifier";
    const SENDER: &str = "sender";
    const RECEIVER: &str = "receiver";
    const REQUESTER: &str = "requester";
    const RESPONDER: &str = "responder";
    const NOTIFIED: &str = "notified";
    const NOTIFIER: &str = "notifier";
    const MEDIATOR: &str = "mediator";
}

impl AsRef<str> for Actor {
    fn as_ref(&self) -> &str {
        match self {
            Self::Inviter => Self::INVITER,
            Self::Invitee => Self::INVITEE,
            Self::Issuer => Self::ISSUER,
            Self::Holder => Self::HOLDER,
            Self::Prover => Self::PROVER,
            Self::Verifier => Self::VERIFIER,
            Self::Sender => Self::SENDER,
            Self::Receiver => Self::RECEIVER,
            Self::Requester => Self::REQUESTER,
            Self::Responder => Self::RESPONDER,
            Self::Notified => Self::NOTIFIED,
            Self::Notifier => Self::NOTIFIER,
            Self::Mediator => Self::MEDIATOR,
            Self::Other(s) => s.as_ref(),
        }
    }
}

impl From<&str> for Actor {
    fn from(s: &str) -> Self {
        match s {
            _ if s == Self::INVITER => Self::Inviter,
            _ if s == Self::INVITEE => Self::Invitee,
            _ if s == Self::ISSUER => Self::Issuer,
            _ if s == Self::HOLDER => Self::Holder,
            _ if s == Self::PROVER => Self::Prover,
            _ if s == Self::VERIFIER => Self::Verifier,
            _ if s == Self::SENDER => Self::Sender,
            _ if s == Self::RECEIVER => Self::Receiver,
            _ if s == Self::REQUESTER => Self::Requester,
            _ if s == Self::RESPONDER => Self::Responder,
            _ if s == Self::NOTIFIED => Self::Notified,
            _ if s == Self::NOTIFIER => Self::Notifier,
            _ if s == Self::MEDIATOR => Self::Mediator,
            _ => Self::Other(s.to_owned()),
        }
    }
}

impl Serialize for Actor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_ref().serialize(serializer)
    }
}
