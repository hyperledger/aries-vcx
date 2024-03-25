use std::{
    collections::HashMap,
    sync::{Mutex, RwLock},
};

use aries_vcx_agent::{aries_vcx::messages::AriesMessage, Agent as AriesAgent};
#[cfg(feature = "askar_wallet")]
use aries_vcx_wallet::wallet::askar::AskarWallet;
#[cfg(feature = "vdrtools_wallet")]
use aries_vcx_wallet::wallet::indy::IndySdkWallet;

use crate::Status;

pub struct HarnessAgent {
    #[cfg(feature = "vdrtools_wallet")]
    pub aries_agent: AriesAgent<IndySdkWallet>,
    #[cfg(feature = "askar_wallet")]
    pub aries_agent: AriesAgent<AskarWallet>,
    pub status: Status,
    // did-exchange specific
    // todo: extra didx specific AATH service
    pub didx_msg_buffer: RwLock<Vec<AriesMessage>>,
    pub didx_pthid_to_thid: Mutex<HashMap<String, String>>,
}

impl HarnessAgent {
    pub fn new(
        #[cfg(feature = "vdrtools_wallet")] aries_agent: &AriesAgent<IndySdkWallet>,
        #[cfg(feature = "askar_wallet")] aries_agent: &AriesAgent<AskarWallet>,
        status: Status,
        msg_buffer: RwLock<Vec<AriesMessage>>,
        pthid_to_thid: Mutex<HashMap<String, String>>,
    ) -> Self {
        Self {
            aries_agent: aries_agent.clone(),
            status,
            didx_msg_buffer: msg_buffer,
            didx_pthid_to_thid: pthid_to_thid,
        }
    }
}
