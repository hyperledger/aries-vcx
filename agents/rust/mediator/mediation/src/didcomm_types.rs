// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0

pub use pickup_delivery_message_structs::*;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

pub mod type_uri {
    pub const FORWARD: &str = "https://didcomm.org/routing/1.0/forward";
    pub const PICKUP_STATUS_REQ: &str = "https://didcomm.org/messagepickup/2.0/status-request";
    pub const PICKUP_STATUS: &str = "https://didcomm.org/messagepickup/2.0/status";
    pub const PICKUP_DELIVERY_REQ: &str = "https://didcomm.org/messagepickup/2.0/delivery-request";
    pub const PICKUP_DELIVERY: &str = "https://didcomm.org/messagepickup/2.0/delivery";
    pub const PICKUP_RECEIVED: &str = "https://didcomm.org/messagepickup/2.0/messages-received";
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ForwardMsg {
    #[serde(rename = "@type")]
    _type: String,
    #[serde(rename = "to")]
    pub recipient_key: String,
    #[serde(rename = "msg")]
    pub message_data: String,
}

impl ForwardMsg {
    pub fn new(recipient_key: &str, message: &str) -> ForwardMsg {
        ForwardMsg {
            _type: type_uri::FORWARD.to_string(),
            recipient_key: recipient_key.to_string(),
            message_data: message.to_string(),
        }
    }
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
    pub recipient_key: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct PickupDeliveryReqMsg {
    #[serde(default)]
    pub limit: u32,
    pub recipient_key: Option<String>,
}

pub mod pickup_delivery_message_structs {
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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LiveDeliveryChangeMsg {
    pub live_delivery: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProblemReportMsg {
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageReceivedMsg {
    pub message_id_list: Vec<String>,
}

pub mod mediator_coord_structs {
    use serde::{Deserialize, Serialize};
    // use serde_with::skip_serializing_none;

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(tag = "@type")]
    pub enum MediatorCoordMsgEnum {
        #[serde(rename = "https://didcomm.org/coordinate-mediation/1.0/mediate-request")]
        MediateRequest,
        #[serde(rename = "https://didcomm.org/coordinate-mediation/1.0/mediate-deny")]
        MediateDeny(MediateDenyData),
        #[serde(rename = "https://didcomm.org/coordinate-mediation/1.0/mediate-grant")]
        MediateGrant(MediateGrantData),
        #[serde(rename = "https://didcomm.org/coordinate-mediation/1.0/keylist-update")]
        KeylistUpdateRequest(KeylistUpdateRequestData),
        #[serde(rename = "https://didcomm.org/coordinate-mediation/1.0/keylist-update-response")]
        KeylistUpdateResponse(KeylistUpdateResponseData),
        #[serde(rename = "https://didcomm.org/coordinate-mediation/1.0/keylist-query")]
        KeylistQuery(KeylistQueryData),
        #[serde(rename = "https://didcomm.org/coordinate-mediation/1.0/keylist")]
        Keylist(KeylistData),
        XumErrorMsg {
            error: String,
        },
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct MediateDenyData {
        pub reason: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct MediateGrantData {
        pub endpoint: String,
        pub routing_keys: Vec<String>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct KeylistUpdateRequestData {
        pub updates: Vec<KeylistUpdateItem>,
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub struct KeylistUpdateResponseData {
        pub updated: Vec<KeylistUpdateItem>,
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub struct KeylistUpdateItem {
        pub recipient_key: String,
        pub action: KeylistUpdateItemAction,
        pub result: Option<KeylistUpdateItemResult>,
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub enum KeylistUpdateItemAction {
        #[serde(rename = "add")]
        Add,
        #[serde(rename = "remove")]
        Remove,
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub enum KeylistUpdateItemResult {
        ClientError,
        ServerError,
        NoChange,
        Success,
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub struct KeylistQueryData {}
    #[derive(Serialize, Deserialize, Debug)]
    pub struct KeylistData {
        pub keys: Vec<KeylistItem>,
    }
    #[derive(Serialize, Deserialize, Debug)]
    pub struct KeylistItem {
        pub recipient_key: String,
    }
}
