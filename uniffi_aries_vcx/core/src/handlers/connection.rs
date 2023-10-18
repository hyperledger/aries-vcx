use std::sync::{Arc, Mutex};

use aries_vcx::{
    errors::error::{AriesVcxError, AriesVcxErrorKind},
    protocols::connection::{
        initiation_type::Inviter, inviter::states::completed::Completed,
        pairwise_info::PairwiseInfo, Connection as VcxConnection,
        GenericConnection as VcxGenericConnection, ThinState,
    },
};
use url::Url;

use crate::{
    core::{http_client::HttpClient, profile::ProfileHolder},
    errors::error::VcxUniFFIResult,
    runtime::block_on,
};

/// Wraps [ThinState], as uniffi cannot process enums with un-named fields
pub struct ConnectionState {
    pub role: ConnectionRole,
    pub protocol_state: ConnectionProtocolState,
}

pub enum ConnectionRole {
    Invitee,
    Inviter,
}

pub enum ConnectionProtocolState {
    Initial,
    Invited,
    Requested,
    Responded,
    Completed,
}

impl From<ThinState> for ConnectionState {
    fn from(x: ThinState) -> Self {
        match x {
            ThinState::Inviter(state) => ConnectionState {
                role: ConnectionRole::Inviter,
                protocol_state: ConnectionProtocolState::from(state),
            },
            ThinState::Invitee(state) => ConnectionState {
                role: ConnectionRole::Invitee,
                protocol_state: ConnectionProtocolState::from(state),
            },
        }
    }
}

impl From<aries_vcx::protocols::connection::State> for ConnectionProtocolState {
    fn from(value: aries_vcx::protocols::connection::State) -> Self {
        match value {
            aries_vcx::protocols::connection::State::Initial => ConnectionProtocolState::Initial,
            aries_vcx::protocols::connection::State::Invited => ConnectionProtocolState::Invited,
            aries_vcx::protocols::connection::State::Requested => {
                ConnectionProtocolState::Requested
            }
            aries_vcx::protocols::connection::State::Responded => {
                ConnectionProtocolState::Responded
            }
            aries_vcx::protocols::connection::State::Completed => {
                ConnectionProtocolState::Completed
            }
        }
    }
}

pub struct Connection {
    handler: Mutex<VcxGenericConnection>,
}

// seperate function since uniffi can't handle constructors with results
pub fn create_inviter(profile: Arc<ProfileHolder>) -> VcxUniFFIResult<Arc<Connection>> {
    block_on(async {
        let pairwise_info = PairwiseInfo::create(profile.inner.wallet()).await?;
        let connection = VcxConnection::new_inviter(String::new(), pairwise_info);
        let handler = Mutex::new(VcxGenericConnection::from(connection));
        Ok(Arc::new(Connection { handler }))
    })
}

// seperate function since uniffi can't handle constructors with results
pub fn create_invitee(profile: Arc<ProfileHolder>) -> VcxUniFFIResult<Arc<Connection>> {
    block_on(async {
        let pairwise_info = PairwiseInfo::create(profile.inner.wallet()).await?;
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
    pub fn accept_invitation(
        &self,
        profile: Arc<ProfileHolder>,
        invitation: String,
    ) -> VcxUniFFIResult<()> {
        let mut handler = self.handler.lock()?;
        let invitation = serde_json::from_str(&invitation)?;

        let connection = VcxConnection::try_from(handler.clone())?;

        block_on(async {
            let new_conn = connection
                .accept_invitation(profile.inner.ledger_read(), invitation)
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
        let url = Url::parse(&service_endpoint).map_err(|err| {
            AriesVcxError::from_msg(AriesVcxErrorKind::InvalidUrl, err.to_string())
        })?;

        block_on(async {
            let new_conn = connection
                .handle_request(profile.inner.wallet(), request, url, routing_keys)
                .await?;

            *handler = VcxGenericConnection::from(new_conn);

            Ok(())
        })
    }

    // NOTE : using string here out of laziness. We could have type this,
    // but UniFFI does not support structs with unnamed fields. So we'd have to
    // wrap these types
    // here request -> aries_vcx::Request
    pub fn handle_response(
        &self,
        profile: Arc<ProfileHolder>,
        response: String,
    ) -> VcxUniFFIResult<()> {
        let mut handler = self.handler.lock()?;
        let response = serde_json::from_str(&response)?;

        let connection = VcxConnection::try_from(handler.clone())?;

        block_on(async {
            let new_conn = connection
                .handle_response(profile.inner.wallet(), response)
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
        let url = Url::parse(&service_endpoint).map_err(|err| {
            AriesVcxError::from_msg(AriesVcxErrorKind::InvalidUrl, err.to_string())
        })?;

        block_on(async {
            let connection = connection.prepare_request(url, routing_keys).await?;
            let request = connection.get_request().clone();
            connection
                .send_message(profile.inner.wallet(), &request.into(), &HttpClient)
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
                .send_message(profile.inner.wallet(), &response.into(), &HttpClient)
                .await?;

            *handler = VcxGenericConnection::from(connection);

            Ok(())
        })
    }

    pub fn send_ack(&self, profile: Arc<ProfileHolder>) -> VcxUniFFIResult<()> {
        let handler = self.handler.lock()?;

        let connection = VcxConnection::try_from(handler.clone())?;

        block_on(async {
            connection
                .send_message(
                    profile.inner.wallet(),
                    &connection.get_ack().into(),
                    &HttpClient,
                )
                .await?;
            Ok(())
        })
    }

    pub fn send_message(
        &self,
        profile_holder: Arc<ProfileHolder>,
        message: String,
    ) -> VcxUniFFIResult<()> {
        let message = serde_json::from_str(&message)?;
        let mut handler = self.handler.lock()?;
        let connection: VcxConnection<Inviter, Completed> =
            VcxConnection::try_from(handler.clone())?;

        block_on(async {
            connection
                .send_message(profile_holder.inner.wallet(), &message, &HttpClient)
                .await?;
            Ok(())
        })
    }
}
