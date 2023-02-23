pub mod pairwise;
pub mod public;

use derive_more::From;
use messages_macros::Message;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::decorators::Timing;
use crate::message_type::message_family::connection::ConnectionV1_0;

use crate::protocols::traits::ConcreteMessage;

use self::pairwise::PairwiseInvitation;
use self::public::PublicInvitation;

/// Type used to encapsulate a fully resolved invitation, which
/// contains all the information necessary for generating a [`crate::protocols::connection::request::Request`].
///
/// Other invitation types would get resolved to this.
pub type CompleteInvitation = InvitationImpl<Url>;

#[derive(Clone, Debug, From, Deserialize, Serialize, Message)]
#[message(kind = "ConnectionV1_0::Invitation")]
#[serde(untagged)]
pub enum Invitation {
    Public(PublicInvitation),
    Pairwise(PairwiseInvitation<Url>),
    PairwiseDID(PairwiseInvitation<String>),
}

/// Represents aninvitation with T as the service endpoint.
/// Essentially, T can only be a DID or a URL.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitationImpl<T> {
    #[serde(rename = "@id")]
    pub id: String,
    pub label: String,
    pub recipient_keys: Vec<String>,
    #[serde(default)]
    pub routing_keys: Vec<String>,
    pub service_endpoint: T,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
