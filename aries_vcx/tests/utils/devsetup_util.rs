use std::sync::Arc;

use futures::future::BoxFuture;

#[cfg(feature = "modular_libs")]
use aries_vcx::core::profile::modular_libs_profile::ModularLibsProfile;
use aries_vcx::core::profile::profile::Profile;
use aries_vcx::core::profile::vdrtools_profile::VdrtoolsProfile;
use aries_vcx::global::settings;
use aries_vcx_core::indy::ledger::pool::indy_open_pool;
#[cfg(feature = "modular_libs")]
use aries_vcx_core::ledger::request_submitter::vdr_ledger::LedgerPoolConfig;
use aries_vcx_core::wallet::base_wallet::BaseWallet;
use aries_vcx_core::wallet::indy::wallet::{create_wallet_with_master_secret, open_wallet};
use aries_vcx_core::wallet::indy::{IndySdkWallet, WalletConfig};
use aries_vcx_core::WalletHandle;

use crate::utils::devsetup_alice::Alice;

#[cfg(test)]
pub mod test_utils {
    use std::collections::HashMap;
    use std::sync::Arc;

    use agency_client::api::downloaded_message::DownloadedMessage;
    #[cfg(feature = "modular_libs")]
    use aries_vcx::core::profile::modular_libs_profile::ModularLibsProfile;
    use aries_vcx::errors::error::{AriesVcxError, AriesVcxErrorKind, VcxResult};
    #[cfg(feature = "modular_libs")]
    use aries_vcx_core::ledger::request_submitter::vdr_ledger::LedgerPoolConfig;
    use aries_vcx_core::wallet::indy::wallet::{close_wallet, delete_wallet};
    use aries_vcx_core::wallet::indy::WalletConfig;
    use aries_vcx_core::WalletHandle;
    use messages::msg_fields::protocols::connection::Connection;
    use messages::msg_fields::protocols::cred_issuance::CredentialIssuance;
    use messages::msg_fields::protocols::present_proof::PresentProof;
    use messages::AriesMessage;

    #[derive(Debug)]
    pub struct VcxAgencyMessage {
        pub uid: String,
        pub decrypted_msg: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum PayloadKinds {
        CredOffer,
        CredReq,
        Cred,
        Proof,
        ProofRequest,
        ConnRequest,
        Other(String),
    }

    fn determine_message_type(a2a_message: AriesMessage) -> PayloadKinds {
        debug!("determine_message_type >>> a2a_message: {:?}", a2a_message);
        match a2a_message.clone() {
            AriesMessage::PresentProof(PresentProof::RequestPresentation(_)) => PayloadKinds::ProofRequest,
            AriesMessage::CredentialIssuance(CredentialIssuance::OfferCredential(_)) => PayloadKinds::CredOffer,
            AriesMessage::CredentialIssuance(CredentialIssuance::IssueCredential(_)) => PayloadKinds::Cred,
            AriesMessage::PresentProof(PresentProof::Presentation(_)) => PayloadKinds::Proof,
            AriesMessage::Connection(Connection::Request(_)) => PayloadKinds::ConnRequest,
            _msg => PayloadKinds::Other(String::from("aries")),
        }
    }

    fn str_message_to_a2a_message(message: &str) -> VcxResult<AriesMessage> {
        Ok(serde_json::from_str(message).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidJson,
                format!("Cannot deserialize A2A message: {}", err),
            )
        })?)
    }

    fn str_message_to_payload_type(message: &str) -> VcxResult<PayloadKinds> {
        let a2a_message = str_message_to_a2a_message(message)?;
        Ok(determine_message_type(a2a_message))
    }

    pub async fn filter_messages(
        messages: Vec<DownloadedMessage>,
        filter_msg_type: PayloadKinds,
    ) -> Option<VcxAgencyMessage> {
        for message in messages.into_iter() {
            let decrypted_msg = &message.decrypted_msg;
            let msg_type = str_message_to_payload_type(decrypted_msg).unwrap();
            if filter_msg_type == msg_type {
                return Some(VcxAgencyMessage {
                    uid: message.uid,
                    decrypted_msg: decrypted_msg.to_string(),
                });
            }
        }
        None
    }
}
