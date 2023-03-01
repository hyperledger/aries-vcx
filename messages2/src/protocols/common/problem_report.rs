use std::{collections::HashMap, fmt::Display};

use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use strum_macros::{AsRefStr, EnumString};
use url::Url;

use crate::{
    decorators::{FieldLocalization, Thread},
    macros::threadlike_opt_impl,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemReport {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_locale: Option<FieldLocalization>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_items: Option<Vec<HashMap<String, String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub who_retries: Option<WhoRetries>,
    #[serde(rename = "fix-hint")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_hint: Option<String>,
    #[serde(rename = "fix-hint~l10n")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_hint_locale: Option<FieldLocalization>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact: Option<Impact>,
    #[serde(rename = "where")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _where: Option<Where>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noticed_time: Option<String>,
    // #[serde(rename = "~timing")]
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub timing: Option<Timing>, // do we use this? not specified in the RFC
    #[serde(rename = "tracking-uri")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_uri: Option<Url>,
    #[serde(rename = "escalation-uri")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escalation_uri: Option<Url>,
}

threadlike_opt_impl!(ProblemReport);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WhoRetries {
    Me,
    You,
    Both,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Impact {
    MessageContent,
    Thread,
    Connection,
}

#[derive(Debug, Clone)]
pub struct Where {
    pub party: WhereParty,
    pub location: String,
}

impl Display for Where {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.party.as_ref(), self.location.as_str())
    }
}

impl Serialize for Where {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Where {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let err_closure = |val: &str| D::Error::custom(format!("invalid where field: {val}"));

        let where_str = <&str>::deserialize(deserializer)?;
        let mut iter = where_str.split(" - ");

        let party = iter
            .next()
            .ok_or_else(|| err_closure(where_str))?
            .try_into()
            .map_err(D::Error::custom)?;

        let location = iter.next().ok_or_else(|| err_closure(where_str))?.to_owned();

        Ok(Where { party, location })
    }
}

#[derive(AsRefStr, Debug, Clone, Serialize, Deserialize, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum WhereParty {
    Me,
    You,
    Other,
}
