use messages::msg_fields::protocols::connection::problem_report::ProblemReport;
use messages::msg_fields::protocols::connection::request::Request;
use messages::msg_fields::protocols::connection::response::Response;

use crate::errors::error::AriesVcxError;
use crate::handlers::util::AnyInvitation;
use crate::protocols::mediated_connection::inviter::states::initial::InitialState;
use crate::protocols::mediated_connection::inviter::states::requested::RequestedState;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InvitedState {
    pub invitation: AnyInvitation,
}

// TODO: These have no justification for being here anymore
impl From<ProblemReport> for InitialState {
    fn from(problem_report: ProblemReport) -> InitialState {
        trace!(
            "ConnectionInviter: transit state to InitialState, problem_report: {:?}",
            problem_report
        );
        InitialState::new(Some(problem_report))
    }
}

impl TryFrom<(Request, Response)> for RequestedState {
    type Error = AriesVcxError;

    fn try_from((request, signed_response): (Request, Response)) -> Result<RequestedState, Self::Error> {
        trace!("ConnectionInviter: transit state to RespondedState");
        Ok(RequestedState {
            signed_response,
            did_doc: request.content.connection.did_doc.try_into()?,
            thread_id: request.id,
        })
    }
}
