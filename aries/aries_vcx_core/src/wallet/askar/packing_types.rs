use indy_vdr::utils::base64::{decode_urlsafe, encode_urlsafe};
use serde::{de::Unexpected, Deserialize, Serialize};

use crate::{
    errors::error::{AriesVcxCoreError, AriesVcxCoreErrorKind, VcxCoreResult},
    wallet::askar::askar_utils::bytes_to_string,
};

pub const PROTECTED_HEADER_ENC: &str = "xchacha20poly1305_ietf";
pub const PROTECTED_HEADER_TYP: &str = "JWM/1.0";

#[derive(Debug)]
pub enum ProtectedHeaderEnc {
    XChaCha20Poly1305,
}

impl Serialize for ProtectedHeaderEnc {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            Self::XChaCha20Poly1305 => PROTECTED_HEADER_ENC,
        })
    }
}

impl<'de> Deserialize<'de> for ProtectedHeaderEnc {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        match value.as_str() {
            PROTECTED_HEADER_ENC => Ok(Self::XChaCha20Poly1305),
            _ => Err(serde::de::Error::invalid_value(
                Unexpected::Str(value.as_str()),
                &PROTECTED_HEADER_ENC,
            )),
        }
    }
}

#[derive(Debug)]
pub enum ProtectedHeaderTyp {
    Jwm,
}

impl Serialize for ProtectedHeaderTyp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            Self::Jwm => PROTECTED_HEADER_TYP,
        })
    }
}

impl<'de> Deserialize<'de> for ProtectedHeaderTyp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;

        match value.as_str() {
            PROTECTED_HEADER_TYP => Ok(Self::Jwm),
            _ => Err(serde::de::Error::invalid_value(
                Unexpected::Str(value.as_str()),
                &PROTECTED_HEADER_TYP,
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(transparent)]
pub struct Base64String(String);

impl Base64String {
    pub fn from_bytes(content: &[u8]) -> Self {
        Self(encode_urlsafe(content))
    }

    pub fn decode(&self) -> VcxCoreResult<Vec<u8>> {
        decode_urlsafe(&self.0)
            .map_err(|e| AriesVcxCoreError::from_msg(AriesVcxCoreErrorKind::InvalidJson, e))
    }

    pub fn decode_to_string(&self) -> VcxCoreResult<String> {
        bytes_to_string(self.decode()?)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().into()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Jwe {
    pub protected: Base64String,
    pub iv: Base64String,
    pub ciphertext: Base64String,
    pub tag: Base64String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum JweAlg {
    Authcrypt,
    Anoncrypt,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProtectedData {
    pub enc: ProtectedHeaderEnc,
    pub typ: ProtectedHeaderTyp,
    pub alg: JweAlg,
    pub recipients: Vec<Recipient>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Recipient {
    Authcrypt(AuthcryptRecipient),
    Anoncrypt(AnoncryptRecipient),
}

impl Recipient {
    pub fn new_authcrypt(
        encrypted_key: Base64String,
        kid: &str,
        iv: Base64String,
        sender: Base64String,
    ) -> Self {
        Self::Authcrypt(AuthcryptRecipient {
            encrypted_key,
            header: AuthcryptHeader {
                kid: kid.into(),
                iv,
                sender,
            },
        })
    }

    pub fn new_anoncrypt(encrypted_key: Base64String, kid: &str) -> Self {
        Self::Anoncrypt(AnoncryptRecipient {
            encrypted_key,
            header: AnoncryptHeader { kid: kid.into() },
        })
    }

    pub fn unwrap_kid(&self) -> &str {
        match self {
            Self::Anoncrypt(inner) => &inner.header.kid,
            Self::Authcrypt(inner) => &inner.header.kid,
        }
    }

    pub fn key_name(&self) -> &str {
        &self.unwrap_kid()[0..16]
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthcryptRecipient {
    pub encrypted_key: Base64String,
    pub header: AuthcryptHeader,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnoncryptRecipient {
    pub encrypted_key: Base64String,
    pub header: AnoncryptHeader,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthcryptHeader {
    pub kid: String,
    pub iv: Base64String,
    pub sender: Base64String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnoncryptHeader {
    pub kid: String,
}
