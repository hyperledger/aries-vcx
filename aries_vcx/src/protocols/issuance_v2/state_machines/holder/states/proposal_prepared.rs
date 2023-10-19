use std::marker::PhantomData;

use crate::protocols::issuance_v2::formats::holder::HolderCredentialIssuanceFormat;

pub struct ProposalPrepared<T: HolderCredentialIssuanceFormat> {
    _marker: PhantomData<T>,
}

impl<T: HolderCredentialIssuanceFormat> ProposalPrepared<T> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}
