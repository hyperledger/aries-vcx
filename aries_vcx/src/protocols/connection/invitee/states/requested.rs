use crate::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};
use crate::protocols::connection::invitee::states::initial::InitialState;
use crate::protocols::connection::invitee::states::responded::RespondedState;
use crate::protocols::connection::trait_bounds::TheirDidDoc;
use messages::diddoc::aries::diddoc::AriesDidDoc;
use messages::protocols::connection::problem_report::ProblemReport;
use messages::protocols::connection::request::Request;
use messages::protocols::connection::response::Response;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestedState {
    pub request: Request,
    pub did_doc: AriesDidDoc,
}

impl RequestedState {
    /// Attempts to convert [`Self`] based on a [`Response`] into a [`RespondedState`].
    ///
    /// # Errors
    /// An error is returned if the response has an unexpected thread ID.
    pub fn try_into_responded(self, response: Response) -> VcxResult<RespondedState> {
        if !response.from_thread(&self.request.get_thread_id()) {
            let err_msg = format!(
                "Cannot handle response: thread id does not match: {:?}",
                response.thread
            );

            let err = AriesVcxError::from_msg(AriesVcxErrorKind::InvalidJson, err_msg);

            Err(err)
        } else {
            let state = RespondedState {
                response,
                request: self.request,
                did_doc: self.did_doc,
            };
            Ok(state)
        }
    }
}

impl From<(RequestedState, ProblemReport)> for InitialState {
    fn from((_state, problem_report): (RequestedState, ProblemReport)) -> InitialState {
        trace!(
            "ConnectionInvitee: transit state from RequestedState to InitialState, problem_report: {:?}",
            problem_report
        );
        InitialState::new(Some(problem_report), _state.did_doc)
    }
}

impl From<(RequestedState, Response)> for RespondedState {
    fn from((state, response): (RequestedState, Response)) -> RespondedState {
        trace!("ConnectionInvitee: transit state from RequestedState to RespondedState");
        RespondedState {
            response,
            did_doc: state.did_doc,
            request: state.request,
        }
    }
}

impl TheirDidDoc for RequestedState {
    fn their_did_doc(&self) -> &AriesDidDoc {
        &self.did_doc
    }
}
