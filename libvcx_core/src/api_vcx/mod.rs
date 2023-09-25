#[macro_use]
pub mod utils;
pub mod api_global;
pub mod api_handle;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Default)]
#[serde(try_from = "u8")]
#[repr(u8)]
pub enum VcxStateType {
    #[default]
    VcxStateNone = 0,
    VcxStateInitialized = 1,
    VcxStateOfferSent = 2,
    VcxStateRequestReceived = 3,
    VcxStateAccepted = 4,
    VcxStateUnfulfilled = 5,
    VcxStateExpired = 6,
    VcxStateRevoked = 7,
    VcxStateRedirected = 8,
    VcxStateRejected = 9,
}
impl serde::Serialize for VcxStateType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl TryFrom<u8> for VcxStateType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(VcxStateType::VcxStateNone),
            1 => Ok(VcxStateType::VcxStateInitialized),
            2 => Ok(VcxStateType::VcxStateOfferSent),
            3 => Ok(VcxStateType::VcxStateRequestReceived),
            4 => Ok(VcxStateType::VcxStateAccepted),
            5 => Ok(VcxStateType::VcxStateUnfulfilled),
            6 => Ok(VcxStateType::VcxStateExpired),
            7 => Ok(VcxStateType::VcxStateRevoked),
            8 => Ok(VcxStateType::VcxStateRedirected),
            9 => Ok(VcxStateType::VcxStateRejected),
            _ => Err(format!(
                "unknown {} value: {}",
                stringify!(VcxStateType),
                value
            )),
        }
    }
}

// undefined is correlated with VcxStateNon -> Haven't received Proof
// Validated is both validated by indy-sdk and by comparing proof-request
// Invalid is that it failed one or both of validation processes
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
#[serde(try_from = "u8")]
#[repr(u8)]
pub enum ProofStateType {
    ProofUndefined = 0,
    ProofValidated = 1,
    ProofInvalid = 2,
}
impl ::serde::Serialize for ProofStateType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

impl TryFrom<u8> for ProofStateType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ProofStateType::ProofUndefined),
            1 => Ok(ProofStateType::ProofValidated),
            2 => Ok(ProofStateType::ProofInvalid),
            _ => Err(format!(
                "unknown {} value: {}",
                stringify!(ProofStateType),
                value
            )),
        }
    }
}

#[repr(C)]
pub struct VcxStatus {
    pub handle: libc::c_int,
    pub status: libc::c_int,
    pub msg: *mut libc::c_char,
}

#[cfg(test)]
mod tests {
    use serde_json;

    use self::VcxStateType::*;
    use super::*;

    #[test]
    fn test_serialize_vcx_state_type() {
        let z = VcxStateNone;
        let y = serde_json::to_string(&z).unwrap();
        assert_eq!(y, "0");
    }
}
