use std::marker::PhantomData;

use messages::msg_fields::protocols::cred_issuance::v2::ack::AckCredentialV2;

use crate::protocols::issuance_v2::formats::holder::HolderCredentialIssuanceFormat;

pub struct Complete<T: HolderCredentialIssuanceFormat> {
    pub(crate) ack: Option<AckCredentialV2>,
    pub(crate) _marker: PhantomData<T>,
}

impl<T: HolderCredentialIssuanceFormat> Complete<T> {
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
