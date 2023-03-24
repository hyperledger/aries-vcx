//! Module containing the `report problem` protocol messages, as defined in the [RFC](https://github.com/hyperledger/aries-rfcs/blob/main/features/0035-report-problem/README.md).

use std::{collections::HashMap, fmt::Display};

use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use strum_macros::{AsRefStr, EnumString};
use url::Url;

use crate::{
    decorators::{localization::FieldLocalization, thread::Thread, timing::Timing},
    misc::utils::into_msg_with_type,
    msg_parts::MsgParts,
    msg_types::types::report_problem::ReportProblemProtocolV1_0,
};

pub type ProblemReport = MsgParts<ProblemReportContent, ProblemReportDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
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

impl Where {
    pub fn new(party: WhereParty, location: String) -> Self {
        Self { party, location }
    }
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
pub enum WhereParty {
    Me,
    You,
    Other,
}

into_msg_with_type!(ProblemReport, ReportProblemProtocolV1_0, ProblemReport);

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{
            localization::tests::make_extended_field_localization, thread::tests::make_extended_thread,
            timing::tests::make_extended_timing,
        },
        misc::test_utils,
    };

    #[test]
    fn test_minimal_problem_report() {
        let content = ProblemReportContent::default();
        let decorators = ProblemReportDecorators::default();

        let expected = json!({});

        test_utils::test_msg(content, decorators, ReportProblemProtocolV1_0::ProblemReport, expected);
    }

    #[test]
    fn test_extended_problem_report() {
        let mut content = ProblemReportContent::default();
        content.description = Some("test_description".to_owned());
        content.who_retries = Some(WhoRetries::Me);
        content.fix_hint = Some("test_fix_hint".to_owned());
        content.impact = Some(Impact::Connection);
        content.location = Some(Where::new(WhereParty::Me, "test_location".to_owned()));
        content.noticed_time = Some("test_noticed_time".to_owned());
        content.tracking_uri = Some("https://dummy.dummy/dummy".parse().unwrap());
        content.escalation_uri = Some("https://dummy.dummy/dummy".parse().unwrap());
        content.problem_items = Some(vec![HashMap::from([(
            "test_prob_item_key".to_owned(),
            "test_prob_item_value".to_owned(),
        )])]);

        let mut decorators = ProblemReportDecorators::default();
        decorators.thread = Some(make_extended_thread());
        decorators.timing = Some(make_extended_timing());
        decorators.description_locale = Some(make_extended_field_localization());
        decorators.fix_hint_locale = Some(make_extended_field_localization());

        let expected = json!({
            "description": content.description,
            "who_retries": content.who_retries,
            "fix-hint": content.fix_hint,
            "impact": content.impact,
            "where": content.location,
            "noticed_time": content.noticed_time,
            "tracking-uri": content.tracking_uri,
            "escalation-uri": content.escalation_uri,
            "problem_items": content.problem_items,
            "~thread": decorators.thread,
            "~timing": decorators.timing,
            "description~l10n": decorators.description_locale,
            "fix-hint~l10n": decorators.fix_hint_locale
        });

        test_utils::test_msg(content, decorators, ReportProblemProtocolV1_0::ProblemReport, expected);
    }
}
