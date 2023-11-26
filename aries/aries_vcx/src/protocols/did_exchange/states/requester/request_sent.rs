use crate::protocols::did_exchange::states::traits::ThreadId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestSent {
    pub invitation_id: String,
    pub request_id: String,
}

impl ThreadId for RequestSent {
    fn thread_id(&self) -> &str {
        self.request_id.as_str()
    }
}
