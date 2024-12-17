use indy_vdr_proxy_client::error::VdrProxyClientError;

use super::error::VcxLedgerError;

impl From<VdrProxyClientError> for VcxLedgerError {
    fn from(err: VdrProxyClientError) -> Self {
        Self::InvalidLedgerResponse(err.to_string())
    }
}
