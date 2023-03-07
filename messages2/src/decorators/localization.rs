use std::collections::HashMap;

use isolang::Language;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

/// We need to wrap this as the default serde
/// behavior is to use ISO 639-3 codes and we need ISO 639-2;
#[derive(Copy, Clone, Debug, PartialEq, Hash, Eq)]
#[repr(transparent)]
pub struct Locale(pub Language);

impl Default for Locale {
    fn default() -> Self {
        Self(Language::Eng)
    }
}

impl Serialize for Locale {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.to_639_1().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Locale {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let code = <&str>::deserialize(deserializer)?;
        let lang = Language::from_639_1(code).ok_or_else(|| D::Error::custom(format!("unknown locale {code}")))?;
        Ok(Locale(lang))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct MsgLocalization {
    pub catalogs: Option<Vec<Url>>,
    #[serde(alias = "details")]
    // Might just be obsolete, but appears in https://github.com/hyperledger/aries-rfcs/blob/main/features/0043-l10n/README.md
    pub locales: Option<HashMap<Locale, Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldLocalization {
    pub code: Option<String>,
    pub locale: Option<Locale>,
    pub catalogs: Option<Vec<Url>>,
    #[serde(flatten)]
    pub translations: HashMap<Locale, String>,
}
