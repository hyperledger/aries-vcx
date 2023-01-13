use crate::utils::qualifier;
use indy_api_types::{
    errors::{IndyError, IndyErrorKind, IndyResult},
    validation::Validatable,
};
use lazy_static::lazy_static;
use regex::Regex;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DidMethod(pub String);

impl Validatable for DidMethod {
    fn validate(&self) -> Result<(), String> {
        lazy_static! {
            static ref REGEX_METHOD_NAME: Regex = Regex::new("^[a-z0-9:]+$").unwrap();
        }
        if !REGEX_METHOD_NAME.is_match(&self.0) {
            return Err(format!(
                "Invalid default name: {}. It does not match the DID method name format.",
                self.0
            ));
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MyDidInfo {
    pub did: Option<DidValue>,
    pub seed: Option<String>,
    pub crypto_type: Option<String>,
    pub cid: Option<bool>,
    pub method_name: Option<DidMethod>,
    pub ledger_type: Option<String>,
}

impl Validatable for MyDidInfo {
    fn validate(&self) -> Result<(), String> {
        if let Some(ref did) = self.did {
            did.validate()?;
        }
        if let Some(ref name) = self.method_name {
            name.validate()?
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TheirDidInfo {
    pub did: DidValue,
    pub verkey: Option<String>,
}

impl TheirDidInfo {
    pub fn new(did: DidValue, verkey: Option<String>) -> TheirDidInfo {
        TheirDidInfo { did, verkey }
    }
}

impl Validatable for TheirDidInfo {
    fn validate(&self) -> Result<(), String> {
        self.did.validate()?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Did {
    pub did: DidValue,
    pub verkey: String,
}

impl Did {
    pub fn new(did: DidValue, verkey: String) -> Did {
        Did { did, verkey }
    }
}

qualifiable_type!(DidValue);

impl DidValue {
    pub const PREFIX: &'static str = "did";

    pub fn new(did: &str, ledger_type: Option<&str>, method: Option<&str>) -> IndyResult<DidValue> {
        match (ledger_type, method) {
            (Some(ledger_type_), Some(method_)) => {
                Ok(DidValue(did.to_string()).set_ledger_and_method(ledger_type_, method_))
            }
            (None, Some(method_)) => Ok(DidValue(did.to_string()).set_method(&method_)),
            (None, None) => Ok(DidValue(did.to_string())),
            (Some(_), None) => Err(IndyError::from_msg(
                IndyErrorKind::InvalidStructure,
                "Ledger type can not be specified if DID method is undefined",
            )),
        }
    }

    pub fn to_short(&self) -> ShortDidValue {
        ShortDidValue(self.to_unqualified().0)
    }

    pub fn qualify(&self, method: &str) -> DidValue {
        self.set_method(&method)
    }

    pub fn to_unqualified(&self) -> DidValue {
        DidValue(qualifier::to_unqualified(&self.0))
    }

    pub fn is_abbreviatable(&self) -> bool {
        match self.get_method() {
            Some(ref method) if method.starts_with("sov") || method.starts_with("indy") => true,
            Some(_) => false,
            None => true,
        }
    }
}

impl Validatable for DidValue {
    fn validate(&self) -> Result<(), String> {
        if self.is_fully_qualified() {
            // pass
        } else {
            let did = bs58::decode(&self.0)
                .into_vec()
                .map_err(|err| err.to_string())?;

            if did.len() != 16 && did.len() != 32 {
                return Err(format!("Trying to use DID with unexpected length: {}. \
                               The 16- or 32-byte number upon which a DID is based should be 22/23 or 44/45 bytes when encoded as base58.", did.len()));
            }
        }
        Ok(())
    }
}

qualifiable_type!(ShortDidValue);

impl ShortDidValue {
    pub const PREFIX: &'static str = "did";

    pub fn qualify(&self, method: Option<String>) -> DidValue {
        match method {
            Some(method_) => DidValue(self.set_method(&method_).0),
            None => DidValue(self.0.to_string()),
        }
    }
}

impl Validatable for ShortDidValue {
    fn validate(&self) -> Result<(), String> {
        let did = bs58::decode(&self.0)
            .into_vec()
            .map_err(|err| err.to_string())?;

        if did.len() != 16 && did.len() != 32 {
            return Err(format!("Trying to use DID with unexpected length: {}. \
                               The 16- or 32-byte number upon which a DID is based should be 22/23 or 44/45 bytes when encoded as base58.", did.len()));
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DidMetadata {
    pub value: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DidWithMeta {
    pub did: DidValue,
    pub verkey: String,
    pub temp_verkey: Option<String>,
    pub metadata: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TheirDid {
    pub did: DidValue,
    pub verkey: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TemporaryDid {
    pub did: DidValue,
    pub verkey: String,
}

impl From<TemporaryDid> for Did {
    fn from(temp_did: TemporaryDid) -> Self {
        Did {
            did: temp_did.did,
            verkey: temp_did.verkey,
        }
    }
}
