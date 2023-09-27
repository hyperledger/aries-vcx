use aries_vcx_core::errors::error::AriesVcxCoreError;
pub use prelude::*;

pub mod prelude;
pub mod structs;

///// Utility function for mapping any error into a `500 Internal Server Error`
///// response.
// fn internal_error<E>(err: E) -> (axum::http::StatusCode, String)
// where
//     E: std::error::Error,
// {
//     (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
// }

// impl From<AriesVcxCoreError> for String {}
