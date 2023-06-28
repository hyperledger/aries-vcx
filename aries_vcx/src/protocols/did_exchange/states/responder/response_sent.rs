use crate::protocols::did_exchange::states::traits::ThreadId;

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
