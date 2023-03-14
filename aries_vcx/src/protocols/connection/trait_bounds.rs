use messages::{
    diddoc::aries::diddoc::AriesDidDoc,
    protocols::discovery::disclose::{Disclose, ProtocolDescriptor},
};

/// Trait implemented for [`super::Connection`] states that store an [`AriesDidDoc`].
pub trait TheirDidDoc {
    /// Returns the [`AriesDidDoc`] currently being used by a [`super::Connection`].
    fn their_did_doc(&self) -> &AriesDidDoc;
}

pub trait BootstrapDidDoc: TheirDidDoc {
    /// Returns the [`AriesDidDoc`] used to bootstrap the connection.
    /// By default, this will be the same as the [`AriesDidDoc`] currently being used
    /// by the connection.
    fn bootstrap_did_doc(&self) -> &AriesDidDoc {
        self.their_did_doc()
    }
}

/// Trait implemented for [`super::Connection`] states that keep track of a thread ID.
pub trait ThreadId {
    fn thread_id(&self) -> &str;
}

/// Trait impletement for [`super::Connection`] in completed states.
pub trait CompletedState {
    fn remote_protocols(&self) -> Option<&[ProtocolDescriptor]>;

    fn handle_disclose(&mut self, disclose: Disclose);
}

/// Marker trait used for implementing
/// [`messages::protocols::connection::problem_report::ProblemReport`] handling on certain
/// [`super::Connection`] types.
pub trait HandleProblem {}
