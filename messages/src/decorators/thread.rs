use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use shared::maybe_known::MaybeKnown;
use typed_builder::TypedBuilder;

/// Struct representing the `~thread` decorator from its [RFC](<https://github.com/hyperledger/aries-rfcs/blob/main/concepts/0008-message-id-and-threading/README.md>).
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
pub struct Thread {
    pub thid: String,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pthid: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_order: Option<u32>,
    #[builder(default, setter(strip_option))]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub received_orders: Option<HashMap<String, u32>>, // should get replaced with DID.
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goal_code: Option<MaybeKnown<ThreadGoalCode>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum ThreadGoalCode {
    #[serde(rename = "aries.vc")]
    AriesVc,
    #[serde(rename = "aries.vc.issue")]
    AriesVcIssue,
    #[serde(rename = "aries.vc.verify")]
    AriesVcVerify,
    #[serde(rename = "aries.vc.revoke")]
    AriesVcRevoke,
    #[serde(rename = "aries.rel")]
    AriesRel,
    #[serde(rename = "aries.rel.build")]
    AriesRelBuild,
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
pub mod tests {
    use serde_json::json;

    use super::*;
    use crate::misc::test_utils;

    pub fn make_minimal_thread() -> Thread {
        let thid = "test".to_owned();
        Thread::builder().thid(thid).build()
    }

    pub fn make_extended_thread() -> Thread {
        let thid = "test".to_owned();
        let pthid = "test_pthid".to_owned();
        let sender_order = 5;
        let received_orders = HashMap::from([
            ("a".to_owned(), 1),
            ("b".to_owned(), 2),
            ("c".to_owned(), 3),
        ]);
        let goal_code = MaybeKnown::Known(ThreadGoalCode::AriesVcVerify);

        Thread::builder()
            .thid(thid)
            .pthid(pthid)
            .sender_order(sender_order)
            .received_orders(received_orders)
            .goal_code(goal_code)
            .build()
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
