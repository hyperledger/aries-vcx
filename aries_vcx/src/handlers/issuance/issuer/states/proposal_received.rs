use crate::error::prelude::*;
use crate::messages::issuance::credential_proposal::CredentialProposal;
use crate::handlers::issuance::is_cred_def_revokable;
use crate::handlers::issuance::issuer::states::offer_sent::OfferSentState;
use crate::messages::a2a::MessageId;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProposalReceivedState {
    pub credential_proposal: CredentialProposal
}

impl ProposalReceivedState {
    pub fn new(credential_proposal: CredentialProposal) -> Self {
        Self { credential_proposal }
    }

    pub fn is_revokable(&self) -> VcxResult<bool> {
        is_cred_def_revokable(&self.credential_proposal.cred_def_id)
    }

    pub fn get_rev_reg_id(&self) -> VcxResult<Option<String>> {
        Ok(None)
    }
}

impl From<(String, String, MessageId, Option<String>, Option<String>)> for OfferSentState {
    fn from((cred_data, offer, sent_id, rev_reg_id, tails_file): (String, String, MessageId, Option<String>, Option<String>)) -> Self {
        trace!("SM is now in OfferSent state");
        OfferSentState {
            offer,
            cred_data,
            rev_reg_id,
            tails_file,
            thread_id: sent_id.0,
        }
    }
}
