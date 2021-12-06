use indy_api_types::errors::IndyResult;

use super::super::super::super::proto::cheqdid::cheqdnode::cheqd::v1::VerificationMethod as ProtoVerificationMethod;
use super::super::super::super::proto::cheqdid::cheqdnode::cheqd::v1::KeyValuePair as ProtoKeyValuePair;
use super::super::super::super::CheqdProtoBase;
use std::collections::HashMap;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct VerificationMethod {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub controller: String,
    #[serde(skip_serializing_if="HashMap::is_empty")]
    #[serde(default)]
    pub public_key_jwk: HashMap<String, String>,
    pub public_key_multibase: String,
}

impl VerificationMethod {
    pub fn new(
        id: String,
        r#type: String,
        controller: String,
        public_key_jwk: HashMap<String, String>,
        public_key_multibase: String) -> Self {
        VerificationMethod {
            id,
            r#type,
            controller,
            public_key_jwk,
            public_key_multibase
        }
    }
}

impl CheqdProtoBase for VerificationMethod {
    type Proto = ProtoVerificationMethod;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            id: self.id.clone(),
            r#type: self.r#type.clone(),
            controller: self.controller.clone(),
            public_key_jwk: self.public_key_jwk
                .iter()
                .map(|kv| {
                    ProtoKeyValuePair {
                        key:(*kv.0).clone(),
                        value:(*kv.1).clone()
                    }
                }).collect::<Vec<ProtoKeyValuePair>>(),
            public_key_multibase: self.public_key_multibase.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        let mut pkj_map: HashMap<String, String> = HashMap::new();
        proto.public_key_jwk
            .iter()
            .for_each(|proto_v| {
                pkj_map.insert(proto_v.key.to_string(), proto_v.value.to_string());
            });
        Ok(Self {
            id: proto.id.clone(),
            r#type: proto.r#type.clone(),
            controller: proto.controller.clone(),
            public_key_jwk: pkj_map,
            public_key_multibase: proto.public_key_multibase.clone(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::VerificationMethod;
    use super::super::super::super::super::CheqdProtoBase;
    use std::collections::HashMap;

    #[test]
    fn test_verification_method() {
        let msg = VerificationMethod::new(
            "id".into(),
            "type".into(),
            "controller".into(),
            HashMap::new(),
            "public_key_multibase".into()
        );

        let proto = msg.to_proto().unwrap();
        let decoded = VerificationMethod::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}
