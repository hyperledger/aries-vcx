use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

use crate::{
    msg_fields::protocols::report_problem::{ProblemReportContent, ProblemReportDecorators},
    msg_parts::MsgParts,
};

pub type PresentProofProblemReport = MsgParts<PresentProofProblemReportContent, ProblemReportDecorators>;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, TypedBuilder)]
#[serde(transparent)]
pub struct PresentProofProblemReportContent {
    pub inner: ProblemReportContent,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use std::collections::HashMap;

    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{
            localization::tests::make_extended_field_localization, thread::tests::make_extended_thread,
            timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_fields::protocols::report_problem::{Description, Impact, Where, WhereParty, WhoRetries},
        msg_types::present_proof::PresentProofTypeV1_0,
    };

    #[test]
    fn test_minimal_problem_report() {
        let description = Description::builder()
            .code("test_problem_report_code".to_owned())
            .build();
        let content = ProblemReportContent::builder().description(description).build();
        let decorators = ProblemReportDecorators::default();

        let expected = json!({
            "description": content.description
        });

        let content = PresentProofProblemReportContent::builder().inner(content).build();

        test_utils::test_msg(content, decorators, PresentProofTypeV1_0::ProblemReport, expected);
    }

    #[test]
    fn test_extended_problem_report() {
        let description = Description::builder()
            .code("test_problem_report_code".to_owned())
            .build();
        let mut content = ProblemReportContent::builder().description(description).build();
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

        let content = PresentProofProblemReportContent::builder().inner(content).build();

        test_utils::test_msg(content, decorators, PresentProofTypeV1_0::ProblemReport, expected);
    }
}
