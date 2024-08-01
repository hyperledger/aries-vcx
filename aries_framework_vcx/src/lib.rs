#![deny(clippy::unwrap_used)]

pub use aries_vcx;
pub use aries_vcx::aries_vcx_wallet::wallet::askar::askar_wallet_config::AskarWalletConfig;
pub use url::Url;
pub mod framework {
    use std::{
        error::Error,
        sync::{mpsc::Receiver, Arc, Mutex},
    };

    use aries_vcx::aries_vcx_wallet::wallet::{
        askar::{
            askar_wallet_config::AskarWalletConfig,
            key_method::{ArgonLevel, AskarKdfMethod, KeyMethod},
            AskarWallet,
        },
        base_wallet::ManageWallet,
    };
    use did_peer::resolver::PeerDidResolver;
    use did_resolver_registry::ResolverRegistry;
    use log::{info, warn};
    use url::Url;

    use crate::{
        connection_service::{ConnectionService, ConnectionServiceConfig},
        invitation_service::InvitationService,
    };

    pub static IN_MEMORY_DB_URL: &str = "sqlite://:memory:";
    pub static DEFAULT_WALLET_PROFILE: &str = "aries_framework_vcx_default";
    pub static DEFAULT_ASKAR_KEY_METHOD: KeyMethod = KeyMethod::DeriveKey {
        inner: AskarKdfMethod::Argon2i {
            inner: (ArgonLevel::Interactive),
        },
    };

    #[derive(Clone)]
    pub struct FrameworkConfig {
        pub wallet_config: AskarWalletConfig,
        pub connection_service_config: ConnectionServiceConfig,
        pub agent_endpoint: Url,
        pub agent_label: String,
    }

    pub struct AriesFrameworkVCX {
        pub framework_config: FrameworkConfig,
        pub wallet: Arc<AskarWallet>,
        pub did_resolver_registry: Arc<ResolverRegistry>,
        pub invitation_service: Arc<Mutex<InvitationService>>,

        /// A service for the management of any and all things related to connections, including the usage of invitations (Out Of Band Invitations), the DID Exchange protocol, and mediation protocols.
        ///
        /// Note: This is service is about generic DIDComm connections and is **NOT** to be confused with the specific Aries handshake connection protocol RFC 0160 - https://github.com/hyperledger/aries-rfcs/tree/main/features/0160-connection-protocol
        pub connection_service: Arc<Mutex<ConnectionService>>,
    }

    impl AriesFrameworkVCX {
        pub async fn initialize(framework_config: FrameworkConfig) -> Result<Self, Box<dyn Error>> {
            info!("Initializing Aries Framework VCX");

            // Warn if the wallet pass key being used is the sample key from the documentation
            if framework_config.wallet_config.pass_key() == "sample_pass_key" {
                warn!("The Default Wallet Pass Key SHOULD NOT be used in production");
            }

            // Wallet Initialization
            let wallet = Arc::new(framework_config.wallet_config.create_wallet().await?);

            // DID Resolver
            // TODO - DID Sov Resolver
            let did_peer_resolver = PeerDidResolver::new();
            let did_resolver_registry = Arc::new(
                ResolverRegistry::new().register_resolver("peer".into(), did_peer_resolver),
            );

            // Service Initializations
            let invitation_service = Arc::new(Mutex::new(InvitationService::new(
                framework_config.clone(),
                wallet.clone(),
            )));
            let connection_service = Arc::new(Mutex::new(ConnectionService::new(
                framework_config.clone(),
                wallet.clone(),
                did_resolver_registry.clone(),
                invitation_service.clone(),
            )));

            Ok(Self {
                framework_config,
                wallet,
                did_resolver_registry,
                invitation_service,
                connection_service,
            })
        }
    }

    // TODO - Consider adding a way to register event emitters with restrictions on the type of events to listen to for a given emitter -- such as, only receive events for did-exchange response messages (rather than having to filter all events)
    pub trait EventEmitter {
        type Event;
        fn emit_event(&mut self, event: Self::Event);
        fn register_event_receiver(&mut self) -> Receiver<Self::Event>;
    }
}

pub mod connection_service {
    use std::{
        error::Error,
        sync::{
            mpsc::{self, Receiver, Sender},
            Arc, Mutex,
        },
    };

    use aries_vcx::{
        aries_vcx_wallet::wallet::askar::AskarWallet,
        messages::msg_fields::protocols::out_of_band::invitation::Invitation,
        protocols::did_exchange::state_machine::{
            generic::GenericDidExchange,
            helpers::create_peer_did_4,
            requester::helpers::{
                invitation_get_acceptable_did_exchange_version, invitation_get_first_did_service,
            },
        },
    };
    use did_resolver_registry::ResolverRegistry;
    use log::{debug, info, trace};
    use uuid::Uuid;

    use crate::{
        framework::{EventEmitter, FrameworkConfig},
        invitation_service::InvitationService,
    };

    #[derive(Clone)]
    pub struct ConnectionServiceConfig {
        pub auto_complete_requests: bool,
        pub auto_respond_to_requests: bool,
        pub auto_handle_requests: bool,
    }

    impl Default for ConnectionServiceConfig {
        fn default() -> Self {
            Self {
                auto_complete_requests: true,
                auto_handle_requests: true,
                auto_respond_to_requests: true,
            }
        }
    }

    pub struct ConnectionService {
        framework_config: FrameworkConfig,
        event_senders: Vec<Sender<ConnectionEvent>>,
        wallet: Arc<AskarWallet>,
        did_resolver_registry: Arc<ResolverRegistry>,
        invitation_service: Arc<Mutex<InvitationService>>,
    }

    impl ConnectionService {
        pub fn new(
            framework_config: FrameworkConfig,
            wallet: Arc<AskarWallet>,
            did_resolver_registry: Arc<ResolverRegistry>,
            invitation_service: Arc<Mutex<InvitationService>>,
        ) -> Self {
            invitation_service
                .lock()
                .expect("unpoisoned mutex")
                .register_event_receiver();
            Self {
                framework_config,
                event_senders: vec![],
                wallet,
                did_resolver_registry,
                invitation_service,
            }
        }

        /// Helper function to request connection, automating everything until connection completed
        pub async fn connect(&mut self) {}

        /// Helper function to request connection and block until complete but with timeout, automating everything until connection completed
        pub async fn connect_and_await() {
            // TODO - add observer
        }

        /// Handles inbound connection requests in relation to a invitation the framework has created. It will automate the process until the connection is completed, barring any errors throughout the process.
        pub async fn handle_request() {}

        /// Handles inbound connection requests in relation to a invitation the framework has created. It will automate the process until the connection is completed, barring any errors throughout the process. This method will not return until completion, error, or the timeout has been reached. Use [`handle_request()`] instead for non blocking behavior.
        ///
        /// [`handle_request()`]: Self::handle_request()
        pub async fn handle_request_and_await(
            &mut self,
            invitation_id: &str,
        ) -> Result<(), Box<dyn Error>> {
            // testing I was doing here, ignore please
            // let invitation = self
            //     .invitation_service
            //     .lock()
            //     .expect("unpoisoned mutex")
            //     .create_invitation()
            //     .await?;
            // self.request_connection(invitation).await?;
            // TODO - add observer
            Ok(())
        }
    }

    // Provides internal framework functions for transitioning between protocol states
    impl ConnectionService {
        async fn request_connection(
            &mut self,
            invitation: Invitation,
        ) -> Result<(), Box<dyn Error>> {
            debug!(
                "Requesting Connection via DID Exchange with invitation {}",
                invitation
            );

            // Create our peer DID using Peer DID Numalgo 4
            // TODO - peer did we create here should be able to be mediated (routing keys should be provided or generated)
            // TODO - create_peer_did_4() should move into peer did 4 implementation
            let (peer_did, _our_verkey) = create_peer_did_4(
                self.wallet.as_ref(),
                self.framework_config.agent_endpoint.clone(),
                vec![],
            )
            .await?;

            // Get Inviter/Responder DID from invitation
            let inviter_did = invitation_get_first_did_service(&invitation)?;

            // Get DID Exchange version to use based off of invitation handshake protocols
            let version = invitation_get_acceptable_did_exchange_version(&invitation)?;

            // TODO - Fix DID Exchange Goal Code definition - Should not be "To establish a connection" - rather should be a goal code or not specified (IIRC)
            // Create DID Exchange Request Message, generate did_exchange_requester for future
            let (state_machine, request) = GenericDidExchange::construct_request(
                &self.did_resolver_registry,
                Some(invitation.id.clone()),
                &inviter_did,
                &peer_did,
                self.framework_config.agent_label.to_owned(),
                version,
            )
            .await?;

            trace!("Created DID Exchange State Machine and request message, going to send message");

            // Send Request
            // TODO - Send Request Message

            // Store Updated State
            let record = ConnectionRecord {
                id: Uuid::parse_str(&request.inner().id)?,
                invitation_id: Uuid::parse_str(&invitation.id)?,
                state_machine,
            };
            // TODO - Store Record

            // Emit new event indicating updated state
            self.emit_event(ConnectionEvent { record });

            Ok(())
        }

        fn process_response() {}

        fn send_complete() {}

        fn process_request() {}

        fn send_response() {}

        fn process_complete() {}
    }

    impl EventEmitter for ConnectionService {
        type Event = ConnectionEvent;

        fn emit_event(&mut self, event: ConnectionEvent) {
            info!("Emitting ConnectionEvent: {:?}", &event);
            self.event_senders
                .retain(|tx| match tx.send(event.clone()) {
                    Ok(_) => true,
                    Err(_) => {
                        debug!("Removing deallocated event listener from event listeners list");
                        false
                    }
                })
        }

        fn register_event_receiver(&mut self) -> Receiver<ConnectionEvent> {
            let (tx, rx): (Sender<ConnectionEvent>, Receiver<ConnectionEvent>) = mpsc::channel();

            self.event_senders.push(tx);
            rx
        }
    }

    #[derive(Debug, Clone)]
    pub struct ConnectionRecord {
        id: Uuid,
        invitation_id: Uuid,
        state_machine: GenericDidExchange,
    }

    #[derive(Debug, Clone)]
    pub struct ConnectionEvent {
        record: ConnectionRecord,
    }
}

pub mod invitation_service {
    use std::{
        error::Error,
        sync::{
            mpsc::{self, Receiver, Sender},
            Arc,
        },
    };

    use aries_vcx::{
        aries_vcx_wallet::wallet::askar::AskarWallet,
        handlers::out_of_band::sender::OutOfBandSender,
        messages::{
            msg_fields::protocols::out_of_band::invitation::{Invitation, OobService},
            msg_types::{
                protocols::did_exchange::{DidExchangeType, DidExchangeTypeV1},
                Protocol,
            },
        },
        protocols::did_exchange::state_machine::helpers::create_peer_did_4,
    };
    use log::{debug, info};

    use crate::framework::{EventEmitter, FrameworkConfig};

    pub struct InvitationService {
        framework_config: FrameworkConfig,
        event_senders: Vec<Sender<InvitationEvent>>,
        wallet: Arc<AskarWallet>,
    }

    #[derive(Debug, Clone)]
    pub struct InvitationEvent {
        pub state: String,
    }

    impl EventEmitter for InvitationService {
        type Event = InvitationEvent;
        fn emit_event(&mut self, event: InvitationEvent) {
            info!("Emitting InvitationEvent: {:?}", &event);
            self.event_senders
                .retain(|tx| match tx.send(event.clone()) {
                    Ok(_) => true,
                    Err(_) => {
                        debug!("Removing deallocated event listener from event listeners list");
                        false
                    }
                })
        }

        fn register_event_receiver(&mut self) -> Receiver<Self::Event> {
            let (tx, rx): (Sender<InvitationEvent>, Receiver<InvitationEvent>) = mpsc::channel();

            self.event_senders.push(tx);
            rx
        }
    }

    impl InvitationService {
        pub fn new(framework_config: FrameworkConfig, wallet: Arc<AskarWallet>) -> Self {
            Self {
                framework_config,
                event_senders: vec![],
                wallet,
            }
        }

        /// Creates an Out of Band Invitation
        pub async fn create_invitation(&mut self) -> Result<Invitation, Box<dyn Error>> {
            debug!("Creating Out Of Band Invitation");
            // TODO - invitation should be able to be mediated (routing keys should be provided or generated)
            // TODO - create_peer_did_4() should move into peer did 4 implementation
            let (peer_did, _our_verkey) = create_peer_did_4(
                self.wallet.as_ref(),
                self.framework_config.agent_endpoint.clone(),
                vec![],
            )
            .await?;

            let service = OobService::Did(peer_did.to_string());

            let sender = OutOfBandSender::create()
                .append_service(&service)
                .append_handshake_protocol(Protocol::DidExchangeType(DidExchangeType::V1(
                    DidExchangeTypeV1::new_v1_1(),
                )))?;

            let invitation = sender.oob;
            info!("Created Out of Band Invitation {:?}", invitation);

            // TODO - persist
            self.emit_event(InvitationEvent {
                state: "created".to_owned(),
            });
            Ok(invitation)
        }

        pub async fn get_invitation(&self) {}
    }
}
