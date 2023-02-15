use std::str::FromStr;

use crate::error::MsgTypeError;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Prefix {
    DidCommOrg,
    DidSov,
}

impl Prefix {
    pub const DID_COM_ORG_PREFIX: &'static str = "https://didcomm.org";
    pub const DID_SOV_PREFIX: &'static str = "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec";
}

impl Default for Prefix {
    fn default() -> Self {
        Self::DidCommOrg
    }
}

// Manual impl so we can reuse the constants.
impl AsRef<str> for Prefix {
    fn as_ref(&self) -> &str {
        match self {
            Prefix::DidCommOrg => Prefix::DID_COM_ORG_PREFIX,
            Prefix::DidSov => Prefix::DID_SOV_PREFIX,
        }
    }
}

// Manual impl so we can reuse the constants.
impl FromStr for Prefix {
    type Err = MsgTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Prefix::DID_COM_ORG_PREFIX => Ok(Prefix::DidCommOrg),
            Prefix::DID_SOV_PREFIX => Ok(Prefix::DidSov),
            _ => Err(MsgTypeError::unknown_prefix(s.to_owned())),
        }
    }
}
