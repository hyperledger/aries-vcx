use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    msg_fields::protocols::report_problem::{
        ProblemReport, ProblemReportContent, ProblemReportDecorators,
    },
    msg_parts::MsgParts,
};

pub type CredIssuanceV1ProblemReport =
    MsgParts<CredIssuanceV1ProblemReportContent, ProblemReportDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
#[serde(transparent)]
pub struct CredIssuanceV1ProblemReportContent {
    pub inner: ProblemReportContent,
}

impl From<ProblemReportContent> for CredIssuanceV1ProblemReportContent {
    fn from(value: ProblemReportContent) -> Self {
        Self { inner: value }
    }
}

impl From<CredIssuanceV1ProblemReport> for ProblemReport {
    fn from(value: CredIssuanceV1ProblemReport) -> Self {
        Self::builder()
            .id(value.id)
            .content(value.content.inner)
            .decorators(value.decorators)
            .build()
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use std::collections::HashMap;

    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{
            localization::tests::make_extended_field_localization,
            thread::tests::make_extended_thread, timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_fields::protocols::report_problem::{
            Description, Impact, Where, WhereParty, WhoRetries,
        },
        msg_types::cred_issuance::CredentialIssuanceTypeV1_0,
    };

    #[test]
    fn test_minimal_problem_report() {
        let description = Description::builder()
            .code("test_problem_report_code".to_owned())
            .build();

        let content: CredIssuanceV1ProblemReportContent = ProblemReportContent::builder()
            .description(description)
            .build();
        let decorators = ProblemReportDecorators::default();

        let expected = json!({
            "description": content.inner.description
        });

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV1_0::ProblemReport,
            expected,
        );
    }

    #[test]
    fn test_extended_problem_report() {
        let description = Description::builder()
            .code("test_problem_report_code".to_owned())
            .build();

        let content: ProblemReportContent = ProblemReportContent::builder()
            .description(description)
            .who_retries(WhoRetries::Me)
            .fix_hint("test_fix_hint".to_owned())
            .impact(Impact::Connection)
            .location(Where::new(WhereParty::Me, "test_location".to_owned()))
            .noticed_time("test_noticed_time".to_owned())
            .tracking_uri("https://dummy.dummy/dummy".parse().unwrap())
            .escalation_uri("https://dummy.dummy/dummy".parse().unwrap())
            .problem_items(vec![HashMap::from([(
                "test_prob_item_key".to_owned(),
                "test_prob_item_value".to_owned(),
            )])])
            .build();

        let decorators = ProblemReportDecorators::builder()
            .thread(make_extended_thread())
            .timing(make_extended_timing())
            .description_locale(make_extended_field_localization())
            .fix_hint_locale(make_extended_field_localization())
            .build();

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

        let content = CredIssuanceV1ProblemReportContent::builder()
            .inner(content)
            .build();

        test_utils::test_msg(
            content,
            decorators,
            CredentialIssuanceTypeV1_0::ProblemReport,
            expected,
        );
    }
}
