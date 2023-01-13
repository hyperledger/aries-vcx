use std::sync::{Arc, Mutex};

use aries_vcx::{
    handlers::connection::connection::Connection as VcxConnection,
    messages::{diddoc::aries::diddoc::AriesDidDoc, protocols::connection::invite::Invitation},
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

    pub fn process_invite(&self, invitation: Invitation) -> VcxUniFFIResult<()> {
        let mut handler = self.handler.lock()?;
        // TODO - do *really* we have to clone...
        *handler = handler.clone().process_invite(invitation).unwrap();

        Ok(())
    }
}
