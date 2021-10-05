use std::collections::HashMap;

use crate::error::prelude::*;
use crate::handlers::connection::connection::Connection;
use crate::handlers::issuance::holder::state_machine::HolderSM;
use crate::handlers::issuance::messages::CredentialIssuanceMessage;
use crate::messages::a2a::A2AMessage;
use crate::messages::issuance::credential_offer::CredentialOffer;
use crate::messages::issuance::credential_proposal::CredentialProposal;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Holder {
    holder_sm: HolderSM,
}

#[derive(Debug, PartialEq)]
pub enum HolderState {
    Initial,
    ProposalSent,
    OfferReceived,
    RequestSent,
    Finished,
    Failed,
}

impl Holder {
    pub fn create_from_proposal(credential_proposal: CredentialProposal, source_id: &str) -> VcxResult<Holder> {
        trace!("Holder::create_from_proposal >>> credential_proposal: {:?}, source_id: {:?}", credential_proposal, source_id);
        let holder_sm = HolderSM::from_proposal(credential_proposal, source_id.to_string());
        Ok(Holder { holder_sm })
    }

    pub fn create_from_offer(credential_offer: CredentialOffer, source_id: &str) -> VcxResult<Holder> {
        trace!("Holder::create_from_offer >>> credential_offer: {:?}, source_id: {:?}", credential_offer, source_id);
        let holder_sm = HolderSM::from_offer(credential_offer, source_id.to_string());
        Ok(Holder { holder_sm })
    }

    pub fn send_proposal(&mut self, credential_proposal: CredentialProposal, send_message: impl Fn(&A2AMessage) -> VcxResult<()>) -> VcxResult<()> {
        self.step(CredentialIssuanceMessage::CredentialProposalSend(credential_proposal), Some(&send_message))
    }

    pub fn send_request(&mut self, my_pw_did: String, send_message: impl Fn(&A2AMessage) -> VcxResult<()>) -> VcxResult<()> {
        self.step(CredentialIssuanceMessage::CredentialRequestSend(my_pw_did), Some(&send_message))
    }

    // TODO: Allow to reject the offer
    pub fn reject_offer(&mut self, my_pw_did: String, send_message: impl Fn(&A2AMessage) -> VcxResult<()>) -> VcxResult<()> {
        Ok(())
    }

    pub fn is_terminal_state(&self) -> bool {
        self.holder_sm.is_terminal_state()
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        self.holder_sm.find_message_to_handle(messages)
    }

    pub fn get_state(&self) -> HolderState {
        self.holder_sm.get_state()
    }

    pub fn get_source_id(&self) -> String {
        self.holder_sm.get_source_id()
    }

    pub fn get_credential(&self) -> VcxResult<(String, A2AMessage)> {
        self.holder_sm.get_credential()
    }

    pub fn get_attributes(&self) -> VcxResult<String> {
        self.holder_sm.get_attributes()
    }

    pub fn get_attachment(&self) -> VcxResult<String> {
        self.holder_sm.get_attachment()
    }

    pub fn get_tails_location(&self) -> VcxResult<String> {
        self.holder_sm.get_tails_location()
    }

    pub fn get_tails_hash(&self) -> VcxResult<String> {
        self.holder_sm.get_tails_hash()
    }

    pub fn get_rev_reg_id(&self) -> VcxResult<String> {
        self.holder_sm.get_rev_reg_id()
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        self.holder_sm.get_thread_id()
    }

    pub fn is_revokable(&self) -> VcxResult<bool> {
        self.holder_sm.is_revokable()
    }

    pub fn delete_credential(&self) -> VcxResult<()> {
        self.holder_sm.delete_credential()
    }

    pub fn get_credential_status(&self) -> VcxResult<u32> {
        Ok(self.holder_sm.credential_status())
    }

    pub fn step(&mut self, message: CredentialIssuanceMessage, send_message: Option<&impl Fn(&A2AMessage) -> VcxResult<()>>) -> VcxResult<()> {
        self.holder_sm = self.holder_sm.clone().handle_message(message, send_message)?;
        Ok(())
    }

    pub fn update_state(&mut self, connection: &Connection) -> VcxResult<HolderState> {
        trace!("Holder::update_state >>> ");
        if self.is_terminal_state() { return Ok(self.get_state()); }
        let send_message = connection.send_message_closure()?;

        let messages = connection.get_messages()?;
        if let Some((uid, msg)) = self.find_message_to_handle(messages) {
            self.step(msg.into(), Some(&send_message))?;
            connection.update_message_status(uid)?;
        }
        Ok(self.get_state())
    }
}
