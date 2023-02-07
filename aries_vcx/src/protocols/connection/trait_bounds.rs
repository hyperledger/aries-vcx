use messages::{diddoc::aries::diddoc::AriesDidDoc, protocols::discovery::disclose::{ProtocolDescriptor, Disclose}};

/// Trait implemented for [`super::Connection`] states that store an [`AriesDidDoc`].
pub trait TheirDidDoc {
    fn their_did_doc(&self) -> &AriesDidDoc;
}

/// Trait implemented for [`super::Connection`] states that keep track of a thread ID.
pub trait ThreadId {
    fn thread_id(&self) -> &str;
}

/// Trait impletement for [`super::Connection`] in complete states.
pub trait CompleteState {
    fn remote_protocols(&self) -> Option<&[ProtocolDescriptor]>;

    fn handle_disclose(&mut self, disclose: Disclose);
}

/// Marker trait used for implementing [`messages::protocols::connection::problem_report::ProblemReport`]
/// handling on certain [`super::Connection`] types.
pub trait HandleProblem {}
