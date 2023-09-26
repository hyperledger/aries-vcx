use std::marker::PhantomData;

use messages::msg_fields::protocols::cred_issuance::v2::propose_credential::ProposeCredentialV2;

use crate::protocols::issuance_v2::formats::holder::HolderCredentialIssuanceFormat;

pub struct ProposalPrepared<T: HolderCredentialIssuanceFormat> {
    pub proposal: ProposeCredentialV2,
    pub _marker: PhantomData<T>,
}
