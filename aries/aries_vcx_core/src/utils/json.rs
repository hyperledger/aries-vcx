extern crate serde;
extern crate serde_json;

use std::string::String;

use serde_json::{Map, Value};

use crate::errors::error::prelude::*;

pub(crate) trait TryGetIndex {
    type Val;
    fn try_get(&self, index: &str) -> Result<Self::Val, AriesVcxCoreError>;
}

impl<'a> TryGetIndex for &'a Value {
    type Val = &'a Value;
    fn try_get(&self, index: &str) -> Result<&'a Value, AriesVcxCoreError> {
        self.get(index).ok_or_else(|| {
            AriesVcxCoreError::from_msg(
                AriesVcxCoreErrorKind::InvalidJson,
                format!("Could not index '{index}' in Value payload: {self:?}"),
            )
        })
    }
}

pub(crate) trait AsTypeOrDeserializationError {
    fn try_as_str(&self) -> Result<&str, AriesVcxCoreError>;

    fn try_as_object(&self) -> Result<&Map<String, Value>, AriesVcxCoreError>;

    fn try_as_bool(&self) -> Result<bool, AriesVcxCoreError>;

    fn try_as_array(&self) -> Result<&Vec<Value>, AriesVcxCoreError>;
}

impl AsTypeOrDeserializationError for &Value {
    fn try_as_str(&self) -> Result<&str, AriesVcxCoreError> {
        self.as_str().ok_or(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::InvalidJson,
            format!("Could not deserialize '{self}' value as string"),
        ))
    }

    fn try_as_object(&self) -> Result<&Map<String, Value>, AriesVcxCoreError> {
        self.as_object().ok_or(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::InvalidJson,
            format!("Could not deserialize '{self}' value as object"),
        ))
    }

    fn try_as_bool(&self) -> Result<bool, AriesVcxCoreError> {
        self.as_bool().ok_or(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::InvalidJson,
            format!("Could not deserialize '{self}' value as bool"),
        ))
    }

    fn try_as_array(&self) -> Result<&Vec<Value>, AriesVcxCoreError> {
        self.as_array().ok_or(AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::InvalidJson,
            format!("Could not deserialize '{self}' value as bool"),
        ))
    }
}
