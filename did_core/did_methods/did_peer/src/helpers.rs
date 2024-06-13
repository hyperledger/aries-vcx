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

// https://multiformats.io/multihash/
// 0x12 - multihash/multicodec "Sha2-256"
// 0x20 - length of 32 bytes (256 bits)
pub(crate) const MULTIHASH_SHA2_256: [u8; 2] = [0x12u8, 0x20u8];
// https://github.com/multiformats/multicodec/blob/master/table.csv
// multicodec JSON (0x0200) as a varint 
pub(crate) const MULTICODEC_JSON_VARINT: [u8; 2] = [0x80u8, 0x04u8];