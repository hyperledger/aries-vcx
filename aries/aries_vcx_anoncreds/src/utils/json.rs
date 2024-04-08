extern crate serde;
extern crate serde_json;

use std::string::String;

use serde_json::{Map, Value};

use crate::errors::error::{VcxAnoncredsError, VcxAnoncredsResult};

pub(crate) trait TryGetIndex {
    type Val;
    fn try_get(&self, index: &str) -> VcxAnoncredsResult<Self::Val>;
}

impl<'a> TryGetIndex for &'a Value {
    type Val = &'a Value;
    fn try_get(&self, index: &str) -> VcxAnoncredsResult<&'a Value> {
        self.get(index).ok_or_else(|| {
            VcxAnoncredsError::InvalidJson(format!(
                "Could not index '{index}' in Value payload: {self:?}"
            ))
        })
    }
}

pub(crate) trait AsTypeOrDeserializationError {
    fn try_as_str(&self) -> VcxAnoncredsResult<&str>;

    fn try_as_object(&self) -> VcxAnoncredsResult<&Map<String, Value>>;

    fn try_as_bool(&self) -> VcxAnoncredsResult<bool>;

    fn try_as_array(&self) -> VcxAnoncredsResult<&Vec<Value>>;
}

impl AsTypeOrDeserializationError for &Value {
    fn try_as_str(&self) -> VcxAnoncredsResult<&str> {
        self.as_str().ok_or(VcxAnoncredsError::InvalidJson(format!(
            "Could not deserialize '{self}' value as string"
        )))
    }

    fn try_as_object(&self) -> VcxAnoncredsResult<&Map<String, Value>> {
        self.as_object()
            .ok_or(VcxAnoncredsError::InvalidJson(format!(
                "Could not deserialize '{self}' value as object"
            )))
    }

    fn try_as_bool(&self) -> VcxAnoncredsResult<bool> {
        self.as_bool().ok_or(VcxAnoncredsError::InvalidJson(format!(
            "Could not deserialize '{self}' value as bool"
        )))
    }

    fn try_as_array(&self) -> VcxAnoncredsResult<&Vec<Value>> {
        self.as_array()
            .ok_or(VcxAnoncredsError::InvalidJson(format!(
                "Could not deserialize '{self}' value as bool"
            )))
    }
}
