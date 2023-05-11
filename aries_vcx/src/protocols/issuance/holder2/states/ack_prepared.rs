use messages::msg_fields::protocols::cred_issuance::ack::AckCredential;

#[derive(Debug)]
pub struct AckPrepared {
    pub(crate) ack_message: AckCredential,
}

impl AckPrepared {
    pub fn new(ack_message: AckCredential) -> Self {
        AckPrepared { ack_message }
    }
}
