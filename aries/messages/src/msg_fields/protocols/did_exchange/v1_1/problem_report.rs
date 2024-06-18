use crate::{
    msg_fields::protocols::did_exchange::v1_x::problem_report::{
        ProblemReportContent, ProblemReportDecorators,
    },
    msg_parts::MsgParts,
    msg_types::{protocols::did_exchange::DidExchangeTypeV1_1, MsgKindType},
};

pub type ProblemReportContentV1_1 = ProblemReportContent<MsgKindType<DidExchangeTypeV1_1>>;
pub type ProblemReport = MsgParts<ProblemReportContentV1_1, ProblemReportDecorators>;

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        decorators::{
            localization::tests::make_extended_msg_localization,
            thread::tests::make_extended_thread, timing::tests::make_extended_timing,
        },
        misc::test_utils,
        msg_fields::protocols::did_exchange::v1_x::problem_report::{
            ProblemCode, ProblemReportDecorators,
        },
        msg_types::protocols::did_exchange::DidExchangeTypeV1_1,
    };

    #[test]
    fn test_minimal_conn_problem_report() {
        let content = ProblemReportContentV1_1::default();

        let decorators = ProblemReportDecorators::new(make_extended_thread());

        let expected = json!({
            "~thread": decorators.thread
        });

        test_utils::test_msg(
            content,
            decorators,
            DidExchangeTypeV1_1::ProblemReport,
            expected,
        );
    }

    #[test]
    fn test_extended_conn_problem_report() {
        let mut content = ProblemReportContentV1_1::default();
        content.problem_code = Some(ProblemCode::RequestNotAccepted);
        content.explain = Some("test_conn_problem_report_explain".to_owned());

        let mut decorators = ProblemReportDecorators::new(make_extended_thread());
        decorators.timing = Some(make_extended_timing());
        decorators.localization = Some(make_extended_msg_localization());

        let expected = json!({
            "problem-code": content.problem_code,
            "explain": content.explain,
            "~thread": decorators.thread,
            "~timing": decorators.timing,
            "~l10n": decorators.localization
        });

        test_utils::test_msg(
            content,
            decorators,
            DidExchangeTypeV1_1::ProblemReport,
            expected,
        );
    }
}
