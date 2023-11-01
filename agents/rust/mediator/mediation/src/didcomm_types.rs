// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
pub mod type_uri {
    pub const FORWARD: &str = "https://didcomm.org/routing/1.0/forward";
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
