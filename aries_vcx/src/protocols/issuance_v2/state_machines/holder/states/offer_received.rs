use std::marker::PhantomData;

use messages::msg_fields::protocols::cred_issuance::v2::offer_credential::OfferCredentialV2;

use crate::protocols::issuance_v2::formats::holder::HolderCredentialIssuanceFormat;

pub struct OfferReceived<T: HolderCredentialIssuanceFormat> {
    offer: OfferCredentialV2,
    _marker: PhantomData<T>,
}

impl<T: HolderCredentialIssuanceFormat> OfferReceived<T> {
    pub fn new(offer: OfferCredentialV2) -> Self {
        Self {
            offer,
            _marker: PhantomData,
        }
    }

    pub fn get_offer(&self) -> &OfferCredentialV2 {
        &self.offer
    }
}
