use std::collections::HashMap;

use crate::aries::handlers::issuance::holder::state_machine::HolderSM;
use crate::aries::handlers::issuance::messages::CredentialIssuanceMessage;
use crate::aries::messages::a2a::A2AMessage;
use crate::aries::messages::issuance::credential_offer::CredentialOffer;
use crate::connection;
use crate::error::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Holder {
    holder_sm: HolderSM
}

impl Holder {
    pub fn create(credential_offer: CredentialOffer, source_id: &str) -> VcxResult<Holder> {
        trace!("Holder::holder_create_credential >>> credential_offer: {:?}, source_id: {:?}", credential_offer, source_id);

        let holder_sm = HolderSM::new(credential_offer, source_id.to_string());

        Ok(Holder { holder_sm })
    }

    pub fn send_request(&mut self, connection_handle: u32) -> VcxResult<()> {
        self.step(CredentialIssuanceMessage::CredentialRequestSend(connection_handle))
    }

    pub fn maybe_update_connection_handle(&mut self, connection_handle: Option<u32>) -> u32 {
        let conn_handle = connection_handle.unwrap_or(self.holder_sm.get_connection_handle());
        self.holder_sm.set_connection_handle(conn_handle);
        conn_handle
    }

    pub fn is_terminal_state(&self) -> bool {
        self.holder_sm.is_terminal_state()
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        self.holder_sm.find_message_to_handle(messages)
    }

    pub fn get_status(&self) -> u32 {
        self.holder_sm.state()
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

    pub fn delete_credential(&self) -> VcxResult<()> {
        self.holder_sm.delete_credential()
    }

    pub fn get_credential_status(&self) -> VcxResult<u32> {
        Ok(self.holder_sm.credential_status())
    }

    pub fn step(&mut self, message: CredentialIssuanceMessage) -> VcxResult<()> {
        self.holder_sm = self.holder_sm.clone().handle_message(message)?;
        Ok(())
    }

    pub fn get_credential_offer_message(connection_handle: u32, msg_id: &str) -> VcxResult<A2AMessage> {
        match connection::get_message_by_id(connection_handle, msg_id.to_string()) {
            Ok(message) => match message {
                A2AMessage::CredentialOffer(_) => Ok(message),
                msg => {
                    return Err(VcxError::from_msg(VcxErrorKind::InvalidMessages,
                                                  format!("Message of different type was received: {:?}", msg)));
                }
            }
            Err(err) => Err(err)
        }
    }

    pub fn get_credential_offer_messages(conn_handle: u32) -> VcxResult<Vec<A2AMessage>> {
        let messages = connection::get_messages(conn_handle)?;
        let msgs: Vec<A2AMessage> = messages
            .into_iter()
            .filter_map(|(_, a2a_message)| {
                match a2a_message {
                    A2AMessage::CredentialOffer(_) => Some(a2a_message),
                    _ => None
                }
            })
            .collect();
        Ok(msgs)
    }
}
