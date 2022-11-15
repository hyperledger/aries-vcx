extern crate serde;
extern crate serde_json;

use std::string::String;

use serde_json::Map;
use serde_json::Value;

use crate::error::prelude::*;

pub trait KeyMatch {
    fn matches(&self, key: &String, context: &Vec<String>) -> bool;
}

impl KeyMatch for String {
    fn matches(&self, key: &String, _context: &Vec<String>) -> bool {
        key.eq(self)
    }
}

/*
Rewrites keys in a serde value structor to new mapped values. Returns the remapped value. Leaves
unmapped keys as they are.
*/
pub fn mapped_key_rewrite<T: KeyMatch>(val: Value, remap: &Vec<(T, String)>) -> VcxResult<Value> {
    let mut context: Vec<String> = Default::default();
    _mapped_key_rewrite(val, &mut context, remap)
}

fn _mapped_key_rewrite<T: KeyMatch>(
    val: Value,
    context: &mut Vec<String>,
    remap: &Vec<(T, String)>,
) -> VcxResult<Value> {
    if let Value::Object(mut map) = val {
        let mut keys: Vec<String> = _collect_keys(&map);

        while let Some(k) = keys.pop() {
            let mut value = map.remove(&k).ok_or_else(|| {
                warn!("Unexpected key value mutation");
                VcxError::from_msg(VcxErrorKind::InvalidJson, "Unexpected key value mutation")
            })?;

            let mut new_k = k;
            for matcher in remap {
                if matcher.0.matches(&new_k, context) {
                    new_k = matcher.1.clone();
                    break;
                }
            }

            context.push(new_k.clone()); // TODO not efficient, should work with references
            value = _mapped_key_rewrite(value, context, remap)?;
            context.pop();

            map.insert(new_k, value);
        }
        Ok(Value::Object(map))
    } else {
        Ok(val)
    }
}

fn _collect_keys(map: &Map<String, Value>) -> Vec<String> {
    let mut rtn: Vec<String> = Default::default();
    for key in map.keys() {
        rtn.push(key.clone());
    }
    rtn
}

pub(crate) trait TryGetIndex {
    type Val;
    fn try_get(&self, index: &str) -> Result<Self::Val, VcxError>;
}

impl<'a> TryGetIndex for &'a Value {
    type Val = &'a Value;
    fn try_get(&self, index: &str) -> Result<&'a Value, VcxError> {
        self.get(index).ok_or(VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!("Could not index '{}' in Value payload: {:?}", index, self),
        ))
    }
}

pub(crate) trait AsTypeOrDeserializationError {
    fn try_as_str(&self) -> Result<&str, VcxError>;

    fn try_as_object(&self) -> Result<&Map<String, Value>, VcxError>;

    fn try_as_bool(&self) -> Result<bool, VcxError>;

    fn try_as_array(&self) -> Result<&Vec<Value>, VcxError>;
}

impl AsTypeOrDeserializationError for &Value {
    fn try_as_str(&self) -> Result<&str, VcxError> {
        self.as_str().ok_or(VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!("Could not deserialize '{}' value as string", self.to_string()),
        ))
    }

    fn try_as_object(&self) -> Result<&Map<String, Value>, VcxError> {
        self.as_object().ok_or(VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!("Could not deserialize '{}' value as object", self.to_string()),
        ))
    }

    fn try_as_bool(&self) -> Result<bool, VcxError> {
        self.as_bool().ok_or(VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!("Could not deserialize '{}' value as bool", self.to_string()),
        ))
    }

    fn try_as_array(&self) -> Result<&Vec<Value>, VcxError> {
        self.as_array().ok_or(VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!("Could not deserialize '{}' value as bool", self.to_string()),
        ))
    }
}

#[cfg(test)]
#[cfg(feature = "general_test")]
mod unit_tests {
    use super::*;

    #[test]
    fn simple() {
        let simple_map = vec![("d".to_string(), "devin".to_string())];
        let simple = json!({"d":"d"});
        let expected = json!({"devin":"d"});
        let transformed = mapped_key_rewrite(simple, &simple_map).unwrap();
        assert_eq!(expected, transformed);

        let simple = json!(null);
        let transformed = mapped_key_rewrite(simple.clone(), &simple_map).unwrap();
        assert_eq!(simple, transformed);

        let simple = json!("null");
        let transformed = mapped_key_rewrite(simple.clone(), &simple_map).unwrap();
        assert_eq!(simple, transformed);
    }
}
