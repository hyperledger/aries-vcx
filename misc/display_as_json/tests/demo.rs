extern crate display_as_json;
extern crate serde;

use serde::{Deserialize, Serialize};

use crate::display_as_json::Display;

#[derive(Serialize, Deserialize, Display)]
struct TestStruct {
    field1: u32,
    field2: String,
}

#[test]
fn test_display_as_json() {
    let instance = TestStruct {
        field1: 42,
        field2: "hello".to_string(),
    };
    let displayed = format!("{}", instance);
    let expected = r#"{"field1":42,"field2":"hello"}"#;
    assert_eq!(displayed, expected);
}
