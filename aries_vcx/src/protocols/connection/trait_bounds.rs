use messages::diddoc::aries::diddoc::AriesDidDoc;

/// Trait used for implementing common [`super::Connection`] behavior based
/// on states implementing it.
pub trait TheirDidDoc {
    fn their_did_doc(&self) -> &AriesDidDoc;
}

/// Trait used for implementing common [`super::Connection`] behavior based
/// on states implementing it.
pub trait ThreadId {
    fn thread_id(&self) -> &str;
}

/// Marker trait used for implementing [`messages::protocols::connection::problem_report::ProblemReport`]
/// handling on certain [`super::Connection`] types.
pub trait HandleProblem {}
