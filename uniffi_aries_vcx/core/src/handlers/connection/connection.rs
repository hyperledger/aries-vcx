use std::sync::{Arc, Mutex};

use aries_vcx::{
    messages::diddoc::aries::diddoc::AriesDidDoc,
    protocols::connection::pairwise_info::PairwiseInfo,
    protocols::connection::Connection as VcxConnection,
    protocols::connection::{GenericConnection as VcxGenericConnection, ThinState},
};

use crate::{
    core::{http_client::HttpClient, profile::ProfileHolder},
    errors::error::VcxUniFFIResult,
    runtime::block_on,
};
pub struct Connection {
    handler: Mutex<VcxGenericConnection>,
}

// seperate function since uniffi can't handle constructors with results
pub fn create_inviter(profile: Arc<ProfileHolder>) -> VcxUniFFIResult<Arc<Connection>> {
    block_on(async {
        let pairwise_info = PairwiseInfo::create(&profile.inner.inject_wallet()).await?;
        let connection = VcxConnection::new_inviter(String::new(), pairwise_info);
        let handler = Mutex::new(VcxGenericConnection::from(connection));
        Ok(Arc::new(Connection { handler }))
    })
}

// seperate function since uniffi can't handle constructors with results
pub fn create_invitee(profile: Arc<ProfileHolder>, did_doc: AriesDidDoc) -> VcxUniFFIResult<Arc<Connection>> {
    block_on(async {
        let pairwise_info = PairwiseInfo::create(&profile.inner.inject_wallet()).await?;
        let connection = VcxConnection::new_invitee(String::new(), pairwise_info);
        let handler = Mutex::new(VcxGenericConnection::from(connection));

        Ok(Arc::new(Connection { handler }))
    })
}

impl Connection {
    pub fn get_state(&self) -> VcxUniFFIResult<ThinState> {
        let handler = self.handler.lock()?;
        Ok(handler.state())
    }

    pub fn pairwise_info(&self) -> VcxUniFFIResult<PairwiseInfo> {
        let handler = self.handler.lock()?;
        Ok(handler.pairwise_info().clone())
    }

    // NOTE : using string here out of laziness. We could have type this,
    // but UniFFI does not support structs with unnamed fields. So we'd have to
    // wrap these types
    // here invitation -> aries_vcx::Invitation
    pub fn accept_invitation(&self, profile: Arc<ProfileHolder>, invitation: String) -> VcxUniFFIResult<()> {
        let mut handler = self.handler.lock()?;
        let invitation = serde_json::from_str(&invitation)?;

        let connection = VcxConnection::try_from(handler.clone())?;

        block_on(async {
            let new_conn = connection.accept_invitation(&profile.inner, invitation).await?;
            *handler = VcxGenericConnection::from(new_conn);
            Ok(())
        })
    }

    // NOTE : using string here out of laziness. We could have type this,
    // but UniFFI does not support structs with unnamed fields. So we'd have to
    // wrap these types
    // here request -> aries_vcx::Request
    pub fn handle_request(
        &self,
        profile: Arc<ProfileHolder>,
        request: String,
        service_endpoint: String,
        routing_keys: Vec<String>,
    ) -> VcxUniFFIResult<()> {
        let mut handler = self.handler.lock()?;
        let request = serde_json::from_str(&request)?;

        let connection = VcxConnection::try_from(handler.clone())?;

        block_on(async {
            let new_conn = connection
                .handle_request(
                    &profile.inner.inject_wallet(),
                    request,
                    service_endpoint,
                    routing_keys,
                    &HttpClient,
                )
                .await?;

            *handler = VcxGenericConnection::from(new_conn);

            Ok(())
        })
    }

    // NOTE : using string here out of laziness. We could have type this,
    // but UniFFI does not support structs with unnamed fields. So we'd have to
    // wrap these types
    // here request -> aries_vcx::Request
    pub fn handle_response(&self, profile: Arc<ProfileHolder>, response: String) -> VcxUniFFIResult<()> {
        let mut handler = self.handler.lock()?;
        let response = serde_json::from_str(&response)?;

        let connection = VcxConnection::try_from(handler.clone())?;

        block_on(async {
            let new_conn = connection
                .handle_response(&profile.inner.inject_wallet(), response, &HttpClient)
                .await?;
            *handler = VcxGenericConnection::from(new_conn);

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

        let connection = VcxConnection::try_from(handler.clone())?;

        block_on(async {
            let new_conn = connection
                .send_request(
                    &profile.inner.inject_wallet(),
                    service_endpoint,
                    routing_keys,
                    &HttpClient,
                )
                .await?;
            *handler = VcxGenericConnection::from(new_conn);

            Ok(())
        })
    }

    pub fn send_response(&self, profile: Arc<ProfileHolder>) -> VcxUniFFIResult<()> {
        let mut handler = self.handler.lock()?;

        let connection = VcxConnection::try_from(handler.clone())?;

        block_on(async {
            let new_conn = connection
                .send_response(&profile.inner.inject_wallet(), &HttpClient)
                .await?;

            *handler = VcxGenericConnection::from(new_conn);

            Ok(())
        })
    }

    pub fn send_ack(&self, profile: Arc<ProfileHolder>) -> VcxUniFFIResult<()> {
        let mut handler = self.handler.lock()?;

        let connection = VcxConnection::try_from(handler.clone())?;

        block_on(async {
            let new_conn = connection.send_ack(&profile.inner.inject_wallet(), &HttpClient).await?;
            *handler = VcxGenericConnection::from(new_conn);

            Ok(())
        })
    }
}
