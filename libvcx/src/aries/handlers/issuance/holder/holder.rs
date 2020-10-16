use std::collections::HashMap;
use std::convert::TryFrom;

use connection;
use error::prelude::*;
use aries::handlers::issuance::holder::state_machine::HolderSM;
use aries::handlers::issuance::messages::CredentialIssuanceMessage;
use aries::messages::a2a::A2AMessage;
use aries::messages::issuance::credential::{Credential, CredentialData};
use aries::messages::issuance::credential_offer::CredentialOffer;

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

    pub fn update_state(&mut self, msg: Option<String>, connection_handle: Option<u32>) -> VcxResult<()> {
        match msg {
            Some(msg) => {
                let message: A2AMessage = ::serde_json::from_str(&msg)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidOption, format!("Cannot update state: Message deserialization failed: {:?}", err)))?;

                self.step(message.into())
            }
            None => {
                self.holder_sm = self.holder_sm.clone().update_state(connection_handle)?;
                Ok(())
            }
        }
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
