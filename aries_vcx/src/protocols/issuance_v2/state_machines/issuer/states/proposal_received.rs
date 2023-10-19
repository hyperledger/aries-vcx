use std::marker::PhantomData;

use messages::msg_fields::protocols::cred_issuance::v2::propose_credential::ProposeCredentialV2;

use crate::protocols::issuance_v2::formats::issuer::IssuerCredentialIssuanceFormat;

pub struct ProposalReceived<T: IssuerCredentialIssuanceFormat> {
    proposal: ProposeCredentialV2,
    _marker: PhantomData<T>,
}

impl<T: IssuerCredentialIssuanceFormat> ProposalReceived<T> {
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
