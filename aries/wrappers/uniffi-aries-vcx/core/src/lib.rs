uniffi::include_scaffolding!("vcx");

pub mod core;
pub mod errors;
pub mod handlers;
pub mod runtime;

use aries_vcx::{
    aries_vcx_core::wallet::indy::wallet_config::WalletConfig,
    protocols::connection::pairwise_info::PairwiseInfo,
};
use handlers::{connection::*, holder::*};

use crate::{
    core::{anoncreds::*, profile::*, unpack_message::*},
    errors::error::*,
};
