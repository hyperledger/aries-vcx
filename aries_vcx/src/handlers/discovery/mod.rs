use indy_sys::WalletHandle;

use crate::did_doc::DidDoc;
use crate::error::VcxResult;

use crate::messages::discovery::disclose::{Disclose, ProtocolDescriptor};
use crate::messages::discovery::query::Query;
use crate::utils::send_message;

pub async fn send_discovery_query(wallet_handle: WalletHandle,
                                  query: Option<String>,
                                  comment: Option<String>,
                                  did_doc: &DidDoc,
                                  pw_vk: &str,
) -> VcxResult<()>
{
    let query_ =
        Query::create()
            .set_query(query)
            .set_comment(comment);
    send_message(wallet_handle, pw_vk.to_string(), did_doc.clone(), query_.to_a2a_message()).await
}

pub async fn respond_discovery_query(wallet_handle: WalletHandle,
                                     query: Query,
                                     did_doc: &DidDoc,
                                     pw_vk: &str,
                                     supported_protocols: Vec<ProtocolDescriptor>,
) -> VcxResult<()>
{
    let disclose = Disclose::create()
        .set_protocols(supported_protocols)
        .set_thread_id(&query.id.0.clone());

    send_message(wallet_handle, pw_vk.to_string(), did_doc.clone(), disclose.to_a2a_message()).await
}