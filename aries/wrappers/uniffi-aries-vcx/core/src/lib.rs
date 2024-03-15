#[cfg(feature = "vdrtools_wallet")]
uniffi::include_scaffolding!("vcx_indy");

#[cfg(feature = "askar_wallet")]
uniffi::include_scaffolding!("vcx_askar");

pub mod core;
pub mod errors;
pub mod handlers;
pub mod runtime;

#[cfg(feature = "askar_wallet")]
use aries_vcx::aries_vcx_core::wallet::askar::{
    askar_wallet_config::AskarWalletConfig,
    key_method::{ArgonLevel, AskarKdfMethod, KeyMethod},
};
#[cfg(feature = "vdrtools_wallet")]
use aries_vcx::aries_vcx_core::wallet::indy::indy_wallet_config::IndyWalletConfig;
use aries_vcx::protocols::connection::pairwise_info::PairwiseInfo;
use handlers::{connection::*, holder::*};

use crate::{
    core::{anoncreds::*, profile::*, unpack_message::*},
    errors::error::*,
    profile::new_indy_profile,
};
