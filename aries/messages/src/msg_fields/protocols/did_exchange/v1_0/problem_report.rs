use crate::{
    msg_fields::protocols::did_exchange::v1_x::problem_report::{
        ProblemReportContent, ProblemReportDecorators,
    },
    msg_parts::MsgParts,
};

/// Alias type for DIDExchange v1.0 Problem Report message.
/// Note that since this inherits from the V1.X message, the direct serialization
/// of this message type is not recommended, as version metadata will be lost.
/// Instead, this type should be converted to/from an AriesMessage
pub type ProblemReport = MsgParts<ProblemReportContent, ProblemReportDecorators>;

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
        msg_fields::protocols::did_exchange::v1_x::problem_report::ProblemCode,
        msg_types::protocols::did_exchange::{DidExchangeTypeV1, DidExchangeTypeV1_0},
    };

    #[test]
    fn test_minimal_conn_problem_report() {
        let content = ProblemReportContent::builder()
            .problem_code(None)
            .explain(None)
            .version(DidExchangeTypeV1::new_v1_0())
            .build();

        let decorators = ProblemReportDecorators::new(make_extended_thread());

        let expected = json!({
            "~thread": decorators.thread
        });

        test_utils::test_msg(
            content,
            decorators,
            DidExchangeTypeV1_0::ProblemReport,
            expected,
        );
    }

    #[test]
    fn test_extended_conn_problem_report() {
        let content = ProblemReportContent::builder()
            .problem_code(Some(ProblemCode::RequestNotAccepted))
            .explain(Some("test_conn_problem_report_explain".to_owned()))
            .version(DidExchangeTypeV1::new_v1_0())
            .build();

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
            DidExchangeTypeV1_0::ProblemReport,
            expected,
        );
    }
}
