use crate::protocols::did_exchange::states::traits::ThreadId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestSent {
    pub request_id: String,
    /* Note: Historical artifact in Aries RFC, used to fill pthread
     * value in Complete message       
     * See more info here: https://github.com/hyperledger/aries-rfcs/issues/817
     */
    pub invitation_id: Option<String>,
}

impl ThreadId for RequestSent {
    fn thread_id(&self) -> &str {
        self.request_id.as_str()
    }
}
