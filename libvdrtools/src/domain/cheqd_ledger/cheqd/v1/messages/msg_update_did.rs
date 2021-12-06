use indy_api_types::errors::IndyResult;

use super::super::super::super::proto::cheqdid::cheqdnode::cheqd::v1::MsgUpdateDid as ProtoMsgUpdateDid;
use super::super::super::super::CheqdProtoBase;
use super::super::models::SignInfo;
use super::MsgUpdateDidPayload;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct MsgUpdateDid {
    pub payload: Option<MsgUpdateDidPayload>,
    pub signatures: Vec<SignInfo>,
}

#[cfg(test)]
impl MsgUpdateDid {
    pub fn new(
        payload: Option<MsgUpdateDidPayload>,
    ) -> Self {
        MsgUpdateDid {
            payload,
            signatures: vec!(),
        }
    }
}

impl CheqdProtoBase for MsgUpdateDid {
    type Proto = ProtoMsgUpdateDid;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            payload: self.payload.to_proto()?,
            signatures: self.signatures.to_proto()?,
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self {
            payload: Option::<MsgUpdateDidPayload>::from_proto(&proto.payload)?,
            signatures: Vec::<SignInfo>::from_proto(&proto.signatures)?,
        })
    }
}

#[cfg(test)]
mod test {
    use super::{MsgUpdateDidPayload, MsgUpdateDid};
    use super::super::{VerificationMethod, Service};
    use super::super::super::super::super::CheqdProtoBase;
    use std::collections::HashMap;

    #[test]
    fn test_msg_update_did() {
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

        let payload = MsgUpdateDidPayload::new(
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
            "version_1".to_string()
        );

        let msg = MsgUpdateDid::new(Some(payload));

        let proto = msg.to_proto().unwrap();
        let decoded = MsgUpdateDid::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}
