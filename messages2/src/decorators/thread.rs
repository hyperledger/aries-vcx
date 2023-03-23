use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::maybe_known::MaybeKnown;

/// Struct representing the `~thread` decorator from its [RFC](https://github.com/hyperledger/aries-rfcs/blob/main/concepts/0008-message-id-and-threading/README.md).
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
    pub goal_code: Option<MaybeKnown<ThreadGoalCode>>,
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ThreadGoalCode {
    AriesVc,
    AriesVcIssue,
    AriesVcVerify,
    AriesVcRevoke,
    AriesRel,
    AriesRelBuild,
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
        let goal_code = MaybeKnown::Known(ThreadGoalCode::AriesVcVerify);

        thread.pthid = Some(pthid);
        thread.sender_order = Some(sender_order);
        thread.received_orders = Some(received_orders);
        thread.goal_code = Some(goal_code);

        thread
    }

    #[test]
    fn test_minimal_thread() {
        let thread = make_minimal_thread();
        let expected = json!({ "thid": thread.thid });

        test_utils::test_serde(thread, expected);
    }

    #[test]
    fn test_extended_thread() {
        let thread = make_extended_thread();

        let expected = json!({
            "thid": thread.thid,
            "pthid": thread.pthid,
            "sender_order": thread.sender_order,
            "received_orders": thread.received_orders,
            "goal_code": thread.goal_code
        });

        test_utils::test_serde(thread, expected);
    }
}
