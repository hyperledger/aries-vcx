use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use crate::errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};

const LINK_SECRET: &str = "VCX_LINK_SECRET";
const CRED: &str = "VCX_CREDENTIAL";
const CRED_DEF: &str = "VCX_CRED_DEF";
const CRED_KEY_CORRECTNESS_PROOF: &str = "VCX_CRED_KEY_CORRECTNESS_PROOF";
const CRED_DEF_PRIV: &str = "VCX_CRED_DEF_PRIV";
const CRED_SCHEMA: &str = "VCX_CRED_SCHEMA";
const CRED_MAP_SCHEMA_ID: &str = "VCX_CRED_MAP_SCHEMA_ID";
const REV_REG: &str = "VCX_REV_REG";
const REV_REG_DELTA: &str = "VCX_REV_REG_DELTA";
const REV_REG_INFO: &str = "VCX_REV_REG_INFO";
const REV_REG_DEF: &str = "VCX_REV_REG_DEF";
const REV_REG_DEF_PRIV: &str = "VCX_REV_REG_DEF_PRIV";
const DID: &str = "Indy::Did";
const TMP_DID: &str = "Indy::TemporaryDid";

#[derive(Clone, Copy, Debug, Default)]
pub enum RecordCategory {
    #[default]
    LinkSecret,
    Cred,
    CredDef,
    CredKeyCorrectnessProof,
    CredDefPriv,
    CredSchema,
    CredMapSchemaId,
    RevReg,
    RevRegDelta,
    RevRegInfo,
    RevRegDef,
    RevRegDefPriv,
    Did,
    TmpDid,
}

impl FromStr for RecordCategory {
    type Err = AriesVcxCoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            LINK_SECRET => Ok(RecordCategory::LinkSecret),
            CRED => Ok(RecordCategory::Cred),
            CRED_DEF => Ok(RecordCategory::CredDef),
            CRED_KEY_CORRECTNESS_PROOF => Ok(RecordCategory::CredKeyCorrectnessProof),
            CRED_DEF_PRIV => Ok(RecordCategory::CredDefPriv),
            CRED_SCHEMA => Ok(RecordCategory::CredSchema),
            CRED_MAP_SCHEMA_ID => Ok(RecordCategory::CredMapSchemaId),
            REV_REG => Ok(RecordCategory::RevReg),
            REV_REG_DELTA => Ok(RecordCategory::RevRegDelta),
            REV_REG_INFO => Ok(RecordCategory::RevRegInfo),
            REV_REG_DEF => Ok(RecordCategory::RevRegDef),
            REV_REG_DEF_PRIV => Ok(RecordCategory::RevRegDefPriv),
            DID => Ok(RecordCategory::Did),
            TMP_DID => Ok(RecordCategory::TmpDid),
            _ => Err(Self::Err::from_msg(
                AriesVcxCoreErrorKind::InvalidInput,
                format!("unknown category: {}", s),
            )),
        }
    }
}

impl Display for RecordCategory {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let value = match self {
            RecordCategory::LinkSecret => LINK_SECRET,
            RecordCategory::Cred => CRED,
            RecordCategory::CredDef => CRED_DEF,
            RecordCategory::CredKeyCorrectnessProof => CRED_KEY_CORRECTNESS_PROOF,
            RecordCategory::CredDefPriv => CRED_DEF_PRIV,
            RecordCategory::CredSchema => CRED_SCHEMA,
            RecordCategory::CredMapSchemaId => CRED_MAP_SCHEMA_ID,
            RecordCategory::RevReg => REV_REG,
            RecordCategory::RevRegDelta => REV_REG_DELTA,
            RecordCategory::RevRegInfo => REV_REG_INFO,
            RecordCategory::RevRegDef => REV_REG_DEF,
            RecordCategory::RevRegDefPriv => REV_REG_DEF_PRIV,
            RecordCategory::Did => DID,
            RecordCategory::TmpDid => TMP_DID,
        };

        write!(f, "{}", value)
    }
}
