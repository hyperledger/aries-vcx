use cosmrs::proto::cosmos::bank::v1beta1::QueryBalanceResponse as ProtoQueryBalanceResponse;

use indy_api_types::errors::IndyResult;

use super::super::CheqdProtoBase;
use super::super::bank::Coin;

/// QueryBalanceResponse is the response type for the Query/Balance RPC method.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct QueryBalanceResponse {
    pub balance: Option<Coin>,
}

impl QueryBalanceResponse {
    pub fn new(
        balance: Option<Coin>,
    ) -> Self {
        QueryBalanceResponse {
            balance,
        }
    }
}

impl CheqdProtoBase for QueryBalanceResponse {
    type Proto = ProtoQueryBalanceResponse;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            balance: self.balance.to_proto()?
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            Option::<Coin>::from_proto(&proto.balance)?
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_query_balance_response() {
        let msg = QueryBalanceResponse::new(None);

        let proto = msg.to_proto().unwrap();
        let decoded = QueryBalanceResponse::from_proto(&proto).unwrap();

        assert_eq!(msg, decoded);
    }
}