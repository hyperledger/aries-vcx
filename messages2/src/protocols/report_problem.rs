use std::{collections::HashMap, fmt::Display};

use messages_macros::MessageContent;
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use strum_macros::{AsRefStr, EnumString};
use url::Url;

use crate::{
    decorators::{FieldLocalization, Thread, Timing},
    msg_types::types::report_problem::ReportProblemV1_0Kind,
    Message,
};

pub type ProblemReport = Message<ProblemReportContent, ProblemReportDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, MessageContent, Default, PartialEq)]
#[message(kind = "ReportProblemV1_0Kind::ProblemReport")]
pub struct ProblemReportContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_items: Option<Vec<HashMap<String, String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub who_retries: Option<WhoRetries>,
    #[serde(rename = "fix-hint")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact: Option<Impact>,
    #[serde(rename = "where")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<Where>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noticed_time: Option<String>,
    #[serde(rename = "tracking-uri")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracking_uri: Option<Url>,
    #[serde(rename = "escalation-uri")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub escalation_uri: Option<Url>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ProblemReportDecorators {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
    #[serde(rename = "~timing")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timing: Option<Timing>,
    #[serde(rename = "description~l10n")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description_locale: Option<FieldLocalization>,
    #[serde(rename = "fix-hint~l10n")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_hint_locale: Option<FieldLocalization>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WhoRetries {
    Me,
    You,
    Both,
    None,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Impact {
    MessageContent,
    Thread,
    Connection,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(AsRefStr, Debug, Copy, Clone, Serialize, Deserialize, EnumString, PartialEq)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum WhereParty {
    Me,
    You,
    Other,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{AriesMessage, Message};

    const PROBLEM_REPORT: &str = "https://didcomm.org/report-problem/1.0/problem-report";

    #[test]
    fn test_minimal_message() {
        let id = "test".to_owned();

        let content = ProblemReportContent::default();

        let decorators = ProblemReportDecorators::default();
        let msg = Message::with_decorators(id.clone(), content, decorators);
        let msg = AriesMessage::from(msg);

        let json = json!({
            "@type": PROBLEM_REPORT,
            "@id": id,
        });

        let deserialized = AriesMessage::deserialize(&json).unwrap();

        assert_eq!(serde_json::to_value(&msg).unwrap(), json);
        assert_eq!(deserialized, msg);
    }

    #[test]
    fn test_extensive_message() {
        let description = "test".to_owned();
        let who_retries = WhoRetries::Me;
        let thid = "test".to_owned();
        let id = "test".to_owned();

        let thread = Thread::new(thid.clone());

        let mut content = ProblemReportContent::default();
        content.description = Some(description.clone());
        content.who_retries = Some(who_retries);

        let mut decorators = ProblemReportDecorators::default();
        decorators.thread = Some(thread);

        let msg = Message::with_decorators(id.clone(), content, decorators);
        let msg = AriesMessage::from(msg);

        let json = json!({
            "@type": PROBLEM_REPORT,
            "@id": id,
            "description": description,
            "who_retries": who_retries,
            "~thread": {
                "thid": thid
            }
        });

        let deserialized = AriesMessage::deserialize(&json).unwrap();

        assert_eq!(serde_json::to_value(&msg).unwrap(), json);
        assert_eq!(deserialized, msg);
    }
}
