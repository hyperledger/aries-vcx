use indy_api_types::errors::IndyResult;

use super::super::super::super::proto::cheqdid::cheqdnode::cheqd::v1::MsgUpdateDidPayload as ProtoMsgUpdateDidPayload;
use super::super::super::super::CheqdProtoBase;
use super::VerificationMethod;
use super::Service;

#[derive(Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct MsgUpdateDidPayload {
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub context: Vec<String>,
    pub id: String,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub controller: Vec<String>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub verification_method: Vec<VerificationMethod>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub authentication: Vec<String>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub assertion_method: Vec<String>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub capability_invocation: Vec<String>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub capability_delegation: Vec<String>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub key_agreement: Vec<String>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub service: Vec<Service>,
    #[serde(skip_serializing_if="Vec::is_empty")]
    #[serde(default)]
    pub also_known_as: Vec<String>,
    pub version_id: String,
}

impl MsgUpdateDidPayload {
    pub fn new(
        context: Vec<String>,
        id: String,
        controller: Vec<String>,
        verification_method: Vec<VerificationMethod>,
        authentication: Vec<String>,
        assertion_method: Vec<String>,
        capability_invocation: Vec<String>,
        capability_delegation: Vec<String>,
        key_agreement: Vec<String>,
        service: Vec<Service>,
        also_known_as: Vec<String>,
        version_id: String)-> Self {
        MsgUpdateDidPayload {
            context,
            id,
            controller,
            verification_method,
            authentication,
            assertion_method,
            capability_invocation,
            capability_delegation,
            key_agreement,
            service,
            also_known_as,
            version_id
        }
    }
}

impl CheqdProtoBase for MsgUpdateDidPayload {
    type Proto = ProtoMsgUpdateDidPayload;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(
            Self::Proto {
                context: self.context.to_proto()?,
                id: self.id.clone(),
                controller: self.controller.to_proto()?,
                verification_method: self.verification_method.to_proto()?,
                authentication: self.authentication.to_proto()?,
                assertion_method: self.assertion_method.to_proto()?,
                capability_invocation: self.capability_invocation.to_proto()?,
                capability_delegation: self.capability_delegation.to_proto()?,
                key_agreement: self.key_agreement.to_proto()?,
                service: self.service.to_proto()?,
                also_known_as: self.also_known_as.to_proto()?,
                version_id: self.version_id.clone(),
            }
        )
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            proto.context.clone(),
            proto.id.clone(),
            proto.controller.clone(),
            Vec::<VerificationMethod>::from_proto(&proto.verification_method)?,
            proto.authentication.clone(),
            proto.assertion_method.clone(),
            proto.capability_invocation.clone(),
            proto.capability_delegation.clone(),
            proto.key_agreement.clone(),
            Vec::<Service>::from_proto(&proto.service)?,
            proto.also_known_as.clone(),
            proto.version_id.clone(),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::{MsgUpdateDidPayload, VerificationMethod, Service};
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

        let msg = MsgUpdateDidPayload::new(
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
            "version_id".to_string(),
        );

        let proto = msg.to_proto().unwrap();
        let decoded = MsgUpdateDidPayload::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}
