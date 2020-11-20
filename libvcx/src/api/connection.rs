use crate::service::connection::create_connection;
use crate::error::VcxResult;

pub fn api_connection_create(source_id: &str) -> VcxResult<u32> {
    info!("vcx_connection_create >>>");
    create_connection(&source_id)
}