use cosmrs::proto::cosmos::crypto::secp256k1::PubKey as ProtoPubKey;
use indy_api_types::errors::IndyResult;

use super::super::super::CheqdProtoBase;

/// PubKey defines a secp256k1 public key
/// Key is the compressed form of the pubkey. The first byte depends is a 0x02 byte
/// if the y-coordinate is the lexicographically largest of the two associated with
/// the x-coordinate. Otherwise the first byte is a 0x03.
/// This prefix is followed with the x-coordinate.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct PubKey {
    pub key: Vec<u8>,
}

impl PubKey {
    pub fn new(key: Vec<u8>) -> Self {
        PubKey { key }
    }
}

impl CheqdProtoBase for PubKey {
    type Proto = ProtoPubKey;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            key: self.key.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(proto.key.clone()))
    }
}
