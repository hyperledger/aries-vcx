use std::collections::HashMap;

use isolang::Language;
use serde::{
    ser::{Error, SerializeMap},
    Deserialize, Serialize, Serializer,
};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct MsgLocalization {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalogs: Option<Vec<Url>>,
    // Might just be obsolete, but appears in https://github.com/hyperledger/aries-rfcs/blob/main/features/0043-l10n/README.md
    // Is details and locales the same thing?
    #[serde(alias = "details")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locales: Option<HashMap<Locale, Vec<String>>>,
}

#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
pub struct FieldLocalization {
    pub code: Option<String>,
    pub locale: Option<Locale>,
    pub catalogs: Option<Vec<Url>>,
    #[serde(flatten)]
    pub translations: HashMap<Locale, String>,
}

/// Manual implementation because `serde_json` does not support
/// non-string map keys.
impl Serialize for FieldLocalization {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = Serializer::serialize_map(serializer, None)?;

        if !Option::is_none(&self.code) {
            state.serialize_entry("code", &self.code)?;
        }
        if !Option::is_none(&self.locale) {
            state.serialize_entry("locale", &self.locale)?;
        }
        if !Option::is_none(&self.catalogs) {
            state.serialize_entry("catalogs", &self.catalogs)?;
        }
        if !HashMap::is_empty(&self.translations) {
            for (key, value) in &self.translations {
                let key = <&str>::try_from(key).map_err(S::Error::custom)?;
                state.serialize_entry(key, value)?;
            }
        }
        SerializeMap::end(state)
    }
}

/// We need to wrap this as the default serde
/// behavior is to use ISO 639-3 codes and we need ISO 639-1;
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Hash, Eq)]
#[repr(transparent)]
#[serde(try_from = "&str")]
pub struct Locale(pub Language);

impl Default for Locale {
    fn default() -> Self {
        Self(Language::Eng)
    }
}

impl<'a> TryFrom<&'a Locale> for &'a str {
    type Error = String;

    fn try_from(value: &'a Locale) -> Result<Self, Self::Error> {
        value
            .0
            .to_639_1()
            .ok_or_else(|| format!("{} has no ISO 639-1 code", value.0))
    }
}

impl<'a> TryFrom<&'a str> for Locale {
    type Error = String;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let lang = Language::from_639_1(value).ok_or_else(|| format!("unknown locale {value}"))?;
        Ok(Locale(lang))
    }
}

impl Serialize for Locale {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let locale_str = <&str>::try_from(self).map_err(S::Error::custom)?;
        locale_str.serialize(serializer)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::{json, Value};

    use super::*;
    use crate::misc::test_utils;

    pub fn make_minimal_field_localization() -> FieldLocalization {
        FieldLocalization::default()
    }

    pub fn make_extended_field_localization() -> FieldLocalization {
        let code = "test_code".to_owned();
        let locale = Locale::default();
        let catalogs = vec!["https://dummy.dummy/dummy".parse().unwrap()];
        let translations = HashMap::from([(Locale(Language::Fra), "test, but in french".to_owned())]);

        let mut localization = FieldLocalization::default();
        localization.code = Some(code);
        localization.locale = Some(locale);
        localization.catalogs = Some(catalogs);
        localization.translations = translations;

        localization
    }

    pub fn make_minimal_msg_localization() -> MsgLocalization {
        MsgLocalization::default()
    }

    pub fn make_extended_msg_localization() -> MsgLocalization {
        let catalogs = vec!["https://dummy.dummy/dummy".parse().unwrap()];
        let locales = HashMap::from([(Locale(Language::Fra), vec!["test, but in french".to_owned()])]);

        let mut localization = MsgLocalization::default();
        localization.catalogs = Some(catalogs);
        localization.locales = Some(locales);

        localization
    }

    #[test]
    fn test_minimal_decorator_field() {
        let localization = make_minimal_field_localization();
        let json = json!({});

        test_utils::test_serde(localization, json);
    }

    #[test]
    fn test_extensive_decorator_field() {
        let localization = make_extended_field_localization();

        let mut json = json!({
            "code": localization.code,
            "locale": localization.locale,
            "catalogs": localization.catalogs,
        });

        let map = json.as_object_mut().unwrap();
        for (key, value) in &localization.translations {
            let key = <&str>::try_from(key).unwrap().to_owned();
            let value = Value::String(value.to_owned());
            map.insert(key, value);
        }

        test_utils::test_serde(localization, json);
    }

    #[test]
    fn test_minimal_decorator_msg() {
        let localization = make_minimal_msg_localization();
        let json = json!({});

        test_utils::test_serde(localization, json);
    }

    #[test]
    fn test_extensive_decorator_msg() {
        let localization = make_extended_msg_localization();

        let json = json!({
            "catalogs": localization.catalogs,
            "locales": localization.locales
        });

        test_utils::test_serde(localization, json);
    }
}
