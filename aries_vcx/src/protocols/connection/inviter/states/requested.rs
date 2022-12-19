use messages::did_doc::DidDoc;
use messages::protocols::connection::problem_report::ProblemReport;
use messages::protocols::connection::response::SignedResponse;
use crate::protocols::connection::inviter::states::initial::InitialState;
use crate::protocols::connection::inviter::states::responded::RespondedState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestedState {
    pub signed_response: SignedResponse,
    pub did_doc: DidDoc,
    pub thread_id: String,
}

impl From<(RequestedState, ProblemReport)> for InitialState {
    fn from((_state, problem_report): (RequestedState, ProblemReport)) -> InitialState {
        trace!(
            "ConnectionInviter: transit state from RequestedState to InitialState, problem_report: {:?}",
            problem_report
        );
        InitialState::new(Some(problem_report))
    }
}

impl From<RequestedState> for RespondedState {
    fn from(state: RequestedState) -> RespondedState {
        trace!("ConnectionInviter: transit state from RequestedState to RespondedState");
        RespondedState {
            signed_response: state.signed_response,
            did_doc: state.did_doc,
        }
    }
}
