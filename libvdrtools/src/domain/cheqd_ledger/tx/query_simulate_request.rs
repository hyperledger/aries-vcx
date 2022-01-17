use cosmrs::proto::cosmos::tx::v1beta1::{SimulateRequest};

use indy_api_types::errors::{IndyResult};

use super::{super::{ CheqdProtoBase, CheqdProto }, Tx};

/// QueryGasRequest is the request type for the Service/Simulate RPC method.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct QuerySimulateRequest {
    pub tx: Option<Tx>,
}

impl QuerySimulateRequest {
    pub fn new(
        tx_bytes: &[u8],
    ) -> IndyResult<Self> {
        let tx = Tx::from_proto_bytes(tx_bytes)?;
        Ok(QuerySimulateRequest { tx: Some(tx) })
    }
}

impl CheqdProtoBase for QuerySimulateRequest {
    type Proto = SimulateRequest;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto { 
            tx: self
                .tx
                .as_ref()
                .map(|p| p.to_proto())
                .transpose()?
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        let tx = proto
            .tx
            .as_ref()
            .map(|p| Tx::from_proto(p))
            .transpose()?;

        Ok(QuerySimulateRequest{ tx })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_query_simulate_request() {
        let tx_bytes = Tx::new(None, None, vec!()).to_proto_bytes().unwrap();
        let query = QuerySimulateRequest::new(&tx_bytes).unwrap();

        let proto = query.to_proto().unwrap();
        let decoded = QuerySimulateRequest::from_proto(&proto).unwrap();

        assert_eq!(query, decoded);
    }
}