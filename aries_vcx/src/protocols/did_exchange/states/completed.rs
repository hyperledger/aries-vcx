use std::clone::Clone;

use super::traits::ThreadId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Completed {
    pub invitation_id: String,
    pub request_id: String,
}

impl ThreadId for Completed {
    fn thread_id(&self) -> &str {
        self.request_id.as_str()
    }
}
