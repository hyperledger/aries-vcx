use cosmrs::proto::cosmos::base::abci::v1beta1::GasInfo as ProtoGasInfo;
use indy_api_types::errors::IndyResult;

use super::super::super::CheqdProtoBase;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct GasInfo {
    pub gas_wanted: u64,
    pub gas_used: u64,
}

impl GasInfo {
    pub fn new(gas_wanted: u64, gas_used: u64) -> Self {
        GasInfo {
            gas_wanted,
            gas_used
        }
    }
}

impl CheqdProtoBase for GasInfo {
    type Proto = ProtoGasInfo;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            gas_wanted: self.gas_wanted.clone(),
            gas_used: self.gas_used.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            proto.gas_wanted.clone(),
            proto.gas_used.clone(),
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_query_gas_info() {
        let query = GasInfo::new(123,456);

        let proto = query.to_proto().unwrap();
        let decoded = GasInfo::from_proto(&proto).unwrap();

        assert_eq!(query, decoded);
    }
}
