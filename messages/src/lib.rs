#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate strum_macros;

#[macro_use]
pub mod thread;

#[macro_use]
pub mod a2a;

#[macro_use]
pub mod ack;

pub mod attachment;
pub mod basic_message;
pub mod connection;
pub mod discovery;
pub mod error;
pub mod problem_report;
pub mod forward;
pub mod issuance;
pub mod revocation_notification;
pub mod localization;
pub mod mime_type;
pub mod out_of_band;
pub mod proof_presentation;
pub mod status;
pub mod timing;
pub mod trust_ping;
pub mod did_doc;
pub mod actors;
pub mod utils;
