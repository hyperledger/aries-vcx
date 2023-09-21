use indy_ledger_response_parser::ResponseParser;

pub const TAA_RESPONSE: &str = r#"{
    "op": "REPLY",
    "result": {
        "type": "6",
        "identifier": "L5AD5g65TDQr1PPHHRoiGf",
        "reqId": 1514308188474704,

        "version": "1.0",

        "seqNo": 10,
        "txnTime": 1514214795,

        "data": {
            "aml": {
              "at_submission": "The agreement was reviewed by the user and accepted at the time of submission of this transaction.",
              "for_session": "The agreement was reviewed by the user and accepted at some point in the user's session prior to submission.",
              "on_file": "An authorized person accepted the agreement, and such acceptance is on file with the user's organization.",
              "product_eula": "The agreement was included in the software product's terms and conditions as part of the license to the user.",
              "service_agreement": "The agreement was included in the terms and conditions the Transaction Author accepted as part of contracting a service.",
              "wallet_agreement": "The agreement was reviewed by the user and this affirmation was persisted in the user's wallet for use during future submissions."
            },
            "digest": "8cee5d7a573e4893b08ff53a0761a22a1607df3b3fcd7e75b98696c92879641f",
            "ratification_ts": 1575417600,
            "version": "2.0",
            "text": "Transaction Author Agreement V2"
        }
    }
}"#;

#[test]
fn test_parse_get_txn_author_agreement_response() {
    let parsed_response = ResponseParser
        .parse_get_txn_author_agreement_response(TAA_RESPONSE)
        .unwrap();

    assert_eq!(parsed_response.version, "2.0");
    assert_eq!(parsed_response.text, "Transaction Author Agreement V2");
    assert_eq!(
        parsed_response.digest.unwrap(),
        "8cee5d7a573e4893b08ff53a0761a22a1607df3b3fcd7e75b98696c92879641f"
    );
}
