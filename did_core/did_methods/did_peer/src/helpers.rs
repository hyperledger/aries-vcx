use std::collections::HashMap;

use did_doc::error::DidDocumentBuilderError;
use serde::Serialize;
use serde_json::Value;

pub fn convert_to_hashmap<T: Serialize>(
    value: &T,
) -> Result<HashMap<String, Value>, DidDocumentBuilderError> {
    let serialized_value = serde_json::to_value(value)?;

    match serialized_value {
        Value::Object(map) => Ok(map.into_iter().collect()),
        _ => Err(DidDocumentBuilderError::CustomError(
            "Expected JSON object".to_string(),
        )),
    }
}
