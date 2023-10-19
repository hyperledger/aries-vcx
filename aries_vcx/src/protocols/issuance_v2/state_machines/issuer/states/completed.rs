use std::marker::PhantomData;

use messages::msg_fields::protocols::cred_issuance::v2::ack::AckCredentialV2;

use crate::protocols::issuance_v2::formats::issuer::IssuerCredentialIssuanceFormat;

pub struct Completed<T: IssuerCredentialIssuanceFormat> {
    ack: Option<AckCredentialV2>,
    _marker: PhantomData<T>,
}

impl<T: IssuerCredentialIssuanceFormat> Completed<T> {
    pub fn new(ack: Option<AckCredentialV2>) -> Self {
        Self {
            ack,
            _marker: PhantomData,
        }
    }

    pub fn get_ack(&self) -> Option<&AckCredentialV2> {
        self.ack.as_ref()
    }
}
