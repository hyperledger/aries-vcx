use std::collections::HashMap;

use serde::{Deserialize, Serialize, Serializer};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
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

#[derive(Clone, Debug, Deserialize, PartialEq)]
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
pub mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    pub fn make_minimal_thread() -> Thread {
        let thid = "test".to_owned();
        Thread::new(thid)
    }

    pub fn make_extended_thread() -> Thread {
        let thid = "test".to_owned();
        let mut thread = Thread::new(thid);

        let pthid = "test_pthid".to_owned();
        let sender_order = 5;
        let received_orders = HashMap::from([("a".to_owned(), 1), ("b".to_owned(), 2), ("c".to_owned(), 3)]);
        let goal_code = ThreadGoalCode::AriesVcVerify;

        thread.pthid = Some(pthid);
        thread.sender_order = Some(sender_order);
        thread.received_orders = Some(received_orders);
        thread.goal_code = Some(goal_code);

        thread
    }

    #[test]
    fn test_minimal_thread() {
        let thread = make_minimal_thread();
        let json = json!({ "thid": thread.thid });

        test_utils::test_serde(thread, json);
    }

    #[test]
    fn test_extensive_thread() {
        let thread = make_extended_thread();

        let json = json!({
            "thid": thread.thid,
            "pthid": thread.pthid,
            "sender_order": thread.sender_order,
            "received_orders": thread.received_orders,
            "goal_code": thread.goal_code
        });

        test_utils::test_serde(thread, json);
    }
}
