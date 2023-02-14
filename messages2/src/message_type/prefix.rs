use std::str::FromStr;

use crate::error::MsgTypeError;

pub enum Prefix {
    DidCommOrg,
    DidSov,
}

impl Prefix {
    pub const DID_COM_ORG_PREFIX: &'static str = "https://didcomm.org";
    pub const DID_SOV_PREFIX: &'static str = "did:sov:BzCbsNYhMrjHiqZDTUASHg";
}

impl Default for Prefix {
    fn default() -> Self {
        Self::DidCommOrg
    }
}

impl AsRef<str> for Prefix {
    fn as_ref(&self) -> &str {
        match self {
            Prefix::DidCommOrg => Prefix::DID_COM_ORG_PREFIX,
            Prefix::DidSov => Prefix::DID_SOV_PREFIX,
        }
    }
}

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

// impl Serialize for Prefix {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         serializer.serialize_str(self.as_ref())
//     }
// }

// impl<'de> Deserialize<'de> for Prefix {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let s = <&str>::deserialize(deserializer)?;
//         Self::from_str(s).map_err(D::Error::custom)
//     }
// }
