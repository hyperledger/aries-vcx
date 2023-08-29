use diddoc_legacy::aries::diddoc::AriesDidDoc;
use std::sync::{Arc, Mutex};

use aries_vcx::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::connection::pairwise_info::PairwiseInfo,
    protocols::connection::Connection as VcxConnection,
    protocols::connection::GenericConnection as VcxGenericConnection,
};
use url::Url;

use crate::{
    core::{http_client::HttpClient, profile::ProfileHolder},
    errors::error::VcxUniFFIResult,
    runtime::block_on,
};

use super::ConnectionState;
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
pub fn create_invitee(profile: Arc<ProfileHolder>, did_doc: String) -> VcxUniFFIResult<Arc<Connection>> {
    block_on(async {
        let _did_doc: AriesDidDoc = serde_json::from_str(&did_doc)?;
        let pairwise_info = PairwiseInfo::create(&profile.inner.inject_wallet()).await?;
        let connection = VcxConnection::new_invitee(String::new(), pairwise_info);
        let handler = Mutex::new(VcxGenericConnection::from(connection));

        Ok(Arc::new(Connection { handler }))
    })
}

impl Connection {
    pub fn get_state(&self) -> VcxUniFFIResult<ConnectionState> {
        let handler = self.handler.lock()?;
        Ok(ConnectionState::from(handler.state()))
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
            let new_conn = connection
                .accept_invitation(&profile.inner.inject_indy_ledger_read(), invitation)
                .await?;
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
        let url = Url::parse(&service_endpoint)
            .map_err(|err| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidUrl, err.to_string()))?;

        block_on(async {
            let new_conn = connection
                .handle_request(&profile.inner.inject_wallet(), request, url, routing_keys, &HttpClient)
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
        let url = Url::parse(&service_endpoint)
            .map_err(|err| AriesVcxError::from_msg(AriesVcxErrorKind::InvalidUrl, err.to_string()))?;

        block_on(async {
            let mut connection = connection.prepare_request(url, routing_keys)
                .await?;
            let request = connection.get_request().clone();
            connection.send_message(&profile.inner.inject_wallet(), &request.into(), &HttpClient)
                .await?;
            *handler = VcxGenericConnection::from(connection);
            Ok(())
        })
    }

    pub fn send_response(&self, profile: Arc<ProfileHolder>) -> VcxUniFFIResult<()> {
        let mut handler = self.handler.lock()?;

        let connection = VcxConnection::try_from(handler.clone())?;

        block_on(async {
            let response = connection.get_connection_response_msg();
            connection
                .send_message(&profile.inner.inject_wallet(), &response.into(), &HttpClient)
                .await?;

            *handler = VcxGenericConnection::from(connection);

            Ok(())
        })
    }

    pub fn send_ack(&self, profile: Arc<ProfileHolder>) -> VcxUniFFIResult<()> {
        let mut handler = self.handler.lock()?;

        let connection = VcxConnection::try_from(handler.clone())?;

        block_on(async {
            connection.send_message(&profile.inner.inject_wallet(), &connection.get_ack().into(), &HttpClient).await?;
            Ok(())
        })
    }
}
