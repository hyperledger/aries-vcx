pub mod pairwise;
pub mod public;

use derive_more::From;
use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;
use url::Url;

use self::{pairwise::PwInvitationContentBuilder, public::PublicInvitationContentBuilder};
pub use self::{
    pairwise::{PairwiseDidInvitationContent, PairwiseInvitationContent},
    public::PublicInvitationContent,
};

use crate::{decorators::timing::Timing, msg_parts::MsgParts};

pub type Invitation = MsgParts<InvitationContent, InvitationDecorators>;

/// We need another level of enum nesting since
/// an invitation can have multiple forms, and this way we
/// take advantage of `untagged` deserialization.
#[derive(Debug, Clone, From, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum InvitationContent {
    Public(PublicInvitationContent),
    Pairwise(PairwiseInvitationContent),
    PairwiseDID(PairwiseDidInvitationContent),
}

impl InvitationContent {
    pub fn builder_public() -> PublicInvitationContentBuilder {
        PublicInvitationContent::builder()
    }

    pub fn builder_pairwise() -> PwInvitationContentBuilder<Url> {
        PairwiseInvitationContent::builder()
    }

    pub fn builder_pairwise_did() -> PwInvitationContentBuilder<String> {
        PairwiseDidInvitationContent::builder()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq, TypedBuilder)]
pub struct InvitationDecorators {
    #[builder(default, setter(strip_option))]
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
}
