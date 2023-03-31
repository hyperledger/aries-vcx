use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectWithVersion<'a, T> {
    pub version: &'a str,
    pub data: T,
}

impl<'a, T> ObjectWithVersion<'a, T>
where
    T: ::serde::Serialize + ::serde::de::DeserializeOwned,
{
    pub fn new(version: &'a str, data: T) -> ObjectWithVersion<'a, T> {
        ObjectWithVersion { version, data }
    }

    pub fn serialize(&self) -> VcxResult<String> {
        ::serde_json::to_string(self).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Cannot serialize object: {}", err),
            )
        })
    }

    pub fn deserialize(data: &str) -> VcxResult<ObjectWithVersion<T>>
    where
        T: ::serde::de::DeserializeOwned,
    {
        ::serde_json::from_str(data).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Cannot deserialize object: {}", err),
            )
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum SerializableObjectWithState<T, P> {
    #[serde(rename = "1.0")]
    V1 {
        data: T,
        state: P,
        source_id: String,
        thread_id: String,
    },
}
