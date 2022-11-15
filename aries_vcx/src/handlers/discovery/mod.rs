use std::sync::Arc;

use messages::did_doc::DidDoc;
use crate::error::VcxResult;

use messages::discovery::disclose::{Disclose, ProtocolDescriptor};
use messages::discovery::query::Query;
use crate::utils::send_message;
use crate::plugins::wallet::base_wallet::BaseWallet;

pub async fn send_discovery_query(
    wallet: &Arc<dyn BaseWallet>,
    query: Option<String>,
    comment: Option<String>,
    did_doc: &DidDoc,
    pw_vk: &str,
) -> VcxResult<()> {
    let query_ = Query::create().set_query(query).set_comment(comment).set_out_time();
    send_message(
        Arc::clone(wallet),
        pw_vk.to_string(),
        did_doc.clone(),
        query_.to_a2a_message(),
    )
    .await
}

pub async fn respond_discovery_query(
    wallet: &Arc<dyn BaseWallet>,
    query: Query,
    did_doc: &DidDoc,
    pw_vk: &str,
    supported_protocols: Vec<ProtocolDescriptor>,
) -> VcxResult<()> {
    let disclose = Disclose::create()
        .set_protocols(supported_protocols)
        .set_thread_id(&query.id.0.clone())
        .set_out_time();

    send_message(
        Arc::clone(wallet),
        pw_vk.to_string(),
        did_doc.clone(),
        disclose.to_a2a_message(),
    )
    .await
}
