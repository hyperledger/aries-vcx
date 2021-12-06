use super::super::super::super::proto::cheqdid::cheqdnode::cheqd::v1::SignInfo as ProtoSignInfo;
use super::super::super::super::CheqdProtoBase;
use indy_api_types::errors::IndyResult;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct SignInfo {
    pub verification_method_id: String,
    pub signature: String,
}

impl SignInfo {
    pub fn new(verification_method_id: String, signature: String) -> Self {
        SignInfo {
            verification_method_id,
            signature,
        }
    }
}

impl CheqdProtoBase for SignInfo {
    type Proto = ProtoSignInfo;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            verification_method_id: self.verification_method_id.clone(),
            signature: self.signature.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self {
            verification_method_id: proto.verification_method_id.clone(),
            signature: proto.signature.clone(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::SignInfo;
    use super::super::super::super::super::CheqdProtoBase;

    #[test]
    fn test_metadata_struct() {
        let msg = SignInfo::new(
            "verification_method_id".into(),
            "signature".into());

        let proto = msg.to_proto().unwrap();
        let decoded = SignInfo::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}
