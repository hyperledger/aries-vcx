use indy_api_types::errors::{IndyResult, IndyResultExt, IndyErrorKind};
use prost::Message;

pub trait ProstMessageExt {
    fn to_bytes(&self) -> IndyResult<Vec<u8>>;
    fn from_bytes(bytes: &[u8]) -> IndyResult<Self>
        where
            Self: Sized;
}

impl<T> ProstMessageExt for T
    where
        T: Message + Default,
{
    fn to_bytes(&self) -> IndyResult<Vec<u8>> {
        let mut bytes = Vec::new();
        Message::encode(self, &mut bytes).to_indy(
            IndyErrorKind::InvalidStructure,
            "Protobuf Message object cannot be encoded into the bytes vector"
        )?;
        Ok(bytes)
    }

    fn from_bytes(bytes: &[u8]) -> IndyResult<Self>
        where
            Self: Sized,
    {
        let decoded = Self::decode(bytes).to_indy(
            IndyErrorKind::InvalidStructure,
            "Protobuf Bytes cannot be decoded into the Message object"
        )?;
        Ok(decoded)
    }
}

#[cfg(test)]
mod test {
    use super::super::super::cheqd_ledger::prost_ext::ProstMessageExt;
    use super::super::super::cheqd_ledger::proto::cheqdid::cheqdnode::cheqd::v1::MsgCreateDidPayload as ProtoMsgCreateDidPayload;
    use super::super::super::cheqd_ledger::cheqd::v1::messages::{MsgCreateDidPayload, VerificationMethod, Service};
    use super::super::super::cheqd_ledger::CheqdProtoBase;
    use std::collections::HashMap;

    #[test]
    fn test_prost_message_ext() {
        let verification_method = VerificationMethod::new(
            "id".into(),
            "type".into(),
            "controller".into(),
            HashMap::new(),
            "public_key_multibase".into()
        );

        let did_service = Service::new(
            "id".into(),
            "type".into(),
            "service_endpoint".into()
        );

        let msg = MsgCreateDidPayload::new(
            vec!("context".to_string()),
            "id".into(),
            vec!("controller".to_string()),
            vec!(verification_method),
            vec!("authentication".to_string()),
            vec!("assertion_method".to_string()),
            vec!("capability_invocation".to_string()),
            vec!("capability_delegation".to_string()),
            vec!("key_agreement".to_string()),
            vec!(did_service),
            vec!("also_known_as".to_string()),
        );

        let proto: ProtoMsgCreateDidPayload = msg.to_proto().unwrap();

        let bytes: Vec<u8> = proto.to_bytes().unwrap();
        let decoded = ProtoMsgCreateDidPayload::from_bytes(bytes.as_slice()).unwrap();

        assert_eq!(proto, decoded);
    }
}
