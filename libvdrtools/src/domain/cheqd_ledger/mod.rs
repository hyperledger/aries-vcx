use std::fmt::Debug;

use indy_api_types::errors::IndyResult;

use prost_ext::ProstMessageExt;

pub mod cosmos_ext;
pub mod prost_ext;
pub mod proto;
pub mod cheqd;
pub mod bank;
pub mod auth;
pub mod base;
pub mod crypto;
pub mod tx;
pub mod vesting;
mod tests;
pub mod prost_types;
pub mod abci_info;

pub trait CheqdProtoBase: Eq + Debug + Sized{
    type Proto;

    fn to_proto(&self) -> IndyResult<Self::Proto>;
    fn from_proto(proto: &Self::Proto) -> IndyResult<Self>;
}

pub trait CheqdProto: CheqdProtoBase {
    fn to_proto_bytes(&self) -> IndyResult<Vec<u8>>;
    fn from_proto_bytes(bytes: &[u8]) -> IndyResult<Self>;
}

impl<T> CheqdProto for T where T: CheqdProtoBase, <T as CheqdProtoBase>::Proto: prost::Message + Default {
    fn to_proto_bytes(&self) -> IndyResult<Vec<u8>> {
        Ok(self.to_proto()?.to_bytes()?)
    }

    fn from_proto_bytes(bytes: &[u8]) -> IndyResult<Self> {
        let proto = Self::Proto::from_bytes(bytes)?;
        Ok(Self::from_proto(&proto)?)
    }
}

impl<T> CheqdProtoBase for Vec<T> where T: CheqdProtoBase {
    type Proto = Vec<T::Proto>;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        self.iter().map(|i| i.clone().to_proto()).collect::<IndyResult<Self::Proto>>()
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        proto.iter().map(|i| T::from_proto(i)).collect::<IndyResult<Self>>()
    }
}

impl<T> CheqdProtoBase for Option<T> where T: CheqdProtoBase {
    type Proto = Option<T::Proto>;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        self.as_ref().map(|i| i.clone().to_proto()).transpose()
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(proto.as_ref().map(|i| T::from_proto(&i)).transpose()?)
    }
}

impl CheqdProtoBase for String {
    type Proto = String;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(self.clone())
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(proto.clone())
    }
}


pub trait ToSignBytesBase: Eq + Debug + Sized{

    fn to_sign_bytes(&self) -> IndyResult<Vec<u8>>;
}
