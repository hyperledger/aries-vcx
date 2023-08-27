use crate::protocols::did_exchange::states::traits::{InvitationId, ThreadId};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseSent {
    pub invitation_id: String,
    pub request_id: String,
}

impl ThreadId for ResponseSent {
    fn thread_id(&self) -> &str {
        self.request_id.as_str()
    }
}

impl InvitationId for ResponseSent {
    fn invitation_id(&self) -> &str {
        self.invitation_id.as_str()
    }
}
