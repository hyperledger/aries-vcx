use std::collections::HashMap;

use messages::decorators::please_ack::AckOn;
use messages::misc::MimeType;
use messages::msg_fields::protocols::cred_issuance::ack::AckCredential;
use messages::msg_fields::protocols::cred_issuance::propose_credential::ProposeCredential;
use messages::msg_fields::protocols::cred_issuance::request_credential::RequestCredential;
use messages::msg_fields::protocols::cred_issuance::{CredentialAttr, CredentialPreview};
use messages::AriesMessage;
use std::sync::Arc;

use agency_client::agency_client::AgencyClient;
use aries_vcx_core::anoncreds::base_anoncreds::BaseAnonCreds;
use aries_vcx_core::ledger::base_ledger::AnoncredsLedgerRead;
use aries_vcx_core::wallet::base_wallet::BaseWallet;

use crate::errors::error::prelude::*;
use crate::handlers::connection::mediated_connection::MediatedConnection;
use crate::handlers::revocation_notification::sender::RevocationNotificationSender;
use crate::handlers::util::OfferInfo;
use crate::protocols::issuance::actions::CredentialIssuanceAction;
use crate::protocols::issuance::issuer::state_machine::{IssuerSM, IssuerState, RevocationInfoV1};
use crate::protocols::revocation_notification::sender::state_machine::SenderConfigBuilder;
use crate::protocols::SendClosure;

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

fn _build_credential_preview(credential_json: &str) -> VcxResult<CredentialPreview> {
    trace!(
        "Issuer::_build_credential_preview >>> credential_json: {:?}",
        secret!(credential_json)
    );

    let cred_values: serde_json::Value = serde_json::from_str(credential_json).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidJson,
            format!(
                "Can't deserialize credential preview json. credential_json: {}, error: {:?}",
                credential_json, err
            ),
        )
    })?;

    // todo: should throw err if cred_values is not serde_json::Value::Array or serde_json::Value::Object
    let mut credential_preview = CredentialPreview::new(Vec::new());

    match cred_values {
        serde_json::Value::Array(cred_values) => {
            for cred_value in cred_values.iter() {
                let key = cred_value.get("name").ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidAttributesStructure,
                    format!("No 'name' field in cred_value: {:?}", cred_value),
                ))?;
                let value = cred_value.get("value").ok_or(AriesVcxError::from_msg(
                    AriesVcxErrorKind::InvalidAttributesStructure,
                    format!("No 'value' field in cred_value: {:?}", cred_value),
                ))?;

                let mut attr = CredentialAttr::new(
                    key.as_str()
                        .ok_or(AriesVcxError::from_msg(
                            AriesVcxErrorKind::InvalidOption,
                            "Credential value names are currently only allowed to be strings",
                        ))?
                        .to_owned(),
                    value
                        .as_str()
                        .ok_or(AriesVcxError::from_msg(
                            AriesVcxErrorKind::InvalidOption,
                            "Credential values are currently only allowed to be strings",
                        ))?
                        .to_owned(),
                );

                attr.mime_type = Some(MimeType::Plain);
                credential_preview.attributes.push(attr);
            }
        }
        serde_json::Value::Object(values_map) => {
            for item in values_map.iter() {
                let (key, value) = item;

                let mut attr = CredentialAttr::new(
                    key.to_owned(),
                    value
                        .as_str()
                        .ok_or(AriesVcxError::from_msg(
                            AriesVcxErrorKind::InvalidOption,
                            "Credential values are currently only allowed to be strings",
                        ))?
                        .to_owned(),
                );

                attr.mime_type = Some(MimeType::Plain);
                credential_preview.attributes.push(attr);
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

    pub fn create_from_proposal(source_id: &str, credential_proposal: &ProposeCredential) -> VcxResult<Issuer> {
        trace!(
            "Issuer::create_from_proposal >>> source_id: {:?}, credential_proposal: {:?}",
            source_id,
            credential_proposal
        );
        let issuer_sm = IssuerSM::from_proposal(source_id, credential_proposal);
        Ok(Issuer { issuer_sm })
    }

    // todo: "build_credential_offer_msg" should take optional revReg as parameter, build OfferInfo from that
    pub async fn build_credential_offer_msg(
        &mut self,
        anoncreds: &Arc<dyn BaseAnonCreds>,
        offer_info: OfferInfo,
        comment: Option<String>,
    ) -> VcxResult<()> {
        let credential_preview = _build_credential_preview(&offer_info.credential_json)?;
        let libindy_cred_offer = anoncreds
            .issuer_create_credential_offer(&offer_info.cred_def_id)
            .await?;
        self.issuer_sm = self.issuer_sm.clone().build_credential_offer_msg(
            &libindy_cred_offer,
            credential_preview,
            comment,
            &offer_info,
        )?;
        Ok(())
    }

    pub fn get_credential_offer_msg(&self) -> VcxResult<AriesMessage> {
        let offer = self.issuer_sm.get_credential_offer_msg()?;
        Ok(offer.into())
    }

    pub fn mark_credential_offer_msg_sent(&mut self) -> VcxResult<()> {
        self.issuer_sm = self.issuer_sm.clone().mark_credential_offer_msg_sent()?;
        Ok(())
    }

    pub async fn send_credential_offer(&mut self, send_message: SendClosure) -> VcxResult<()> {
        self.issuer_sm = self.issuer_sm.clone().send_credential_offer(send_message).await?;
        Ok(())
    }

    pub fn process_credential_request(&mut self, request: RequestCredential) -> VcxResult<()> {
        self.issuer_sm = self.issuer_sm.clone().receive_request(request)?;
        Ok(())
    }

    pub fn process_credential_ack(&mut self, ack: AckCredential) -> VcxResult<()> {
        self.issuer_sm = self.issuer_sm.clone().receive_ack(ack)?;
        Ok(())
    }

    pub async fn send_credential(
        &mut self,
        anoncreds: &Arc<dyn BaseAnonCreds>,
        send_message: SendClosure,
    ) -> VcxResult<()> {
        self.issuer_sm = self.issuer_sm.clone().send_credential(anoncreds, send_message).await?;
        Ok(())
    }

    pub async fn send_revocation_notification(
        &mut self,
        ack_on: Vec<AckOn>,
        comment: Option<String>,
        send_message: SendClosure,
    ) -> VcxResult<()> {
        // TODO: Check if actually revoked
        if self.issuer_sm.is_revokable() {
            // TODO: Store to allow checking not. status (sent, acked)
            let config = SenderConfigBuilder::default()
                .rev_reg_id(self.get_rev_reg_id()?)
                .cred_rev_id(self.get_rev_id()?)
                .comment(comment)
                .ack_on(ack_on)
                .build()?;
            RevocationNotificationSender::build()
                .send_revocation_notification(config, send_message)
                .await?;
            Ok(())
        } else {
            Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                format!(
                    "Can't send revocation notification in state {:?}, credential is not revokable",
                    self.issuer_sm.get_state()
                ),
            ))
        }
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

    pub fn find_message_to_handle(&self, messages: HashMap<String, AriesMessage>) -> Option<(String, AriesMessage)> {
        self.issuer_sm.find_message_to_handle(messages)
    }

    pub fn get_revocation_id(&self) -> VcxResult<String> {
        self.issuer_sm
            .get_revocation_info()
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Credential has not yet been created",
            ))?
            .cred_rev_id
            .ok_or(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Credential has not yet been created or is irrevocable",
            ))
    }

    pub async fn revoke_credential_local(&self, anoncreds: &Arc<dyn BaseAnonCreds>) -> VcxResult<()> {
        let revocation_info: RevocationInfoV1 = self.issuer_sm.get_revocation_info().ok_or(AriesVcxError::from_msg(
            AriesVcxErrorKind::InvalidState,
            "Credential is not revocable, no revocation info has been found.",
        ))?;
        if let (Some(cred_rev_id), Some(rev_reg_id), Some(tails_file)) = (
            revocation_info.cred_rev_id,
            revocation_info.rev_reg_id,
            revocation_info.tails_file,
        ) {
            anoncreds
                .revoke_credential_local(&tails_file, &rev_reg_id, &cred_rev_id)
                .await?;
        } else {
            return Err(AriesVcxError::from_msg(
                AriesVcxErrorKind::InvalidState,
                "Revocation info is not complete, cannot revoke credential.",
            ));
        }
        Ok(())
    }

    pub fn get_rev_reg_id(&self) -> VcxResult<String> {
        self.issuer_sm.get_rev_reg_id()
    }

    pub fn get_rev_id(&self) -> VcxResult<String> {
        self.issuer_sm.get_rev_id()
    }

    pub fn get_thread_id(&self) -> VcxResult<String> {
        self.issuer_sm.thread_id()
    }

    pub fn get_proposal(&self) -> VcxResult<ProposeCredential> {
        self.issuer_sm.get_proposal()
    }

    pub fn get_credential_status(&self) -> VcxResult<u32> {
        Ok(self.issuer_sm.credential_status())
    }

    pub fn is_revokable(&self) -> bool {
        self.issuer_sm.is_revokable()
    }

    pub async fn is_revoked(&self, ledger: &Arc<dyn AnoncredsLedgerRead>) -> VcxResult<bool> {
        self.issuer_sm.is_revoked(ledger).await
    }

    pub async fn step(
        &mut self,
        anoncreds: &Arc<dyn BaseAnonCreds>,
        message: CredentialIssuanceAction,
        send_message: Option<SendClosure>,
    ) -> VcxResult<()> {
        self.issuer_sm = self
            .issuer_sm
            .clone()
            .handle_message(anoncreds, message, send_message)
            .await?;
        Ok(())
    }

    pub async fn update_state(
        &mut self,
        wallet: &Arc<dyn BaseWallet>,
        anoncreds: &Arc<dyn BaseAnonCreds>,
        agency_client: &AgencyClient,
        connection: &MediatedConnection,
    ) -> VcxResult<IssuerState> {
        trace!("Issuer::update_state >>>");
        if self.is_terminal_state() {
            return Ok(self.get_state());
        }
        let send_message = connection.send_message_closure(Arc::clone(wallet)).await?;
        let messages = connection.get_messages(agency_client).await?;
        if let Some((uid, msg)) = self.find_message_to_handle(messages) {
            self.step(anoncreds, msg.into(), Some(send_message)).await?;
            connection.update_message_status(&uid, agency_client).await?;
        }
        Ok(self.get_state())
    }
}

pub mod test_utils {
    use agency_client::agency_client::AgencyClient;
    use messages::msg_fields::protocols::cred_issuance::propose_credential::ProposeCredential;
    use messages::msg_fields::protocols::cred_issuance::CredentialIssuance;
    use messages::AriesMessage;

    use crate::errors::error::prelude::*;
    use crate::handlers::connection::mediated_connection::MediatedConnection;

    pub async fn get_credential_proposal_messages(
        agency_client: &AgencyClient,
        connection: &MediatedConnection,
    ) -> VcxResult<Vec<(String, ProposeCredential)>> {
        let credential_proposals: Vec<(String, ProposeCredential)> = connection
            .get_messages(agency_client)
            .await?
            .into_iter()
            .filter_map(|(uid, message)| match message {
                AriesMessage::CredentialIssuance(CredentialIssuance::ProposeCredential(proposal)) => {
                    Some((uid, proposal))
                }
                _ => None,
            })
            .collect();

        Ok(credential_proposals)
    }
}

// #[cfg(test)]
// pub mod unit_tests {
//     use messages::msg_fields::protocols::cred_issuance::CredentialIssuance;

//     use crate::common::test_utils::mock_profile;
//     use crate::protocols::issuance::issuer::state_machine::unit_tests::_send_message;
//     use crate::utils::devsetup::SetupMocks;

//     use super::*;

//     fn _cred_data() -> String {
//         json!({"name": "alice"}).to_string()
//     }

//     fn _issuer() -> Issuer {
//         Issuer::create("test_source_id").unwrap()
//     }

//     fn _issuer_revokable_from_proposal() -> Issuer {
//         Issuer::create_from_proposal("test_source_id", &_credential_proposal()).unwrap()
//     }

//     fn _send_message_but_fail() -> Option<SendClosure> {
//         Some(Box::new(|_: AriesMessage| {
//             Box::pin(async { Err(AriesVcxError::from_msg(AriesVcxErrorKind::IOError, "Mocked error")) })
//         }))
//     }

//     impl Issuer {
//         async fn to_offer_sent_state_unrevokable(mut self) -> Issuer {
//             self.build_credential_offer_msg(&mock_profile(), _offer_info_unrevokable(), None)
//                 .await
//                 .unwrap();
//             self.mark_credential_offer_msg_sent().unwrap();
//             self
//         }

//         async fn to_request_received_state(mut self) -> Issuer {
//             self = self.to_offer_sent_state_unrevokable().await;
//             self.step(
//                 &mock_profile(),
//                 CredentialIssuanceAction::CredentialRequest(_credential_request()),
//                 _send_message(),
//             )
//             .await
//             .unwrap();
//             self
//         }

//         async fn to_finished_state_unrevokable(mut self) -> Issuer {
//             self = self.to_request_received_state().await;
//             self.step(
//                 &mock_profile(),
//                 CredentialIssuanceAction::CredentialSend(),
//                 _send_message(),
//             )
//             .await
//             .unwrap();
//             self.step(
//                 &mock_profile(),
//                 CredentialIssuanceAction::CredentialAck(_ack()),
//                 _send_message(),
//             )
//             .await
//             .unwrap();
//             self
//         }
//     }

//     #[tokio::test]
//     async fn test_build_credential_preview() {
//         fn verify_preview(preview: CredentialPreview) {
//             let value_name = preview
//                 .attributes
//                 .clone()
//                 .into_iter()
//                 .find(|x| x.name == "name")
//                 .unwrap();
//             let value_age = preview
//                 .attributes
//                 .clone()
//                 .into_iter()
//                 .find(|x| x.name == "age")
//                 .unwrap();
//             assert_eq!(value_name.name, "name");
//             assert_eq!(value_name.value, "Alice");
//             assert_eq!(value_age.name, "age");
//             assert_eq!(value_age.value, "123");
//         }

//         let _setup = SetupMocks::init();
//         let input = json!({"name":"Alice","age":"123"}).to_string();
//         let preview = _build_credential_preview(&input).unwrap();
//         verify_preview(preview);

//         let input = json!([
//             {"name":"name", "value": "Alice"},
//             {"name": "age", "value": "123"}
//         ])
//         .to_string();
//         let preview = _build_credential_preview(&input).unwrap();
//         verify_preview(preview);
//     }

//     #[tokio::test]
//     async fn test_cant_revoke_without_revocation_details() {
//         let _setup = SetupMocks::init();
//         let issuer = _issuer().to_finished_state_unrevokable().await;
//         assert_eq!(IssuerState::Finished, issuer.get_state());
//         let revoc_result = issuer.revoke_credential_local(&mock_profile()).await;
//         assert_eq!(revoc_result.unwrap_err().kind(), AriesVcxErrorKind::InvalidState)
//     }

//     #[tokio::test]
//     async fn test_credential_can_be_resent_after_failure() {
//         let _setup = SetupMocks::init();
//         let mut issuer = _issuer().to_request_received_state().await;
//         assert_eq!(IssuerState::RequestReceived, issuer.get_state());

//         let send_result = issuer
//             .send_credential(&mock_profile(), _send_message_but_fail().unwrap())
//             .await;
//         assert_eq!(send_result.is_err(), true);
//         assert_eq!(IssuerState::RequestReceived, issuer.get_state());

//         let send_result = issuer.send_credential(&mock_profile(), _send_message().unwrap()).await;
//         assert_eq!(send_result.is_err(), false);
//         assert_eq!(IssuerState::CredentialSent, issuer.get_state());
//     }

//     #[tokio::test]
//     async fn exchange_credential_from_proposal_without_negotiation() {
//         let _setup = SetupMocks::init();
//         let mut issuer = _issuer_revokable_from_proposal();
//         assert_eq!(IssuerState::ProposalReceived, issuer.get_state());

//         issuer
//             .build_credential_offer_msg(&mock_profile(), _offer_info(), Some("comment".into()))
//             .await
//             .unwrap();
//         issuer.send_credential_offer(_send_message().unwrap()).await.unwrap();
//         assert_eq!(IssuerState::OfferSent, issuer.get_state());

//         let messages = map!(
//             "key_1".to_string() => AriesMessage::CredentialIssuance(CredentialIssuance::RequestCredential(_credential_request()))
//         );
//         let (_, msg) = issuer.find_message_to_handle(messages).unwrap();
//         issuer.step(&mock_profile(), msg.into(), _send_message()).await.unwrap();
//         assert_eq!(IssuerState::RequestReceived, issuer.get_state());

//         issuer
//             .send_credential(&mock_profile(), _send_message().unwrap())
//             .await
//             .unwrap();
//         assert_eq!(IssuerState::CredentialSent, issuer.get_state());

//         let messages = map!(
//             "key_1".to_string() => AriesMessage::CredentialIssuance(CredentialIssuance::Ack(_ack()))
//         );
//         let (_, msg) = issuer.find_message_to_handle(messages).unwrap();
//         issuer.step(&mock_profile(), msg.into(), _send_message()).await.unwrap();
//         assert_eq!(IssuerState::Finished, issuer.get_state());
//     }

//     #[tokio::test]
//     async fn exchange_credential_from_proposal_with_negotiation() {
//         let _setup = SetupMocks::init();
//         let mut issuer = _issuer_revokable_from_proposal();
//         assert_eq!(IssuerState::ProposalReceived, issuer.get_state());

//         issuer
//             .build_credential_offer_msg(&mock_profile(), _offer_info(), Some("comment".into()))
//             .await
//             .unwrap();
//         issuer.send_credential_offer(_send_message().unwrap()).await.unwrap();
//         assert_eq!(IssuerState::OfferSent, issuer.get_state());

//         let messages = map!(
//             "key_1".to_string() => AriesMessage::CredentialIssuance(CredentialIssuance::ProposeCredential(_credential_proposal()))()
//         );
//         let (_, msg) = issuer.find_message_to_handle(messages).unwrap();
//         issuer.step(&mock_profile(), msg.into(), _send_message()).await.unwrap();
//         assert_eq!(IssuerState::ProposalReceived, issuer.get_state());

//         issuer
//             .build_credential_offer_msg(&mock_profile(), _offer_info(), Some("comment".into()))
//             .await
//             .unwrap();
//         issuer.send_credential_offer(_send_message().unwrap()).await.unwrap();
//         assert_eq!(IssuerState::OfferSent, issuer.get_state());

//         let messages = map!(
//             "key_1".to_string() => AriesMessage::CredentialIssuance(CredentialIssuance::RequestCredential(_credential_request()))
//         );
//         let (_, msg) = issuer.find_message_to_handle(messages).unwrap();
//         issuer.step(&mock_profile(), msg.into(), _send_message()).await.unwrap();
//         assert_eq!(IssuerState::RequestReceived, issuer.get_state());

//         issuer
//             .send_credential(&mock_profile(), _send_message().unwrap())
//             .await
//             .unwrap();
//         assert_eq!(IssuerState::CredentialSent, issuer.get_state());

//         let messages = map!(
//             "key_1".to_string() => AriesMessage::CredentialIssuance(CredentialIssuance::Ack(_ack()))
//         );
//         let (_, msg) = issuer.find_message_to_handle(messages).unwrap();
//         issuer.step(&mock_profile(), msg.into(), _send_message()).await.unwrap();
//         assert_eq!(IssuerState::Finished, issuer.get_state());
//     }

//     #[tokio::test]
//     async fn issuer_cant_send_offer_twice() {
//         let _setup = SetupMocks::init();
//         let mut issuer = _issuer().to_offer_sent_state_unrevokable().await;
//         assert_eq!(IssuerState::OfferSent, issuer.get_state());

//         let res = issuer.send_credential_offer(_send_message_but_fail().unwrap()).await;
//         assert_eq!(IssuerState::OfferSent, issuer.get_state());
//         assert!(res.is_err());
//     }
// }
