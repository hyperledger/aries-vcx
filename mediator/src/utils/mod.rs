pub use prelude::*;

mod prelude;

/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (axum::http::StatusCode, String)
where
    E: std::error::Error,
{
    (axum::http::StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
