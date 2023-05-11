use messages::msg_fields::protocols::cred_issuance::offer_credential::OfferCredential;

#[derive(Debug)]
pub struct OfferReceived {
    pub(crate) offer_message: OfferCredential,
}

impl OfferReceived {
    pub fn new(offer_message: OfferCredential) -> Self {
        OfferReceived { offer_message }
    }
}
