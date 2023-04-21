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
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_serialize() {
        let value = Schema {
            data: vec!["name".to_string(), "age".to_string()],
            state: null,
            source_id: "foo".to_string(),
            thread_id: "bat".to_string(),
            ..Schema::default()
        };

        let serialized = value.serialize().unwrap();
        assert!(serialized.contains(r#""data":["name","age"]"#));
        assert!(serialized.contains(r#""source_id":"foo""#));
        assert!(serialized.contains(r#""state":"null""#));
        assert!(serialized.contains(r#""thread":"bat""#));

    }

    #[test]
    fn test_deserialize() {
        let serialized = r#"
        {
          "data": {
            "data": [
              "name",
              "age"
            ],
            "state": 1,
            "source_id": "foo",
            "thread_id": "bat"
          }
        }
        "#;

        let result = deserialize(serialized);
        assert!(result.is_ok());
        let ans = result.unwrap();

        assert_eq!(ans.data, vec!["name".to_string(), "age".to_string()]);
        assert_eq!(ans.state, PublicEntityStateType::Published);
        assert_eq!(ans.source_id, "foo");
        assert_eq!(ans.thread_id, "bat");
    }
}
