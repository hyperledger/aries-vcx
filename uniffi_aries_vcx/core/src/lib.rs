uniffi::include_scaffolding!("vcx");

pub mod core;
pub mod errors;
pub mod handlers;
pub mod runtime;

use crate::core::profile::*;
use crate::errors::error::*;
use aries_vcx::{aries_vcx_core::indy::wallet::WalletConfig, protocols::connection::pairwise_info::PairwiseInfo};
use diddoc::{
    aries::service::AriesService,
    w3c::model::{Authentication, Ed25519PublicKey},
};
use handlers::connection::{connection::*, *};
