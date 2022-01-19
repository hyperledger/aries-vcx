use indy_api_types::errors::IndyResult;

use super::super::super::super::proto::cheqdid::cheqdnode::cheqd::v1::QueryGetDidRequest as ProtoQueryGetDidRequest;
use super::super::super::super::CheqdProtoBase;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct QueryGetDidRequest {
    pub id: String,
}

#[cfg(test)]
impl QueryGetDidRequest {
    pub fn new(id: String) -> Self {
        QueryGetDidRequest { id }
    }
}

impl CheqdProtoBase for QueryGetDidRequest {
    type Proto = ProtoQueryGetDidRequest;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            id: self.id.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self {
            id: proto.id.clone(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::QueryGetDidRequest;
    use super::super::super::super::super::CheqdProtoBase;

    #[test]
    fn test_query_get_did_request() {
        let msg = QueryGetDidRequest::new("456".into());

        let proto = msg.to_proto().unwrap();
        let decoded = QueryGetDidRequest::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}
