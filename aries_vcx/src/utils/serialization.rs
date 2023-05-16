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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_serialize() {
        let value = SerializableObjectWithState::V1 {
            data: vec!["name".to_string(), "age".to_string()],
            state: "1".to_string(),
            source_id: "foo".to_string(),
            thread_id: "bat".to_string(),
        };

        let serialized = serde_json::to_string(&value).unwrap();

        assert!(serialized.contains(r#""data":["name","age"]"#));
        assert!(serialized.contains(r#""state":"1""#));
        assert!(serialized.contains(r#""source_id":"foo""#));
        assert!(serialized.contains(r#""thread_id":"bat""#));
    }

    #[test]
    fn test_deserialize() {
        let serialized = r#"
        {
            "data": [
            "name",
            "age"
            ],
            "state": "1",
            "source_id": "foo",
            "thread_id": "bat",
            "version": "1.0"
        }
        "#;

        let result = serde_json::from_str(&serialized);
        let ans: SerializableObjectWithState<Vec<String>, String> = result.unwrap();

        let (data, state, source_id, thread_id) = match ans {
            SerializableObjectWithState::V1 {
                data,
                state,
                source_id,
                thread_id,
            } => (data, state, source_id, thread_id),
        };

        assert_eq!(data, vec!["name".to_string(), "age".to_string()]);
        assert_eq!(state, "1");
        assert_eq!(source_id, "foo");
        assert_eq!(thread_id, "bat");
    }
}
