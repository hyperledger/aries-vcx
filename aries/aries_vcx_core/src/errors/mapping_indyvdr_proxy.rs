use indy_vdr_proxy_client::error::VdrProxyClientError;

use super::error::{AriesVcxCoreError, AriesVcxCoreErrorKind};

impl From<VdrProxyClientError> for AriesVcxCoreError {
    fn from(err: VdrProxyClientError) -> Self {
        AriesVcxCoreError::from_msg(
            AriesVcxCoreErrorKind::InvalidLedgerResponse,
            format!("VdrProxyClient error: {:?}", err),
        )
    }
}
