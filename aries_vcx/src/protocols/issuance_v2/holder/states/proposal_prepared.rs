use std::marker::PhantomData;

use messages::msg_fields::protocols::cred_issuance::v2::propose_credential::ProposeCredentialV2;

use crate::protocols::issuance_v2::formats::holder::HolderCredentialIssuanceFormat;

pub struct ProposalPrepared<T: HolderCredentialIssuanceFormat> {
    pub(crate) proposal: ProposeCredentialV2,
    pub(crate) _marker: PhantomData<T>,
}

impl<T: HolderCredentialIssuanceFormat> ProposalPrepared<T> {
    pub fn new(proposal: ProposeCredentialV2) -> Self {
        Self {
            proposal,
            _marker: PhantomData,
        }
    }

    pub fn get_proposal(&self) -> &ProposeCredentialV2 {
        &self.proposal
    }
}
