//! Common components for V1.X DIDExchange messages (v1.0 & v1.1).
//! Necessary to prevent duplicated code, since most types between v1.0 & v1.1 are identical

pub mod complete;
pub mod problem_report;
pub mod request;
pub mod response;
