use std::sync::{Arc, Mutex};

use aries_vcx::{
    handlers::connection::connection::Connection as VcxConnection, messages::diddoc::aries::diddoc::AriesDidDoc,
    protocols::connection::pairwise_info::PairwiseInfo,
};

use crate::{core::profile::ProfileHolder, errors::error::VcxUniFFIResult, runtime::block_on};

use super::ConnectionState;

pub struct Connection {
    handler: Mutex<VcxConnection>,
}

// seperate function since uniffi can't handle constructors with results
pub fn create_inviter(profile: Arc<ProfileHolder>) -> VcxUniFFIResult<Arc<Connection>> {
    block_on(async {
        let handler = Mutex::new(VcxConnection::create_inviter(&profile.inner).await?);

        Ok(Arc::new(Connection { handler }))
    })
}

// seperate function since uniffi can't handle constructors with results
pub fn create_invitee(profile: Arc<ProfileHolder>, did_doc: AriesDidDoc) -> VcxUniFFIResult<Arc<Connection>> {
    block_on(async {
        let handler = Mutex::new(VcxConnection::create_invitee(&profile.inner, did_doc).await?);

        Ok(Arc::new(Connection { handler }))
    })
}

impl Connection {
    pub fn get_state(&self) -> VcxUniFFIResult<ConnectionState> {
        let handler = self.handler.lock()?;
        Ok(ConnectionState::from(handler.get_state()))
    }

    pub fn pairwise_info(&self) -> VcxUniFFIResult<PairwiseInfo> {
        let handler = self.handler.lock()?;
        Ok(handler.pairwise_info().clone())
    }

    // NOTE : using string here out of laziness. We could have type this,
    // but UniFFI does not support structs with unnamed fields. So we'd have to
    // wrap these types
    // here invitation -> aries_vcx::Invitation
    pub fn process_invite(&self, invitation: String) -> VcxUniFFIResult<()> {
        let mut handler = self.handler.lock()?;
        let invitation = serde_json::from_str(&invitation)?;
        // TODO - do *really* we have to clone...
        *handler = handler.clone().process_invite(invitation).unwrap();

        Ok(())
    }

    // NOTE : using string here out of laziness. We could have type this,
    // but UniFFI does not support structs with unnamed fields. So we'd have to
    // wrap these types
    // here request -> aries_vcx::Request
    pub fn process_request(
        &self,
        profile: Arc<ProfileHolder>,
        request: String,
        service_endpoint: String,
        routing_keys: Vec<String>,
    ) -> VcxUniFFIResult<()> {
        let mut handler = self.handler.lock()?;
        let request = serde_json::from_str(&request)?;
        block_on(async {
            // TODO - do *really* we have to clone...
            *handler = handler
                .clone()
                .process_request(&profile.inner, request, service_endpoint, routing_keys, None)
                .await?;

            Ok(())
        })
    }

    // NOTE : using string here out of laziness. We could have type this,
    // but UniFFI does not support structs with unnamed fields. So we'd have to
    // wrap these types
    // here request -> aries_vcx::Request
    pub fn process_response(&self, profile: Arc<ProfileHolder>, response: String) -> VcxUniFFIResult<()> {
        let mut handler = self.handler.lock()?;
        let response = serde_json::from_str(&response)?;
        block_on(async {
            // TODO - do *really* we have to clone...
            *handler = handler.clone().process_response(&profile.inner, response, None).await?;

            Ok(())
        })
    }

    pub fn send_request(
        &self,
        profile: Arc<ProfileHolder>,
        service_endpoint: String,
        routing_keys: Vec<String>,
    ) -> VcxUniFFIResult<()> {
        let mut handler = self.handler.lock()?;
        block_on(async {
            // TODO - do *really* we have to clone...
            *handler = handler
                .clone()
                .send_request(&profile.inner, service_endpoint, routing_keys, None)
                .await?;

            Ok(())
        })
    }

    pub fn send_response(&self, profile: Arc<ProfileHolder>) -> VcxUniFFIResult<()> {
        let mut handler = self.handler.lock()?;
        block_on(async {
            // TODO - do *really* we have to clone...
            *handler = handler.clone().send_response(&profile.inner, None).await?;

            Ok(())
        })
    }

    pub fn send_ack(&self, profile: Arc<ProfileHolder>) -> VcxUniFFIResult<()> {
        let mut handler = self.handler.lock()?;
        block_on(async {
            // TODO - do *really* we have to clone...
            *handler = handler.clone().send_ack(&profile.inner, None).await?;

            Ok(())
        })
    }
}
