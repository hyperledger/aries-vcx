use serde::Deserialize;

pub mod connection;
pub mod credential_definition;
pub mod didcomm;
pub mod general;
pub mod issuance;
pub mod out_of_band;
pub mod presentation;
pub mod revocation;
pub mod schema;
pub mod did_exchange;

#[derive(Deserialize)]
pub struct Request<T> {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub data: T,
}
