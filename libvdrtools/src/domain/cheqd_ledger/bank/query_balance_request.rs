use cosmrs::proto::cosmos::bank::v1beta1::QueryBalanceRequest as ProtoQueryBalanceRequest;

use indy_api_types::errors::IndyResult;

use super::super::CheqdProtoBase;

/// QueryBalanceRequest is the request type for the Query/Balance RPC method.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct QueryBalanceRequest {
    pub address: String,
    pub denom: String,
}

impl QueryBalanceRequest {
    pub fn new(
        address: String,
        denom: String,
    ) -> Self {
        QueryBalanceRequest {
            address,
            denom,
        }
    }
}

impl CheqdProtoBase for QueryBalanceRequest {
    type Proto = ProtoQueryBalanceRequest;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            address: self.address.clone(),
            denom: self.denom.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            proto.address.clone(),
            proto.denom.clone(),
        ))
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::environment;

    #[test]
    fn test_query_balance() {
        let msg = QueryBalanceRequest::new(
            "cheqd1rnr5jrt4exl0samwj0yegv99jeskl0hsxmcz96".to_string(),
            environment::cheqd_denom(),
        );

        let proto = msg.to_proto().unwrap();
        let decoded = QueryBalanceRequest::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}