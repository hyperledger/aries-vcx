use indy_api_types::errors::IndyResult;

use cosmrs::proto::cosmos::tx::v1beta1::AuthInfo as ProtoAuthInfo;

use super::super::super::cheqd_ledger::CheqdProtoBase;
use super::SignerInfo;
use super::Fee;

/// AuthInfo describes the fee and signer modes that are used to sign a
/// transaction.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct AuthInfo {
    /// signer_infos defines the signing modes for the required signers. The number
    /// and order of elements must match the required signers from TxBody's
    /// messages. The first element is the primary signer and the one which pays
    /// the fee.
    pub signer_infos: Vec<SignerInfo>,

    /// Fee is the fee and gas limit for the transaction. The first signer is the
    /// primary signer and the one which pays the fee. The fee can be calculated
    /// based on the cost of evaluating the body and doing signature verification
    /// of the signers. This can be estimated via simulation.
    pub fee: Option<Fee>,
}

impl AuthInfo {
    pub fn new(
        signer_infos: Vec<SignerInfo>,
        fee: Option<Fee>,
    ) -> Self {
        AuthInfo {
            signer_infos,
            fee,
        }
    }
}


impl CheqdProtoBase for AuthInfo {
    type Proto = ProtoAuthInfo;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            signer_infos: self.signer_infos.to_proto()?,
            fee: self.fee.to_proto()?,
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            Vec::<SignerInfo>::from_proto(&proto.signer_infos)?,
            Option::<Fee>::from_proto(&proto.fee)?
        ))
    }
}

