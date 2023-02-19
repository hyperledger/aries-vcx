uniffi_macros::include_scaffolding!("vcx");

pub mod core;
pub mod errors;
pub mod handlers;
pub mod runtime;

use crate::core::profile::*;
use crate::errors::error::*;
use aries_vcx::{
    indy::wallet::WalletConfig,
    messages::diddoc::{
        aries::{diddoc::AriesDidDoc, service::AriesService},
        w3c::model::{Authentication, Ed25519PublicKey},
    },
    protocols::connection::pairwise_info::PairwiseInfo,
};
use handlers::connection::{connection::*, *};
