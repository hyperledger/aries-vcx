// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0

use std::sync::Once;

use serde_json::json;
static INIT: Once = Once::new();

const BASE_URL: &str = "http://localhost:7999";

// Test variables
const AUTH_PUBKEY: &str = "Anderson Smith n0r3t1";
const RECIPIENT_KEY: &str = "Anderson Smith n0r3t1r1";
const DID_DOC: &str = "{}";

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
pub fn initialize() {
    INIT.call_once(|| {
        setup_account();
        setup_recipient();
    });
}

// #[ignore]
#[test]
fn test_forward_message() {
    initialize();
    let client = reqwest::blocking::Client::new();
    let endpoint = format!("{BASE_URL}/forward");
    let forward_message = json!(
        {
            "@type" : "https://didcomm.org/routing/1.0/forward",
            "@id": "54ad1a63-29bd-4a59-abed-1c5b1026e6fd",
            "to"   : RECIPIENT_KEY,
            "msg"  : "<super secret packed DIDCOMM message>"
        }
    );
    let res = client.post(endpoint).json(&forward_message).send().unwrap();
    res.error_for_status().unwrap();
}
