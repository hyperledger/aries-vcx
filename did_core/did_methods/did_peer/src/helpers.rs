use std::collections::HashMap;
use serde::Serialize;
use serde_json::Value;
use did_doc::error::DidDocumentSovError;

pub fn convert_to_hashmap<T: Serialize>(
    value: &T,
) -> Result<HashMap<String, Value>, DidDocumentSovError> {
    let serialized_value = serde_json::to_value(value)?;

    match serialized_value {
        Value::Object(map) => Ok(map.into_iter().collect()),
        _ => Err(DidDocumentSovError::ParsingError(
            "Expected JSON object".to_string(),
        )),
    }
}
