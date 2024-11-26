uniffi::include_scaffolding!("vcx");

pub mod core;
pub mod errors;
pub mod handlers;
pub mod runtime;

use aries_vcx::{
    aries_vcx_wallet::wallet::askar::{
        askar_wallet_config::AskarWalletConfig,
        key_method::{ArgonLevel, AskarKdfMethod, KeyMethod},
    },
    protocols::connection::pairwise_info::PairwiseInfo,
};
use handlers::{connection::*, holder::*};

use crate::{
    core::{anoncreds::*, profile::*, unpack_message::*},
    errors::error::*,
    profile::new_indy_profile,
};
