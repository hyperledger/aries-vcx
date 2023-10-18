#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct PickupStatusMsg {
    pub message_count: u32,
    pub recipient_key: Option<String>,
}

impl PickupStatusMsg {
    pub fn new(message_count: u32, recipient_key: Option<String>) -> PickupStatusMsg {
        PickupStatusMsg {
            message_count,
            recipient_key,
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct PickupStatusReqMsg {
    #[serde(default)]
    pub auth_pubkey: String,
    pub recipient_key: Option<String>,
}

// impl PickupStatusReqMsg {
//     pub fn new(recipient_key: Option<String>) -> PickupStatusReqMsg {
//         PickupStatusReqMsg { recipient_key }
//     }
//     // pub fn custom_type(self, _type: String) -> PickupStatusReqMsg {
//     //     PickupStatusReqMsg {
//     //          recipient_key: self.recipient_key,
//     //     }
//     // }
// }

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct PickupDeliveryReqMsg {
    #[serde(default)]
    pub auth_pubkey: String,
    pub limit: u32,
    pub recipient_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "@type")]
pub enum PickupMsgEnum {
    #[serde(rename = "https://didcomm.org/messagepickup/2.0/status")]
    PickupStatusMsg(PickupStatusMsg),
    #[serde(rename = "https://didcomm.org/messagepickup/2.0/status-request")]
    PickupStatusReqMsg(PickupStatusReqMsg),
    #[serde(rename = "https://didcomm.org/messagepickup/2.0/delivery-request")]
    PickupDeliveryReq(PickupDeliveryReqMsg),
    #[serde(rename = "https://didcomm.org/messagepickup/2.0/delivery")]
    PickupDelivery(PickupDeliveryMsg),
    #[serde(rename = "https://didcomm.org/messagepickup/2.0/messages-received")]
    MessageReceived(MessageReceivedMsg),
    #[serde(rename = "https://didcomm.org/messagepickup/2.0/live-delivery-change")]
    LiveDeliveryChange(LiveDeliveryChangeMsg),
    #[serde(rename = "https://didcomm.org/notification/1.0/problem-report")]
    ProblemReport(ProblemReportMsg),
}

use serde_with::{base64::Base64, serde_as};

use super::{skip_serializing_none, Deserialize, Serialize};

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct PickupDeliveryMsgAttachData {
    #[serde_as(as = "Base64")]
    pub base64: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PickupDeliveryMsgAttach {
    pub id: String,
    #[serde(rename = "data")]
    pub data: PickupDeliveryMsgAttachData,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct PickupDeliveryMsg {
    pub recipient_key: Option<String>,
    #[serde(rename = "~attach")]
    pub attach: Vec<PickupDeliveryMsgAttach>,
}
