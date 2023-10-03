use std::{fmt::Debug, sync::Arc};

use aries_vcx::{
    errors::error::VcxResult,
    handlers::{out_of_band::sender::OutOfBandSender, util::AnyInvitation},
    messages::msg_fields::protocols::out_of_band::invitation::OobService,
    protocols::connection::{
        invitee::{
            states::{
                completed::Completed, initial::Initial, requested::Requested as InviteeRequested,
            },
            InviteeConnection,
        },
        inviter::{states::requested::Requested as InviterRequested, InviterConnection},
        pairwise_info::PairwiseInfo,
        trait_bounds::TheirDidDoc,
        Connection,
    },
    utils::{encryption_envelope::EncryptionEnvelope, mockdata::profile::mock_ledger::MockLedger},
};
use aries_vcx_core::{
    ledger::base_ledger::IndyLedgerRead,
    wallet::{
        base_wallet::BaseWallet,
        indy::{wallet::create_and_open_wallet, IndySdkWallet, WalletConfig},
    },
};
use async_trait::async_trait;
use diddoc_legacy::aries::service::AriesService;
use env_logger;
use messages::{
    msg_fields::protocols::{
        basic_message::{BasicMessage, BasicMessageContent, BasicMessageDecorators},
        connection::{request::Request, response::Response},
        out_of_band::invitation::Invitation,
    },
    AriesMessage,
};
use serde::Deserialize;
use tokio::sync::mpsc::{Receiver, Sender};
use url::Url;

use crate::mpsc_registry::MpscRegistry;

pub struct DemoAgent {
    channels_didcomm: MpscRegistry<EncryptionEnvelope>,
    channels_invitations: MpscRegistry<Invitation>,
    wallet: Arc<dyn BaseWallet>,
    endpoint_url: Url,
    name: String,
    log_color: String,
}

pub fn build_basic_message(text_message: String) -> BasicMessage {
    let content = BasicMessageContent::builder()
        .content(text_message)
        .sent_time(Default::default())
        .build();
    let decorator = BasicMessageDecorators::builder().build();
    BasicMessage::builder()
        .id(uuid::Uuid::new_v4().to_string())
        .content(content)
        .decorators(decorator)
        .build()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageData {
    message: String,
    recipient_verkey: String,
    sender_verkey: String,
}

pub static COLOR_YELLOW: &str = "\x1b[93m";
pub static COLOR_GREEN: &str = "\x1b[32m";

impl DemoAgent {
    pub async fn new(name: String, mediator_base_url: Url, log_color: String) -> DemoAgent {
        let endpoint_url = mediator_base_url
            .join(&format!("/send_user_message/{name}"))
            .unwrap();
        let agent_uuid = uuid::Uuid::new_v4().to_string();
        let wallet_config = WalletConfig {
            wallet_name: format!("demo_{name}_{agent_uuid}"),
            wallet_key: "8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY".into(),
            wallet_key_derivation: "RAW".into(),
            wallet_type: None,
            storage_config: None,
            storage_credentials: None,
            rekey: None,
            rekey_derivation_method: None,
        };
        let handle = create_and_open_wallet(&wallet_config).await.unwrap();
        let wallet = IndySdkWallet {
            wallet_handle: handle,
        };
        info!("Created agent {name} with mediator endpoint {endpoint_url}");
        DemoAgent {
            channels_didcomm: MpscRegistry::new(),
            channels_invitations: MpscRegistry::new(),
            name,
            wallet: Arc::new(wallet),
            endpoint_url,
            log_color,
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    // the reason why we return in Arc is because we don't want fail send if receiver has been
    // dropped somewhere That simulates reality. Just because we send didcomm message, doesn't
    // mean someone is going to be waiting for it and read it
    pub async fn register_didcomm_channel(
        &self,
        transport_id: String,
        didcomm_sender: Sender<EncryptionEnvelope>,
        didcomm_receiver: Receiver<EncryptionEnvelope>,
    ) {
        self.channels_didcomm
            .register_sender(transport_id.clone(), didcomm_sender)
            .await;
        self.channels_didcomm
            .register_receiver(transport_id, didcomm_receiver)
            .await;
    }

    pub async fn register_invitation_receiver(
        &self,
        transport_id: String,
        invitation_receiver: Receiver<Invitation>,
    ) {
        self.channels_invitations
            .register_receiver(transport_id, invitation_receiver)
            .await
    }

    pub async fn register_invitation_sender(
        &self,
        transport_id: String,
        invitation_sender: Sender<Invitation>,
    ) {
        self.channels_invitations
            .register_sender(transport_id, invitation_sender)
            .await
    }

    /*
    Waits until a new message address for the agent is received by mediator. When a message addressed
    for the respective agent is received, it attempts to decrypt the message and deserialize into
    expected type T.
     */
    pub async fn wait_for_didcomm_message<T>(&self, transport_id: &str) -> Result<T, String>
    where
        T: for<'de> Deserialize<'de>,
        T: Debug,
    {
        info!(
            "{}Agent[{}] is waiting for didcomm message",
            self.name, self.log_color
        );
        let didcomm_msg = self.channels_didcomm.receive_msg(transport_id).await;
        let unpacked_msg = self
            .wallet
            .unpack_message(&didcomm_msg.0.as_slice())
            .await
            .unwrap();
        let payload_msg: T = serde_json::from_str(&unpacked_msg.message).unwrap();
        info!(
            "{}Agent[{}] has received a message sender verkey: {:?}",
            self.name, self.log_color, &unpacked_msg.sender_verkey
        );
        Ok(payload_msg)
    }

    pub async fn wait_for_invitation_message(&self, transport_id: &str) -> Invitation {
        info!(
            "{}Agent[{}] is waiting for Invitation message",
            self.name, self.log_color
        );
        self.channels_invitations.receive_msg(transport_id).await
    }

    // Note: this would in practice be implemented as sending POST message to service endpoint
    //       specified in DidDoc for our counterparty.
    //       For demo purposes, we only pass messages in-memory via mpsc channels.
    pub async fn send_didcomm_message(&self, transport_id: &str, message: EncryptionEnvelope) {
        self.channels_didcomm.send_msg(transport_id, message).await
    }

    pub async fn send_invitation_message(&self, transport_id: &str, message: Invitation) {
        self.channels_invitations
            .send_msg(transport_id, message)
            .await
    }

    pub async fn prepare_invitation(&self) -> (Invitation, PairwiseInfo) {
        let (faber_pw_did, faber_invite_key) = self
            .wallet
            .create_and_store_my_did(None, None)
            .await
            .unwrap();
        let service = AriesService {
            id: uuid::Uuid::new_v4().to_string(),
            type_: "".to_string(),
            priority: 0,
            recipient_keys: vec![faber_invite_key.clone()],
            routing_keys: vec![],
            service_endpoint: self.endpoint_url.clone(),
        };
        let sender = OutOfBandSender::create().append_service(&OobService::AriesService(service));
        let msg_invitation = sender.as_invitation_msg();
        info!(
            "{}Agent[{}] prepared invitation message: {}",
            self.name,
            self.log_color,
            serde_json::to_string(&msg_invitation).unwrap()
        );
        let pw_info_faber = PairwiseInfo {
            pw_did: faber_pw_did,
            pw_vk: faber_invite_key,
        };
        (msg_invitation, pw_info_faber)
    }

    pub async fn process_invitation(
        &self,
        msg_oob_invitation: Invitation,
    ) -> (InviteeConnection<InviteeRequested>, EncryptionEnvelope) {
        let (pw_did, pw_vk) = self
            .wallet
            .create_and_store_my_did(None, None)
            .await
            .unwrap();
        info!(
            "{}Agent[{}] in role of invitee generated pairwise key {pw_vk}",
            self.name, self.log_color
        );
        let mock_ledger: Arc<dyn IndyLedgerRead> = Arc::new(MockLedger {}); // cause we know we want call ledger *eew...*
        let mut invitee_invited =
            InviteeConnection::<Initial>::new_invitee("foo".into(), PairwiseInfo { pw_did, pw_vk })
                .accept_invitation(&mock_ledger, AnyInvitation::Oob(msg_oob_invitation.into()))
                .await
                .unwrap();

        let invitee_requested = invitee_invited
            .prepare_request(self.endpoint_url.clone(), vec![])
            .await
            .unwrap();

        let msg_connection_request = invitee_requested.get_request().clone();
        info!(
            "{}Agent[{}] in role of invitee is sending connection-request message {}",
            self.name,
            self.log_color,
            serde_json::to_string(&msg_connection_request).unwrap()
        );
        let msg_encrypted = invitee_requested
            .encrypt_message(&self.wallet, &msg_connection_request.into())
            .await
            .unwrap();

        (invitee_requested, msg_encrypted)
    }

    pub async fn process_connection_request(
        &self,
        msg_request: Request,
        faber_invite_info: PairwiseInfo,
    ) -> (InviterConnection<InviterRequested>, EncryptionEnvelope) {
        let inviter_invited = InviterConnection::new_inviter("".to_owned(), faber_invite_info)
            .into_invited(
                &msg_request
                    .decorators
                    .thread
                    .as_ref()
                    .map(|t| t.thid.as_str())
                    .unwrap_or(msg_request.id.as_str()),
            );
        let inviter_requested = inviter_invited
            .handle_request(&self.wallet, msg_request, self.endpoint_url.clone(), vec![])
            .await
            .unwrap();
        let msg_response = inviter_requested.get_connection_response_msg();
        info!(
            "{}Agent[{}] in role of inviter processed connection-request and is sending \
             connection-response {}",
            self.name,
            self.log_color,
            serde_json::to_string(&msg_response).unwrap()
        );

        let msg_encrypted = inviter_requested
            .encrypt_message(&self.wallet, &msg_response.into())
            .await
            .unwrap();

        (inviter_requested, msg_encrypted)
    }

    pub async fn process_connection_response(
        &self,
        invitee_requested: InviteeConnection<InviteeRequested>,
        response: Response,
    ) -> InviteeConnection<Completed> {
        let invitee_complete = invitee_requested
            .handle_response(&self.wallet.clone(), response)
            .await
            .unwrap();
        info!(
            "{}Agent[{}] in role of invitee processed connection-response",
            self.name, self.log_color,
        );
        invitee_complete
    }

    pub async fn encrypt_message<I, S>(
        &self,
        connection: Connection<I, S>,
        message: AriesMessage,
    ) -> EncryptionEnvelope
    where
        S: TheirDidDoc,
    {
        info!(
            "{}Agent[{}] is going to encrypt aries message for a connectin. Message: {}",
            self.name,
            self.log_color,
            serde_json::to_string(&message).unwrap()
        );
        connection
            .encrypt_message(&self.wallet, &message)
            .await
            .unwrap()
    }
}
