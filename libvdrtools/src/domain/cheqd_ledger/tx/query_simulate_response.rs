use cosmrs::proto::cosmos::tx::v1beta1::SimulateResponse;

use indy_api_types::errors::IndyResult;

use super::super::CheqdProtoBase;

use super::super::base::abci::GasInfo;
use super::super::base::abci::Result;

/// QueryGasRequest is the request type for the Service/Simulate RPC method.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct QuerySimulateResponse {
    pub gas_info: Option<GasInfo>,
    pub result: Option<Result>
}

impl QuerySimulateResponse {
    pub fn new(
        gas_info: Option<GasInfo>,
        result: Option<Result>
    ) -> Self {
        QuerySimulateResponse {
            gas_info,
            result
        }
    }
}

impl CheqdProtoBase for QuerySimulateResponse {
    type Proto = SimulateResponse;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        let gas_info = self
            .gas_info
            .as_ref()
            .map(|d| d.to_proto())
            .transpose()?;
     
        let result = self
            .result
            .as_ref()
            .map(|d| d.to_proto())
            .transpose()?;

        Ok(Self::Proto { gas_info, result })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        let gas_info = proto
            .gas_info
            .as_ref()
            .map(|p| GasInfo::from_proto(p))
            .transpose()?;

        let result = proto
            .result
            .as_ref()
            .map(|p| Result::from_proto(p))
            .transpose()?;
    
        Ok(Self::new(gas_info, result))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_query_simulate_response() {
        let gas_info = GasInfo::new(123,456);
        let query = QuerySimulateResponse::new(Some(gas_info), None);

        let proto = query.to_proto().unwrap();
        let decoded = QuerySimulateResponse::from_proto(&proto).unwrap();

        assert_eq!(query, decoded);
    }
}