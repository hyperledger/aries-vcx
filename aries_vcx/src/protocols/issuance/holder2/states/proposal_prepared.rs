use messages2::msg_fields::protocols::cred_issuance::propose_credential::ProposeCredential;

pub struct ProposalPrepared {
    pub(crate) proposal_message: ProposeCredential,
}

impl ProposalPrepared {
    pub fn new(proposal_message: ProposeCredential) -> Self {
        Self { proposal_message }
    }
}
