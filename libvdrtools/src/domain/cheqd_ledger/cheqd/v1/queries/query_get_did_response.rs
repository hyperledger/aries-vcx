use indy_api_types::errors::IndyResult;

use super::super::models::{Did, Metadata};
use super::super::super::super::proto::cheqdid::cheqdnode::cheqd::v1::QueryGetDidResponse as ProtoQueryGetDidResponse;
use super::super::super::super::CheqdProtoBase;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct QueryGetDidResponse {
    pub did: Option<Did>,
    pub metadata: Option<Metadata>,
}

impl QueryGetDidResponse {
    pub fn new(did: Option<Did>, metadata: Option<Metadata>) -> Self {
        QueryGetDidResponse { did, metadata }
    }
}

impl CheqdProtoBase for QueryGetDidResponse {
    type Proto = ProtoQueryGetDidResponse;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            did: self.did.to_proto()?,
            metadata: self.metadata.to_proto()?,
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            Option::<Did>::from_proto(&proto.did)?,
            Option::<Metadata>::from_proto(&proto.metadata)?
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::super::models::{VerificationMethod, Service};
    use std::collections::HashMap;

    #[test]
    fn test_query_get_did_response() {
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

        let did_data = Did::new(
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

        let metadata = Metadata::new(
            "created".into(),
            "updated".into(),
            true,
            "version_id".into());

        let msg = QueryGetDidResponse::new(
            Some(did_data),
            Some(metadata)
        );

        let proto = msg.to_proto().unwrap();
        let decoded = QueryGetDidResponse::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}
