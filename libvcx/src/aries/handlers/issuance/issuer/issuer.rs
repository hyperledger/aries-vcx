use std::collections::HashMap;

use crate::aries::handlers::issuance::issuer::state_machine::IssuerSM;
use crate::aries::handlers::issuance::messages::CredentialIssuanceMessage;
use crate::aries::messages::a2a::A2AMessage;
use crate::credential_def;
use crate::error::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Issuer {
    issuer_sm: IssuerSM
}

impl Issuer {
    pub fn create(cred_def_handle: u32, credential_data: &str, source_id: &str) -> VcxResult<Issuer> {
        trace!("Issuer::issuer_create_credential >>> cred_def_handle: {:?}, credential_data: {:?}, source_id: {:?}", cred_def_handle, credential_data, source_id);

        let cred_def_id = credential_def::get_cred_def_id(cred_def_handle)?;
        let rev_reg_id = credential_def::get_rev_reg_id(cred_def_handle).ok();
        let tails_file = credential_def::get_tails_file(cred_def_handle)?;
        let issuer_sm = IssuerSM::new(&cred_def_id, credential_data, rev_reg_id, tails_file, source_id);
        Ok(Issuer { issuer_sm })
    }

    pub fn send_credential_offer(&mut self, connection_handle: u32, comment: Option<String>) -> VcxResult<()> {
        self.step(CredentialIssuanceMessage::CredentialInit(connection_handle, comment))
    }

    pub fn send_credential(&mut self, connection_handle: u32) -> VcxResult<()> {
        self.step(CredentialIssuanceMessage::CredentialSend(connection_handle))
    }

    pub fn get_state(&self) -> VcxResult<u32> {
        Ok(self.issuer_sm.state())
    }

    pub fn get_source_id(&self) -> VcxResult<String> {
        Ok(self.issuer_sm.get_source_id())
    }

    pub fn is_terminal_state(&self) -> bool {
        self.issuer_sm.is_terminal_state()
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        self.issuer_sm.find_message_to_handle(messages)
    }

    pub fn revoke_credential(&self, publish: bool) -> VcxResult<()> {
        self.issuer_sm.revoke(publish)
    }

    pub fn get_rev_reg_id(&self) -> VcxResult<String> {
        self.issuer_sm.get_rev_reg_id()
    }

    pub fn maybe_update_connection_handle(&mut self, connection_handle: Option<u32>) -> u32 {
        let conn_handle = connection_handle.unwrap_or(self.issuer_sm.get_connection_handle());
        self.issuer_sm.set_connection_handle(conn_handle);
        conn_handle
    }

    pub fn get_credential_status(&self) -> VcxResult<u32> {
        Ok(self.issuer_sm.credential_status())
    }

    pub fn step(&mut self, message: CredentialIssuanceMessage) -> VcxResult<()> {
        self.issuer_sm = self.issuer_sm.clone().handle_message(message)?;
        Ok(())
    }
}
