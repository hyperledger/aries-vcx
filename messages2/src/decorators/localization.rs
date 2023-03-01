use std::collections::HashMap;

use isolang::Language;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

use super::EmptyDecorator;

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
    catalogs: Option<Vec<Url>>,
    #[serde(alias = "details")]
    // Might just be obsolete, but appears in https://github.com/hyperledger/aries-rfcs/blob/main/features/0043-l10n/README.md
    locales: Option<HashMap<Locale, Vec<String>>>,
}

impl EmptyDecorator for MsgLocalization {
    fn is_empty(&self) -> bool {
        self.catalogs.is_none() && self.locales.as_ref().map(|h| h.is_empty()).unwrap_or(true)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldLocalization {
    code: Option<String>,
    locale: Option<Locale>,
    catalogs: Option<Vec<Url>>,
    #[serde(flatten)]
    translations: HashMap<Locale, String>,
}

impl EmptyDecorator for FieldLocalization {
    fn is_empty(&self) -> bool {
        self.code.is_none() && self.locale.is_none() && self.catalogs.is_none() && self.translations.is_empty()
    }
}
