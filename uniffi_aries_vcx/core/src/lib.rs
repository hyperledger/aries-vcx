uniffi::include_scaffolding!("vcx");

pub mod core;
pub mod errors;
pub mod handlers;
pub mod runtime;

use crate::core::profile::*;
use crate::errors::error::*;
use aries_vcx::protocols::connection::pairwise_info::PairwiseInfo;
use aries_vcx::aries_vcx_core::wallet::indy_wallet::WalletConfig;
use diddoc_legacy::{
    aries::service::AriesService,
    w3c::model::{Authentication, Ed25519PublicKey},
};
use handlers::connection::{*, connection::*};
