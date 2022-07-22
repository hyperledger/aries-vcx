use crate::error::{VcxError, VcxErrorKind, VcxResult, VcxResultExt};

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectWithVersion<'a, T> {
    pub version: &'a str,
    pub data: T,
}

impl<'a, 'de, T> ObjectWithVersion<'a, T> where T: ::serde::Serialize + ::serde::de::DeserializeOwned {
    pub fn new(version: &'a str, data: T) -> ObjectWithVersion<'a, T> {
        ObjectWithVersion { version, data }
    }

    pub fn serialize(&self) -> VcxResult<String> {
        ::serde_json::to_string(self)
            .to_vcx(VcxErrorKind::InvalidState, "Cannot serialize object")
    }

    pub fn deserialize(data: &str) -> VcxResult<ObjectWithVersion<T>> where T: ::serde::de::DeserializeOwned {
        ::serde_json::from_str(data)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize object: {}", err)))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum SerializableObjectWithState<T, P> {
    #[serde(rename = "1.0")]
    V1 { data: T, state: P, source_id: String, thread_id: String },
}
