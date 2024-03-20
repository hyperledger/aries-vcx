use indy_vdr_proxy_client::error::VdrProxyClientError;

use super::error::VcxLedgerError;

impl From<VdrProxyClientError> for VcxLedgerError {
    fn from(_err: VdrProxyClientError) -> Self {
        Self::InvalidLedgerResponse
    }
}
