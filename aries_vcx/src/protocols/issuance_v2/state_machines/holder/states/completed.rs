use std::marker::PhantomData;

use crate::protocols::issuance_v2::formats::holder::HolderCredentialIssuanceFormat;

pub struct Completed<T: HolderCredentialIssuanceFormat> {
    _marker: PhantomData<T>,
}

impl<T: HolderCredentialIssuanceFormat> Completed<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}
