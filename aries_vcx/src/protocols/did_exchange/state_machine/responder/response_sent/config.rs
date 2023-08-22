use std::sync::Arc;

use aries_vcx_core::wallet::base_wallet::BaseWallet;
use did_resolver_registry::ResolverRegistry;
use messages::msg_fields::protocols::did_exchange::request::Request;
use public_key::Key;
use url::Url;

pub struct ReceiveRequestConfig {
    pub wallet: Arc<dyn BaseWallet>,
    pub resolver_registry: Arc<ResolverRegistry>,
    pub request: Request,
    pub service_endpoint: Url,
    pub routing_keys: Vec<String>,
    pub invitation_id: String,
    pub invitation_key: Key,
}
