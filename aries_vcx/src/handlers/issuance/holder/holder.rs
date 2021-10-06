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
    pub fn create_from_proposal(source_id: &str, credential_proposal: CredentialProposal) -> VcxResult<Holder> {
        trace!("Holder::create_from_proposal >>> source_id: {:?}, credential_proposal: {:?}", source_id, credential_proposal);
        let holder_sm = HolderSM::from_proposal(credential_proposal, source_id.to_string());
        Ok(Holder { holder_sm })
    }

    pub fn create_from_offer(source_id: &str, credential_offer: CredentialOffer) -> VcxResult<Holder> {
        trace!("Holder::create_from_offer >>> source_id: {:?}, credential_offer: {:?}", source_id, credential_offer);
        let holder_sm = HolderSM::from_offer(credential_offer, source_id.to_string());
        Ok(Holder { holder_sm })
    }

    // TODO: Maybe this would be better?
    // pub fn send_proposal(&mut self, credential_proposal: CredentialProposal, send_message: impl Fn(&A2AMessage) -> VcxResult<()>) -> VcxResult<()> {
    //     self.step(CredentialIssuanceMessage::CredentialProposalSend(credential_proposal), Some(&send_message))
    // }

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

        match self.get_state() {
            HolderState::Initial => self.step(CredentialIssuanceMessage::CredentialProposalSend(self.holder_sm.get_proposal()?), Some(&send_message))?,
            _ => {
                let messages = connection.get_messages()?;
                if let Some((uid, msg)) = self.find_message_to_handle(messages) {
                    self.step(msg.into(), Some(&send_message))?;
                    connection.update_message_status(uid)?;
                }
            }
        };

        Ok(self.get_state())
    }
}

#[cfg(test)]
pub mod test {
    use crate::messages::issuance::credential::test_utils::_credential;
    use crate::messages::issuance::credential_offer::test_utils::_credential_offer;
    use crate::messages::issuance::credential_proposal::test_utils::_credential_proposal;
    use crate::messages::issuance::credential_request::test_utils::_my_pw_did;
    use crate::utils::devsetup::SetupMocks;

    use super::*;

    pub fn _send_message() -> Option<&'static impl Fn(&A2AMessage) -> VcxResult<()>> {
        Some(&|_: &A2AMessage| VcxResult::Ok(()))
    }

    fn _holder_from_offer() -> Holder {
        Holder::create_from_offer("test_source_id", _credential_offer()).unwrap()
    }

    fn _holder_from_proposal() -> Holder {
        Holder::create_from_proposal("test_source_id", _credential_proposal()).unwrap()
    }

    impl Holder {
        fn to_proposal_sent_state(mut self) -> Holder {
            self.step(CredentialIssuanceMessage::CredentialProposalSend(_credential_proposal()), _send_message()).unwrap();
            self
        }

        fn to_offer_received_state(mut self) -> Holder {
            self.step(CredentialIssuanceMessage::CredentialProposalSend(_credential_proposal()), _send_message()).unwrap();
            self.step(CredentialIssuanceMessage::CredentialOffer(_credential_offer()), _send_message()).unwrap();
            self
        }

        fn to_request_sent_state(mut self) -> Holder {
            self.step(CredentialIssuanceMessage::CredentialProposalSend(_credential_proposal()), _send_message()).unwrap();
            self.step(CredentialIssuanceMessage::CredentialOffer(_credential_offer()), _send_message()).unwrap();
            self.step(CredentialIssuanceMessage::CredentialRequestSend(_my_pw_did()), _send_message()).unwrap();
            self
        }

        fn to_finished_state(mut self) -> Holder {
            self.step(CredentialIssuanceMessage::CredentialProposalSend(_credential_proposal()), _send_message()).unwrap();
            self.step(CredentialIssuanceMessage::CredentialOffer(_credential_offer()), _send_message()).unwrap();
            self.step(CredentialIssuanceMessage::CredentialRequestSend(_my_pw_did()), _send_message()).unwrap();
            self.step(CredentialIssuanceMessage::Credential(_credential()), _send_message()).unwrap();
            self
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn exchange_credential_from_proposal_without_negotiation() {
        let _setup = SetupMocks::init();
        let issuer = _holder_from_proposal().to_finished_state();
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn exchange_credential_from_proposal_with_negotiation() {
    }
}
