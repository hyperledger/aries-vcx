use super::super::CheqdProtoBase;
use indy_api_types::errors::IndyResult;

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Any {
    pub type_url: String,
    pub value: Vec<u8>
}

impl CheqdProtoBase for Any {
    type Proto = ::prost_types::Any;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            type_url: self.type_url.clone(),
            value: self.value.clone()
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self{
            type_url: proto.type_url.clone(),
            value: proto.value.clone()
        })
    }
}
