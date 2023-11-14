// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Once;

use env_logger::Env;
use log::warn;
use serde_json::json;
static INIT: Once = Once::new();
const BASE_URL: &str = "http://localhost:7999";

// Test variables
const AUTH_PUBKEY: &str = "Anderson Smith n0r3t1";
const RECIPIENT_KEY: &str = "Anderson Smith n0r3t1r1";
const DID_DOC: &str = "{}";

fn setup_logging() {
    let env = Env::default().default_filter_or("info");
    env_logger::init_from_env(env);
}
fn setup_account() {
    let client = reqwest::blocking::Client::new();
    let endpoint = format!("{BASE_URL}/coord");
    let new_account_req = json!(
        {
            "@type": "https://didcomm.org/coordinate-mediation/1.0/mediate-request",
            "auth_pubkey": AUTH_PUBKEY,
            "did_doc": DID_DOC
        }
    );
    let res = client.post(endpoint).json(&new_account_req).send().unwrap();
    res.error_for_status().unwrap();
}
fn setup_recipient() {
    let client = reqwest::blocking::Client::new();
    let endpoint = format!("{BASE_URL}/coord");
    let add_recipient_req = json!(
        {
            "@type": "https://didcomm.org/coordinate-mediation/1.0/keylist-update",
            "auth_pubkey": AUTH_PUBKEY,
            "updates": [
              {
                "recipient_key": RECIPIENT_KEY,
                "action": "add"
              }
            ]
          }
    );
    let res = client
        .post(endpoint)
        .json(&add_recipient_req)
        .send()
        .unwrap();
    res.error_for_status().unwrap();
}

fn initialize() {
    INIT.call_once(|| {
        setup_logging();
        setup_account();
        setup_recipient();
    });
}

#[test]
fn test_status_request_endpoint_exists() {
    initialize();
    let client = reqwest::blocking::Client::new();
    let endpoint = format!("{BASE_URL}/pickup");

    let (id, recipient_key) = (123, RECIPIENT_KEY);
    let status_request = json!(
        {
            "@id": id,
            "@type": "https://didcomm.org/messagepickup/2.0/status-request",
            "auth_pubkey": AUTH_PUBKEY,
            "recipient_key": recipient_key
        }
    );
    let res = client.post(endpoint).json(&status_request).send().unwrap();
    res.error_for_status().unwrap();
}

#[test]
fn test_status_request_returns_a_valid_status() {
    initialize();
    let client = reqwest::blocking::Client::new();
    let endpoint = format!("{BASE_URL}/pickup");

    let status_request = json!(
        {
            "@id": 123,
            "@type": "https://didcomm.org/messagepickup/2.0/status-request",
            "auth_pubkey": AUTH_PUBKEY,
        }
    );
    let res = client.post(endpoint).json(&status_request).send().unwrap();
    let res_msg = res.json::<serde_json::Value>().unwrap();
    assert_eq!(
        "https://didcomm.org/messagepickup/2.0/status",
        res_msg["@type"]
    );
    assert!(res_msg.get("message_count").is_some());
}

#[test]
fn test_status_request_for_key_returns_a_valid_status() {
    initialize();
    let client = reqwest::blocking::Client::new();
    let endpoint = format!("{BASE_URL}/pickup");

    let status_request = json!(
        {
            "@id": 123,
            "@type": "https://didcomm.org/messagepickup/2.0/status-request",
            "auth_pubkey": AUTH_PUBKEY,
            "recipient_key": RECIPIENT_KEY
        }
    );
    let res = client.post(endpoint).json(&status_request).send().unwrap();
    let res_msg = res.json::<serde_json::Value>().unwrap();
    assert_eq!(
        "https://didcomm.org/messagepickup/2.0/status",
        res_msg["@type"]
    );
    assert!(res_msg.get("message_count").is_some());
    assert_eq!(RECIPIENT_KEY, res_msg["recipient_key"]);
}

// {
//     "@id": "123456781",
//     "@type": "https://didcomm.org/messagepickup/2.0/status",
//     "recipient_key": "<key for messages>",
//     "message_count": 7,
//     "longest_waited_seconds": 3600,
//     "newest_received_time": "2019-05-01 12:00:00Z",
//     "oldest_received_time": "2019-05-01 12:00:01Z",
//     "total_bytes": 8096,
//     "live_delivery": false
// }

#[test]
fn test_delivery_request_returns_status_when_queue_empty() {
    initialize();
    let client = reqwest::blocking::Client::new();
    let endpoint = format!("{BASE_URL}/pickup");
    // Use non existing recipient key to test 0 waiting messages case
    let delivery_req = json!(
        {
            "@id": "123456781",
            "@type": "https://didcomm.org/messagepickup/2.0/delivery-request",
            "auth_pubkey": AUTH_PUBKEY,

            "limit": 10,
            "recipient_key": "<key for messages>"
        }
    );

    let res = client.post(endpoint).json(&delivery_req).send().unwrap();
    if let Err(err) = res.error_for_status_ref() {
        warn!("Error response status {:#?}", err);
    }
    let res_msg = res.json::<serde_json::Value>().unwrap();
    assert_eq!(
        "https://didcomm.org/messagepickup/2.0/status",
        res_msg["@type"]
    );
    assert_eq!(0, res_msg["message_count"]);
}

#[test]
fn test_delivery_request() {
    initialize();
    let client = reqwest::blocking::Client::new();
    let endpoint = format!("{BASE_URL}/pickup");
    let delivery_request = json!(
        {
            "@id": 123,
            "@type": "https://didcomm.org/messagepickup/2.0/delivery-request",
            "auth_pubkey": AUTH_PUBKEY,
            "limit": 10
        }
    );
    let res = client
        .post(endpoint)
        .json(&delivery_request)
        .send()
        .unwrap();
    if let Err(err) = res.error_for_status_ref() {
        warn!("Error response status {:#?}", err);
    }
    let res_msg = res.json::<serde_json::Value>().unwrap();
    assert_eq!(
        "https://didcomm.org/messagepickup/2.0/delivery",
        res_msg["@type"]
    );
    // assert_ne!(0, res_msg["message_count"]);
}
// {
//     "@id": "123456781",
//     "@type": "https://didcomm.org/messagepickup/2.0/delivery-request",
//     "limit": 10,
//     "recipient_key": "<key for messages>"
// }

// {
//     "@type": "https://didcomm.org/messagepickup/2.0/delivery-request",
//     "limit": 1
// }

// {
//     "@id": "123456781",
//     "~thread": {
//         "thid": "<message id of delivery-request message>"
//       },
//     "@type": "https://didcomm.org/messagepickup/2.0/delivery",
//     "recipient_key": "<key for messages>",
//     "~attach": [{
//     	"@id": "<messageid>",
//     	"data": {
//     		"base64": ""
//     	}
//     }]
// }

// {
//     "@type": "https://didcomm.org/messagepickup/2.0/messages-received",
//     "message_id_list": ["123","456"]
// }

// Multiple Recipients

// // If a message arrives at a Mediator addressed to multiple Recipients,
// // the message MUST be queued for each Recipient independently.
// // If one of the addressed Recipients retrieves a message and indicates it has been received,
// // that message MUST still be held and then removed by the other addressed Recipients.

// {
//     "@type": "https://didcomm.org/messagepickup/2.0/live-delivery-change",
//     "live_delivery": true
// }

// {
//     "@type": "https://didcomm.org/notification/1.0/problem-report",
//     "~thread": {
//       "pthid": "<message id of offending live_delivery_change>"
//     },
//     "description": "Connection does not support Live Delivery"
// }
