use cosmrs::proto::cosmos::auth::v1beta1::QueryAccountRequest as ProtoQueryAccountRequest;
use indy_api_types::errors::IndyResult;

use super::super::CheqdProtoBase;

/// QueryAccountRequest is the request type for the Query/Account RPC method.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct QueryAccountRequest {
    /// address defines the address to query for.
    pub address: String,
}

impl QueryAccountRequest {
    pub fn new(address: String) -> Self {
        QueryAccountRequest { address }
    }
}

impl CheqdProtoBase for QueryAccountRequest {
    type Proto = ProtoQueryAccountRequest;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            address: self.address.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(proto.address.clone()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_query_account_request() {
        let msg =
            QueryAccountRequest::new("cheqd1rnr5jrt4exl0samwj0yegv99jeskl0hsxmcz96".to_string());

        let proto = msg.to_proto().unwrap();
        let decoded = QueryAccountRequest::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}
