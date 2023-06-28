pub trait ThreadId {
    fn thread_id(&self) -> &str;
}

pub trait InvitationId {
    fn invitation_id(&self) -> &str;
}
