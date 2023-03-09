use std::collections::HashMap;

use serde::{Deserialize, Serialize, Serializer};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Thread {
    pub thid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pthid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_order: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub received_orders: Option<HashMap<String, u32>>, // should get replaced with DID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<ThreadGoalCode>,
}

impl Thread {
    pub fn new(thid: String) -> Self {
        Self {
            thid,
            pthid: None,
            sender_order: None,
            received_orders: None,
            goal_code: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(from = "&str")]
pub enum ThreadGoalCode {
    AriesVc,
    AriesVcIssue,
    AriesVcVerify,
    AriesVcRevoke,
    AriesRel,
    AriesRelBuild,
    Other(String),
}

impl ThreadGoalCode {
    const ARIES_VC: &str = "aries.vc";
    const ARIES_VC_ISSUE: &str = "aries.vc.issue";
    const ARIES_VC_VERIFY: &str = "aries.vc.verify";
    const ARIES_VC_REVOKE: &str = "aries.vc.revoke";
    const ARIES_REL: &str = "aries.rel";
    const ARIES_REL_BUILD: &str = "aries.rel.build";
}

impl AsRef<str> for ThreadGoalCode {
    fn as_ref(&self) -> &str {
        match self {
            Self::AriesVc => Self::ARIES_VC,
            Self::AriesVcIssue => Self::ARIES_VC_ISSUE,
            Self::AriesVcVerify => Self::ARIES_VC_VERIFY,
            Self::AriesVcRevoke => Self::ARIES_VC_REVOKE,
            Self::AriesRel => Self::ARIES_REL,
            Self::AriesRelBuild => Self::ARIES_REL_BUILD,
            Self::Other(s) => s.as_ref(),
        }
    }
}

impl From<&str> for ThreadGoalCode {
    fn from(s: &str) -> Self {
        match s {
            _ if s == Self::ARIES_VC => Self::AriesVc,
            _ if s == Self::ARIES_VC_ISSUE => Self::AriesVcIssue,
            _ if s == Self::ARIES_VC_VERIFY => Self::AriesVcVerify,
            _ if s == Self::ARIES_VC_REVOKE => Self::AriesVcRevoke,
            _ if s == Self::ARIES_REL => Self::AriesRel,
            _ if s == Self::ARIES_REL_BUILD => Self::AriesRelBuild,
            _ => Self::Other(s.to_owned()),
        }
    }
}

impl Serialize for ThreadGoalCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_ref().serialize(serializer)
    }
}
