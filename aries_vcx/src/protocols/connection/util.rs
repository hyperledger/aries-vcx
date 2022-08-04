use std::clone::Clone;
use std::future::Future;

use indy_sys::WalletHandle;

use crate::error::prelude::*;
use crate::global::settings;
use crate::messages::a2a::A2AMessage;
use crate::did_doc::DidDoc;
use crate::messages::trust_ping::ping::Ping;
use crate::messages::trust_ping::ping_response::PingResponse;

fn _build_ping_response(ping: &Ping) -> PingResponse {
    PingResponse::create().set_thread_id(
        &ping.thread.as_ref().and_then(|thread| thread.thid.clone()).unwrap_or(ping.id.0.clone()))
}

pub async fn handle_ping<F, T>(wallet_handle: WalletHandle,
                               ping: &Ping,
                               pw_vk: &str,
                               did_doc: &DidDoc,
                               send_message: F,
) -> VcxResult<()>
    where
        F: Fn(WalletHandle, String, DidDoc, A2AMessage) -> T,
        T: Future<Output=VcxResult<()>>
{
    if ping.response_requested {
        send_message(wallet_handle, pw_vk.to_string(), did_doc.clone(), _build_ping_response(ping).to_a2a_message()).await?;
    }
    Ok(())
}

pub fn verify_thread_id(thread_id: &str, message: &A2AMessage) -> VcxResult<()> {
    if !settings::indy_mocks_enabled() && !message.thread_id_matches(thread_id) {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot handle message {:?}: thread id does not match, expected {:?}", message, thread_id)));
    };
    Ok(())
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod unit_tests {
    use crate::messages::trust_ping::ping::unit_tests::{_ping, _ping_no_thread};
    use crate::messages::trust_ping::ping_response::unit_tests::_ping_response;

    use super::*;

    #[test]
    fn test_build_ping_response_works() {
        assert_eq!(_build_ping_response(&_ping()).get_thread_id(), _ping_response().get_thread_id());
        assert_eq!(_build_ping_response(&_ping_no_thread()).get_thread_id(), _ping_response().get_thread_id());
        assert_eq!(_build_ping_response(&_ping_no_thread()).get_thread_id(), _ping_no_thread().id.0);
    }
}
