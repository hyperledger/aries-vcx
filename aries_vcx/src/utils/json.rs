extern crate serde;
extern crate serde_json;

use std::string::String;

use serde_json::{Map, Value};

use crate::errors::error::prelude::*;

pub(crate) trait TryGetIndex {
    type Val;
    fn try_get(&self, index: &str) -> Result<Self::Val, AriesVcxError>;
}

impl<'a> TryGetIndex for &'a Value {
    type Val = &'a Value;
    fn try_get(&self, index: &str) -> Result<&'a Value, AriesVcxError> {
        self.get(index).ok_or_else(|| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Could not index '{}' in Value payload: {:?}", index, self),
            )
        })
    }
}

pub(crate) trait AsTypeOrDeserializationError {
    fn try_as_str(&self) -> Result<&str, AriesVcxError>;

    fn try_as_object(&self) -> Result<&Map<String, Value>, AriesVcxError>;

    fn try_as_bool(&self) -> Result<bool, AriesVcxError>;

    fn try_as_array(&self) -> Result<&Vec<Value>, AriesVcxError>;
}

impl AsTypeOrDeserializationError for &Value {
    fn try_as_str(&self) -> Result<&str, AriesVcxError> {
        self.as_str().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!("Could not deserialize '{}' value as string", self),
        ))
    }

    fn try_as_object(&self) -> Result<&Map<String, Value>, AriesVcxError> {
        self.as_object().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!("Could not deserialize '{}' value as object", self),
        ))
    }

    fn try_as_bool(&self) -> Result<bool, AriesVcxError> {
        self.as_bool().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!("Could not deserialize '{}' value as bool", self),
        ))
    }

    fn try_as_array(&self) -> Result<&Vec<Value>, AriesVcxError> {
        self.as_array().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!("Could not deserialize '{}' value as bool", self),
        ))
    }
}
