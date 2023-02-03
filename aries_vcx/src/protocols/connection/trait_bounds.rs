use messages::diddoc::aries::diddoc::AriesDidDoc;

/// Trait implemented for [`super::Connection`] states that store an [`AriesDidDoc`].
pub trait TheirDidDoc {
    fn their_did_doc(&self) -> &AriesDidDoc;
}

/// Trait implemented for [`super::Connection`] states that keep track of a thread ID.
pub trait ThreadId {
    fn thread_id(&self) -> &str;
}

/// Marker trait used for implementing [`messages::protocols::connection::problem_report::ProblemReport`]
/// handling on certain [`super::Connection`] types.
pub trait HandleProblem {}
