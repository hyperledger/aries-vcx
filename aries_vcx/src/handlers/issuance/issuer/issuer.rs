use std::collections::HashMap;

use crate::error::prelude::*;
use crate::handlers::SendClosure;
use crate::handlers::connection::connection::Connection;
use crate::handlers::issuance::issuer::state_machine::IssuerSM;
use crate::handlers::issuance::messages::CredentialIssuanceMessage;
use crate::libindy::utils::anoncreds::libindy_issuer_create_credential_offer;
use crate::messages::a2a::A2AMessage;
use crate::messages::issuance::credential_offer::{CredentialOffer, OfferInfo};
use crate::messages::issuance::credential_proposal::CredentialProposal;
use crate::messages::issuance::CredentialPreviewData;
use crate::messages::mime_type::MimeType;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Issuer {
    issuer_sm: IssuerSM,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IssuerConfig {
    pub cred_def_id: String,
    pub rev_reg_id: Option<String>,
    pub tails_file: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum IssuerState {
    Initial,
    OfferSet,
    ProposalReceived,
    OfferSent,
    RequestReceived,
    CredentialSent,
    Finished,
    Failed,
}

fn _build_credential_preview(credential_json: &str) -> VcxResult<CredentialPreviewData> {
    trace!("Issuer::_build_credential_preview >>> credential_json: {:?}", credential_json);

    let cred_values: serde_json::Value = serde_json::from_str(credential_json)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Can't deserialize credential preview json. credential_json: {}, error: {:?}", credential_json, err)))?;

    let mut credential_preview = CredentialPreviewData::new();
    match cred_values {
        serde_json::Value::Array(cred_values) => {
            for cred_value in cred_values.iter() {
                let key = cred_value.get("name").ok_or(VcxError::from_msg(VcxErrorKind::InvalidAttributesStructure, format!("No 'name' field in cred_value: {:?}", cred_value)))?;
                let value = cred_value.get("value").ok_or(VcxError::from_msg(VcxErrorKind::InvalidAttributesStructure, format!("No 'value' field in cred_value: {:?}", cred_value)))?;
                credential_preview = credential_preview.add_value(
                    &key.to_string(),
                    &value.to_string(),
                    MimeType::Plain,
                );
            };
        }
        serde_json::Value::Object(values_map) => {
            for item in values_map.iter() {
                let (key, value) = item;
                credential_preview = credential_preview.add_value(
                    key,
                    &value.to_string(),
                    MimeType::Plain,
                );
            }
        }
        _ => {}
    };
    Ok(credential_preview)
}

impl Issuer {
    pub fn create(source_id: &str) -> VcxResult<Issuer> {
        trace!("Issuer::create >>> source_id: {:?}", source_id);
        let issuer_sm = IssuerSM::new(source_id);
        Ok(Issuer { issuer_sm })
    }

    pub fn create_from_proposal(source_id: &str, credential_proposal: &CredentialProposal) -> VcxResult<Issuer> {
        trace!("Issuer::create_from_proposal >>> source_id: {:?}, credential_proposal: {:?}", source_id, credential_proposal);
        let issuer_sm = IssuerSM::from_proposal(source_id, credential_proposal);
        Ok(Issuer { issuer_sm })
    }

    pub fn build_credential_offer_msg(&mut self, offer_info: OfferInfo, comment: Option<String>) -> VcxResult<()> {
        let credential_preview = _build_credential_preview(&offer_info.credential_json)?;
        let libindy_cred_offer = libindy_issuer_create_credential_offer(&offer_info.cred_def_id)?;
        let cred_offer_msg = CredentialOffer::create()
            .set_id(&self.issuer_sm.thread_id()?)
            .set_offers_attach(&libindy_cred_offer)?
            .set_credential_preview_data(credential_preview)
            .set_comment(comment);
        self.issuer_sm = self.issuer_sm.clone().set_offer(cred_offer_msg, &offer_info)?;
        Ok(())
    }

    pub fn get_credential_offer_msg(&self) -> VcxResult<A2AMessage> {
        let offer = self.issuer_sm.get_credential_offer()?;
        Ok(offer.to_a2a_message())
    }

    pub fn mark_credential_offer_msg_sent(&mut self) -> VcxResult<()> {
        self.issuer_sm = self.issuer_sm.clone().mark_credential_offer_msg_sent()?;
        Ok(())
    }

    pub async fn send_credential_offer(&mut self, send_message: SendClosure) -> VcxResult<()> {
        if self.issuer_sm.get_state() == IssuerState::OfferSet {
            let cred_offer_msg = self.get_credential_offer_msg()?;
            send_message(cred_offer_msg).await?;
            self.issuer_sm = self.issuer_sm.clone().mark_credential_offer_msg_sent()?;
        } else {
            return Err(VcxError::from_msg(
                VcxErrorKind::InvalidState,
                format!("Can't send credential offer in state {:?}", self.issuer_sm.get_state())
            ));
        }
        Ok(())
    }

    pub async fn send_credential(&mut self, send_message: SendClosure) -> VcxResult<()> {
        self.step(CredentialIssuanceMessage::CredentialSend(), Some(send_message)).await
    }

    pub fn get_state(&self) -> IssuerState {
        self.issuer_sm.get_state()
    }

    pub fn get_source_id(&self) -> VcxResult<String> {
        Ok(self.issuer_sm.get_source_id())
    }

    pub fn is_terminal_state(&self) -> bool {
        self.issuer_sm.is_terminal_state()
    }

    pub fn find_message_to_handle(&self, messages: HashMap<String, A2AMessage>) -> Option<(String, A2AMessage)> {
        self.issuer_sm.find_message_to_handle(messages)
    }

    pub fn revoke_credential(&self, publish: bool) -> VcxResult<()> {
        self.issuer_sm.revoke(publish)
    }

    pub fn get_rev_reg_id(&self) -> VcxResult<String> {
        self.issuer_sm.get_rev_reg_id()
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        self.issuer_sm.thread_id()
    }

    pub fn get_proposal(&self) -> VcxResult<CredentialProposal> {
        self.issuer_sm.get_proposal()
    }

    pub fn is_revokable(&self) -> VcxResult<bool> {
        self.issuer_sm.is_revokable()
    }

    pub fn get_credential_status(&self) -> VcxResult<u32> {
        Ok(self.issuer_sm.credential_status())
    }

    pub async fn step(&mut self, message: CredentialIssuanceMessage, send_message: Option<SendClosure>) -> VcxResult<()> {
        self.issuer_sm = self.issuer_sm.clone().handle_message(message, send_message).await?;
        Ok(())
    }

    pub async fn update_state(&mut self, connection: &Connection) -> VcxResult<IssuerState> {
        trace!("Issuer::update_state >>>");
        if self.is_terminal_state() { return Ok(self.get_state()); }
        let send_message = connection.send_message_closure()?;

        let messages = connection.get_messages().await?;
        if let Some((uid, msg)) = self.find_message_to_handle(messages) {
            self.step(msg.into(), Some(send_message)).await?;
            connection.update_message_status(&uid).await?;
        }
        Ok(self.get_state())
    }
}

#[cfg(test)]
pub mod test {
    use crate::handlers::issuance::issuer::state_machine::test::_send_message;
    use crate::messages::issuance::credential_offer::test_utils::{_offer_info, _offer_info_unrevokable};
    use crate::messages::issuance::credential_proposal::test_utils::_credential_proposal;
    use crate::messages::issuance::credential_request::test_utils::_credential_request;
    use crate::utils::devsetup::SetupMocks;

    use super::*;

    fn _cred_data() -> String {
        json!({"name": "alice"}).to_string()
    }

    fn _issuer() -> Issuer {
        Issuer::create("test_source_id").unwrap()
    }

    fn _issuer_revokable_from_proposal() -> Issuer {
        Issuer::create_from_proposal("test_source_id", &_credential_proposal()).unwrap()
    }

    fn _send_message_but_fail() -> Option<SendClosure> {
        Some(Box::new(|_: A2AMessage| Box::pin(async { Err(VcxError::from(VcxErrorKind::IOError)) })))
    }

    impl Issuer {
        fn to_offer_sent_state_unrevokable(mut self) -> Issuer {
            self.build_credential_offer_msg(_offer_info_unrevokable(), None).unwrap();
            self.mark_credential_offer_msg_sent().unwrap();
            self
        }

        async fn to_request_received_state(mut self) -> Issuer {
            self = self.to_offer_sent_state_unrevokable();
            self.step(CredentialIssuanceMessage::CredentialRequest(_credential_request()), _send_message()).await.unwrap();
            self
        }

        async fn to_finished_state_unrevokable(mut self) -> Issuer {
            self = self.to_request_received_state().await;
            self.step(CredentialIssuanceMessage::CredentialSend(), _send_message()).await.unwrap();
            self
        }
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_cant_revoke_without_revocation_details() {
        let _setup = SetupMocks::init();
        let issuer = _issuer().to_finished_state_unrevokable().await;
        assert_eq!(IssuerState::Finished, issuer.get_state());
        let revoc_result = issuer.revoke_credential(true);
        assert_eq!(revoc_result.unwrap_err().kind(), VcxErrorKind::InvalidRevocationDetails)
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn test_credential_can_be_resent_after_failure() {
        let _setup = SetupMocks::init();
        let mut issuer = _issuer().to_request_received_state().await;
        assert_eq!(IssuerState::RequestReceived, issuer.get_state());

        let send_result = issuer.send_credential(_send_message_but_fail().unwrap()).await;
        assert_eq!(send_result.is_err(), true);
        assert_eq!(IssuerState::RequestReceived, issuer.get_state());

        let send_result = issuer.send_credential(_send_message().unwrap()).await;
        assert_eq!(send_result.is_err(), false);
        assert_eq!(IssuerState::Finished, issuer.get_state());
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn exchange_credential_from_proposal_without_negotiation() {
        let _setup = SetupMocks::init();
        let mut issuer = _issuer_revokable_from_proposal();
        assert_eq!(IssuerState::ProposalReceived, issuer.get_state());

        issuer.build_credential_offer_msg(_offer_info(), Some("comment".into())).unwrap();
        issuer.send_credential_offer(_send_message().unwrap()).await.unwrap();
        assert_eq!(IssuerState::OfferSent, issuer.get_state());

        let messages = map!(
            "key_1".to_string() => A2AMessage::CredentialRequest(_credential_request())
        );
        let (_, msg) = issuer.find_message_to_handle(messages).unwrap();
        issuer.step(msg.into(), _send_message()).await.unwrap();
        assert_eq!(IssuerState::RequestReceived, issuer.get_state());

        issuer.send_credential(_send_message().unwrap()).await.unwrap();
        assert_eq!(IssuerState::Finished, issuer.get_state());
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn exchange_credential_from_proposal_with_negotiation() {
        let _setup = SetupMocks::init();
        let mut issuer = _issuer_revokable_from_proposal();
        assert_eq!(IssuerState::ProposalReceived, issuer.get_state());

        issuer.build_credential_offer_msg(_offer_info(), Some("comment".into())).unwrap();
        issuer.send_credential_offer(_send_message().unwrap()).await.unwrap();
        assert_eq!(IssuerState::OfferSent, issuer.get_state());

        let messages = map!(
            "key_1".to_string() => A2AMessage::CredentialProposal(_credential_proposal())
        );
        let (_, msg) = issuer.find_message_to_handle(messages).unwrap();
        issuer.step(msg.into(), _send_message()).await.unwrap();
        assert_eq!(IssuerState::ProposalReceived, issuer.get_state());

        issuer.build_credential_offer_msg(_offer_info(), Some("comment".into())).unwrap();
        issuer.send_credential_offer(_send_message().unwrap()).await.unwrap();
        assert_eq!(IssuerState::OfferSent, issuer.get_state());

        let messages = map!(
            "key_1".to_string() => A2AMessage::CredentialRequest(_credential_request())
        );
        let (_, msg) = issuer.find_message_to_handle(messages).unwrap();
        issuer.step(msg.into(), _send_message()).await.unwrap();
        assert_eq!(IssuerState::RequestReceived, issuer.get_state());

        issuer.send_credential(_send_message().unwrap()).await.unwrap();
        assert_eq!(IssuerState::Finished, issuer.get_state());
    }

    #[tokio::test]
    #[cfg(feature = "general_test")]
    async fn issuer_cant_send_offer_twice() {
        let _setup = SetupMocks::init();
        let mut issuer = _issuer().to_offer_sent_state_unrevokable();
        assert_eq!(IssuerState::OfferSent, issuer.get_state());

        let res = issuer.send_credential_offer(_send_message_but_fail().unwrap()).await;
        assert_eq!(IssuerState::OfferSent, issuer.get_state());
        assert!(res.is_err());
    }
}
