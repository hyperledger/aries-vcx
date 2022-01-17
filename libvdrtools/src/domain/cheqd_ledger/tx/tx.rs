use indy_api_types::errors::IndyResult;

use cosmrs::proto::cosmos::tx::v1beta1::Tx as ProtoTx;

use super::super::super::cheqd_ledger::CheqdProtoBase;
use super::TxBody;
use super::AuthInfo;

/// Tx is the standard type used for broadcasting transactions.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Tx {
    /// body is the processable content of the transaction
    pub body: Option<TxBody>,

    /// auth_info is the authorization related content of the transaction,
    /// specifically signers, signer modes and fee
    pub auth_info: Option<AuthInfo>,

    /// signatures is a list of signatures that matches the length and order of
    /// AuthInfo's signer_infos to allow connecting signature meta information like
    /// public key and signing mode by position.
    pub signatures: Vec<Vec<u8>>,
}

impl Tx {
    pub fn new(
        body: Option<TxBody>,
        auth_info: Option<AuthInfo>,
        signatures: Vec<Vec<u8>>,
    ) -> Self {
        Tx {
            body,
            auth_info,
            signatures,
        }
    }
}

impl CheqdProtoBase for Tx {
    type Proto = ProtoTx;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            body: self.body.to_proto()?,
            auth_info: self.auth_info.to_proto()?,
            signatures: self.signatures.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            Option::<TxBody>::from_proto(&proto.body)?,
            Option::<AuthInfo>::from_proto(&proto.auth_info)?,
            proto.signatures.clone(),
        ))
    }
}
