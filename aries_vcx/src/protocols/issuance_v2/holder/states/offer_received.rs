use std::marker::PhantomData;

use messages::msg_fields::protocols::cred_issuance::v2::offer_credential::OfferCredentialV2;

use crate::protocols::issuance_v2::formats::holder::HolderCredentialIssuanceFormat;

pub struct OfferReceived<T: HolderCredentialIssuanceFormat> {
    pub offer: OfferCredentialV2,
    pub _marker: PhantomData<T>,
}
