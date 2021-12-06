use indy_api_types::errors::IndyResult;

use cosmrs::proto::cosmos::tx::v1beta1::SignerInfo as ProtoTx;

use super::super::super::cheqd_ledger::CheqdProtoBase;
use super::ModeInfo;
use super::super::super::cheqd_ledger::crypto::PubKey;

/// SignerInfo describes the public key and signing mode of a single top-level
/// signer.
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct SignerInfo {
    /// public_key is the public key of the signer. It is optional for accounts
    /// that already exist in state. If unset, the verifier can use the required \
    /// signer address for this position and lookup the public key.
    pub public_key: Option<PubKey>,

    /// mode_info describes the signing mode of the signer and is a nested
    /// structure to support nested multisig pubkey's
    pub mode_info: Option<ModeInfo>,

    /// sequence is the sequence of the account, which describes the
    /// number of committed transactions signed by a given address. It is used to
    /// prevent replay attacks.
    pub sequence: u64,
}

impl SignerInfo {
    pub fn new(
        public_key: Option<PubKey>,
        mode_info: Option<ModeInfo>,
        sequence: u64,
    ) -> Self {
        SignerInfo {
            public_key,
            mode_info,
            sequence,
        }
    }
}

impl CheqdProtoBase for SignerInfo {
    type Proto = ProtoTx;

    fn to_proto(&self) -> IndyResult<Self::Proto> {
        Ok(Self::Proto {
            public_key: self.public_key.to_proto()?,
            mode_info: self.mode_info.to_proto()?,
            sequence: self.sequence.clone(),
        })
    }

    fn from_proto(proto: &Self::Proto) -> IndyResult<Self> {
        Ok(Self::new(
            Option::<PubKey>::from_proto(&proto.public_key)?,
            Option::<ModeInfo>::from_proto(&proto.mode_info)?,
            proto.sequence.clone(),
        ))
    }
}
