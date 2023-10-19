// Copyright 2023 Naian G.
// SPDX-License-Identifier: Apache-2.0

use reqwest::Error;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "http://localhost:7999";

#[test]
fn test_base_path_returns_hey() -> Result<(), Error> {
    let client = reqwest::blocking::Client::new();
    let res = client.get(BASE_URL.to_owned()).send()?;

    match res.error_for_status() {
        Ok(res) => {
            let res_text = res.text()?;
            assert_eq!("hey", res_text, "Unexpected response text");
            Ok(())
        }
        Err(err) => Err(err),
    }
}

#[test]
fn test_json_path_returns_groot() -> Result<(), Error> {
    let client = reqwest::blocking::Client::new();

    #[derive(Serialize)]
    struct ReqMsg {
        message: String,
    }
    #[derive(Deserialize, Debug)]
    struct ResMsg {
        message: String,
        response: String,
    }
    let endpoint = format!("{BASE_URL}/json");
    let req_msg = ReqMsg {
        message: "Hello Axum".to_owned(),
    };
    let res = client.post(endpoint).json(&req_msg).send()?;
    // match res.error_for_status_ref() {
    //     Ok(_) => (),
    //     Err(err) => {
    //         return Err(err);
    //     }
    // }
    res.error_for_status_ref()?;

    let res_json: ResMsg = res.json::<ResMsg>()?;
    assert_eq!(&req_msg.message, &res_json.message, "Mispronounced echo");
    assert_eq!(
        "I am groot", &res_json.response,
        "Server did not groot! Expected groot."
    );
    Ok(())
}
