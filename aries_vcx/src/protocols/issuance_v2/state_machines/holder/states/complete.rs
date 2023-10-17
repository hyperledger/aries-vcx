use std::marker::PhantomData;

use crate::protocols::issuance_v2::formats::holder::HolderCredentialIssuanceFormat;

pub struct Complete<T: HolderCredentialIssuanceFormat> {
    pub(crate) _marker: PhantomData<T>,
}

impl<T: HolderCredentialIssuanceFormat> Complete<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}
