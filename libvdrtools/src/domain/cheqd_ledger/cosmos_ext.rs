use cosmrs::proto::cosmos::tx::v1beta1::{SignDoc as ProtoSignDoc, TxRaw, Tx as ProtoTx};
use cosmrs::tx::{Msg, Raw, SignDoc};
use indy_api_types::errors::IndyResult;
use prost_types::Any;

use super::super::cheqd_ledger::prost_ext::ProstMessageExt;
use cosmrs::Tx;

pub trait CosmosMsgExt {
    fn to_bytes(&self) -> IndyResult<Vec<u8>>;
    fn from_bytes(bytes: &[u8]) -> IndyResult<Self>
        where
            Self: Sized;
}

impl CosmosMsgExt for Msg {
    fn to_bytes(&self) -> IndyResult<Vec<u8>> {
        let proto: Any = self.clone().into();
        Ok(proto.to_bytes()?)
    }

    fn from_bytes(bytes: &[u8]) -> IndyResult<Self>
        where
            Self: Sized,
    {
        let res = Any::from_bytes(bytes)?;
        Ok(res.into())
    }
}

pub trait CosmosSignDocExt {
    fn to_bytes(&self) -> IndyResult<Vec<u8>>;
    fn from_bytes(bytes: &[u8]) -> IndyResult<Self>
        where
            Self: Sized;
}

impl CosmosSignDocExt for SignDoc {
    fn to_bytes(&self) -> IndyResult<Vec<u8>> {
        let proto: ProtoSignDoc = self.clone().into();
        Ok(proto.to_bytes()?)
    }

    fn from_bytes(bytes: &[u8]) -> IndyResult<Self>
        where
            Self: Sized,
    {
        let proto = ProtoSignDoc::from_bytes(bytes)?;
        Ok(proto.into())
    }
}

pub trait CosmosTxExt {
    fn to_bytes(&self) -> IndyResult<Vec<u8>>;
    fn from_bytes(bytes: &[u8]) -> IndyResult<Self>
        where
            Self: Sized;
}

impl CosmosTxExt for Tx {
    fn to_bytes(&self) -> IndyResult<Vec<u8>> {
        let proto: ProtoTx = self.clone().into();
        Ok(proto.to_bytes()?)
    }

    fn from_bytes(bytes: &[u8]) -> IndyResult<Self> where
        Self: Sized {
        let tx = Tx::from_bytes(bytes)?;
        Ok(tx.into())
    }
}

pub trait CosmosRawExt {
    fn to_bytes(&self) -> IndyResult<Vec<u8>>;
    fn from_bytes(bytes: &[u8]) -> IndyResult<Self>
        where
            Self: Sized;
}

impl CosmosRawExt for Raw {
    fn to_bytes(&self) -> IndyResult<Vec<u8>> {
        let proto: TxRaw = self.clone().into();
        Ok(proto.to_bytes()?)
    }

    fn from_bytes(bytes: &[u8]) -> IndyResult<Self>
        where
            Self: Sized,
    {
        let proto = TxRaw::from_bytes(bytes)?;
        Ok(proto.into())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use cosmrs::tx::{Msg, MsgType};
    use super::super::super::cheqd_ledger::cheqd::v1::messages::{MsgCreateDid, MsgCreateDidPayload, VerificationMethod, Service};
    use super::super::super::cheqd_ledger::CheqdProtoBase;
    use std::collections::HashMap;

    #[test]
    fn test_cosmos_msg_ext() {
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

        let payload = MsgCreateDidPayload::new(
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

        let msg = MsgCreateDid::new(Some(payload));
        let msg = msg.to_proto().unwrap().to_msg().unwrap();

        let bytes: Vec<u8> = msg.to_bytes().unwrap();
        let decoded = Msg::from_bytes(bytes.as_slice()).unwrap();

        assert_eq!(msg, decoded);
    }
}
