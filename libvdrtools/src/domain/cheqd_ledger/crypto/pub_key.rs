//! Helper class to handle private keys generic proto conversion

use indy_api_types::errors::{IndyErrorKind, IndyResult};
use indy_api_types::IndyError;

use super::super::CheqdProtoBase;

use super::secp256k1;
use super::super::CheqdProto;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type_url", content = "value")]
pub enum PubKey {
    Secp256k1(secp256k1::PubKey),
}

impl CheqdProtoBase for PubKey {
    type Proto = prost_types::Any;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        match self {
            PubKey::Secp256k1(pk) => {
                Ok(prost_types::Any {
                    type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
                    value: pk.to_proto_bytes()?,
                })
            }
        }
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        match &proto.type_url[..] {
            "/cosmos.crypto.secp256k1.PubKey" => {
                let val = secp256k1::PubKey::from_proto_bytes(&proto.value)?;
                Ok(PubKey::Secp256k1(val))
            }
            unknown_type => Err(IndyError::from_msg(
                IndyErrorKind::InvalidStructure,
                format!("Unknown pub_key type: {}", unknown_type),
            )),
        }
    }
}
