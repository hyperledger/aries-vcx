use std::collections::HashMap;

use isolang::Language;
use serde::{
    ser::{Error, SerializeMap},
    Deserialize, Serialize, Serializer,
};
use shared_vcx::misc::utils::CowStr;
use typed_builder::TypedBuilder;
use url::Url;

/// Struct representing the `~l10n` decorator, when it decorates the entire message, from its [RFC](<https://github.com/hyperledger/aries-rfcs/blob/main/features/0043-l10n/README.md>).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, TypedBuilder)]
pub struct MsgLocalization {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalogs: Option<Vec<Url>>,
    // Might just be obsolete, but appears in <https://github.com/hyperledger/aries-rfcs/blob/main/features/0043-l10n/README.md>
    // Is details and locales the same thing?
    #[builder(default, setter(strip_option))]
    #[serde(alias = "details")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locales: Option<HashMap<Locale, Vec<String>>>,
}

/// Struct representing the `~l10n` decorator, when it decorates a single field, from its [RFC](<https://github.com/hyperledger/aries-rfcs/blob/main/features/0043-l10n/README.md>).
#[derive(Debug, Clone, Deserialize, Default, PartialEq, TypedBuilder)]
pub struct FieldLocalization {
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<Locale>,
    #[builder(default, setter(strip_option))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalogs: Option<Vec<Url>>,
    #[builder(default)]
    #[serde(flatten)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
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

/// Represents an ISO 639-1, two letter, language code.
///
/// We need to wrap this as the default serde
/// behavior is to use ISO 639-3 codes and we need ISO 639-1;
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Hash, Eq)]
#[repr(transparent)]
#[serde(try_from = "CowStr")]
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

impl<'a> TryFrom<CowStr<'a>> for Locale {
    type Error = String;

    fn try_from(value: CowStr<'a>) -> Result<Self, Self::Error> {
        let value = value.0.as_ref();
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
#[allow(clippy::field_reassign_with_default)]
pub mod tests {
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
        let translations =
            HashMap::from([(Locale(Language::Fra), "test, but in french".to_owned())]);

        FieldLocalization::builder()
            .code(code)
            .locale(locale)
            .catalogs(catalogs)
            .translations(translations)
            .build()
    }

    pub fn make_minimal_msg_localization() -> MsgLocalization {
        MsgLocalization::default()
    }

    pub fn make_extended_msg_localization() -> MsgLocalization {
        let catalogs = vec!["https://dummy.dummy/dummy".parse().unwrap()];
        let locales = HashMap::from([(
            Locale(Language::Fra),
            vec!["test, but in french".to_owned()],
        )]);

        MsgLocalization::builder()
            .catalogs(catalogs)
            .locales(locales)
            .build()
    }

    #[test]
    fn test_minimal_field_localization() {
        let localization = make_minimal_field_localization();
        let expected = json!({});

        test_utils::test_serde(localization, expected);
    }

    #[test]
    fn test_extended_field_localization() {
        let localization = make_extended_field_localization();

        let mut expected = json!({
            "code": localization.code,
            "locale": localization.locale,
            "catalogs": localization.catalogs,
        });

        let map = expected.as_object_mut().unwrap();
        for (key, value) in &localization.translations {
            let key = <&str>::try_from(key).unwrap().to_owned();
            let value = Value::String(value.to_owned());
            map.insert(key, value);
        }

        test_utils::test_serde(localization, expected);
    }

    #[test]
    fn test_minimal_msg_localization() {
        let localization = make_minimal_msg_localization();
        let expected = json!({});

        test_utils::test_serde(localization, expected);
    }

    #[test]
    fn test_extended_msg_localization() {
        let localization = make_extended_msg_localization();

        let expected = json!({
            "catalogs": localization.catalogs,
            "locales": localization.locales
        });

        test_utils::test_serde(localization, expected);
    }
}
