use messages::msg_fields::protocols::cred_issuance::propose_credential::ProposeCredential;

#[derive(Debug)]
pub struct ProposalPrepared {
    pub(crate) proposal_message: ProposeCredential,
}

impl ProposalPrepared {
    pub fn new(proposal_message: ProposeCredential) -> Self {
        Self { proposal_message }
    }
}
