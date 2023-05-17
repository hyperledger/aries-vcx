use diddoc_legacy::aries::diddoc::AriesDidDoc;
use messages::msg_fields::protocols::connection::{problem_report::ProblemReport, request::Request, ConnectionData};

use crate::protocols::mediated_connection::invitee::states::initial::InitialState;

/// For retro-fitting the new messages.
#[derive(Serialize, Deserialize)]
struct Response {
    connection: ConnectionData,
}

/// For retro-fitting the new messages.
#[derive(Serialize, Deserialize)]
struct RespondedStateDe {
    pub resp_con_data: Response,
    pub request: Request,
    pub did_doc: AriesDidDoc,
}

impl From<RespondedStateDe> for RespondedState {
    fn from(value: RespondedStateDe) -> Self {
        Self {
            resp_con_data: value.resp_con_data.connection,
            request: value.request,
            did_doc: value.did_doc,
        }
    }
}

impl From<RespondedState> for RespondedStateDe {
    fn from(value: RespondedState) -> Self {
        Self {
            resp_con_data: Response {
                connection: value.resp_con_data,
            },
            request: value.request,
            did_doc: value.did_doc,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(from = "RespondedStateDe", into = "RespondedStateDe")]
pub struct RespondedState {
    pub resp_con_data: ConnectionData,
    pub request: Request,
    pub did_doc: AriesDidDoc,
}

impl From<(RespondedState, ProblemReport)> for InitialState {
    fn from((_state, problem_report): (RespondedState, ProblemReport)) -> InitialState {
        trace!(
            "ConnectionInvitee: transit state from RespondedState to InitialState, problem_report: {:?}",
            problem_report
        );
        InitialState::new(Some(problem_report), None)
    }
}
