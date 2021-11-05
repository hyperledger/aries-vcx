use serde::{de, Deserialize, Deserializer, ser, Serialize, Serializer};
use serde_json::Value;

use crate::messages::connection::invite::{PublicInvitation, PairwiseInvitation};
use log;

use crate::messages::ack::Ack;
use crate::messages::basic_message::message::BasicMessage;
use crate::messages::connection::problem_report::ProblemReport as ConnectionProblemReport;
use crate::messages::connection::request::Request;
use crate::messages::connection::response::SignedResponse;
use crate::messages::discovery::disclose::Disclose;
use crate::messages::discovery::query::Query;
use crate::messages::error::ProblemReport as CommonProblemReport;
use crate::messages::forward::Forward;
use crate::messages::issuance::credential::Credential;
use crate::messages::issuance::credential_offer::CredentialOffer;
use crate::messages::issuance::credential_proposal::CredentialProposal;
use crate::messages::issuance::credential_request::CredentialRequest;
use crate::messages::proof_presentation::presentation::Presentation;
use crate::messages::proof_presentation::presentation_proposal::PresentationProposal;
use crate::messages::proof_presentation::presentation_request::PresentationRequest;
use crate::messages::trust_ping::ping::Ping;
use crate::messages::trust_ping::ping_response::PingResponse;
use crate::handlers::out_of_band::OutOfBand;

use self::message_family::MessageFamilies;
use self::message_type::MessageType;

pub mod message_family;
pub mod message_type;
pub mod protocol_registry;

#[derive(Debug, PartialEq, Clone)]
pub enum A2AMessage {
    /// routing
    Forward(Forward),

    /// DID Exchange
    ConnectionInvitationPairwise(PairwiseInvitation),
    ConnectionInvitationPublic(PublicInvitation),
    ConnectionRequest(Request),
    ConnectionResponse(SignedResponse),
    ConnectionProblemReport(ConnectionProblemReport),

    /// trust ping
    Ping(Ping),
    PingResponse(PingResponse),

    /// notification
    Ack(Ack),
    CommonProblemReport(CommonProblemReport),

    /// credential issuance
    CredentialProposal(CredentialProposal),
    CredentialOffer(CredentialOffer),
    CredentialRequest(CredentialRequest),
    Credential(Credential),
    CredentialAck(Ack),

    /// proof presentation
    PresentationProposal(PresentationProposal),
    PresentationRequest(PresentationRequest),
    Presentation(Presentation),
    PresentationAck(Ack),

    /// discovery features
    Query(Query),
    Disclose(Disclose),

    /// basic message
    BasicMessage(BasicMessage),

    /// out of band
    OutOfBand(OutOfBand),

    /// Any Raw Message
    Generic(Value),
}

impl A2AMessage {
    pub fn thread_id_matches(&self, thread_id: &str) -> bool {
        match self {
            Self::Presentation(presentation) => presentation.from_thread(thread_id),
            // Self::PingResponse(ping_response) => ping_response.from_thread(thread_id),
            Self::ConnectionProblemReport(connection_problem_report) => connection_problem_report.from_thread(thread_id),
            Self::ConnectionRequest(request) => request.from_thread(thread_id),
            Self::CommonProblemReport(common_problem_report) => common_problem_report.from_thread(thread_id),
            Self::CredentialOffer(credential_offer) => credential_offer.from_thread(thread_id),
            Self::CredentialProposal(credential_proposal) => credential_proposal.from_thread(thread_id),
            Self::Credential(credential) => credential.from_thread(thread_id),
            Self::PresentationProposal(presentation_proposal) => presentation_proposal.from_thread(thread_id),
            Self::PresentationAck(ack) | Self::CredentialAck(ack) | Self::Ack(ack) => ack.from_thread(thread_id),
            _ => true
        }
    }
}

impl<'de> Deserialize<'de> for A2AMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;

        if log::log_enabled!(log::Level::Trace) {
            let message_json = serde_json::ser::to_string(&value);
            let message_type_json = serde_json::ser::to_string(&value["@type"].clone());

            trace!("Deserializing v3::A2AMessage in V3 json: {:?}", &message_json);
            trace!("Found v3::A2AMessage message type {:?}", &message_type_json);
        };

        let message_type: MessageType = match serde_json::from_value(value["@type"].clone()) {
            Ok(message_type) => message_type,
            Err(_) => return Ok(A2AMessage::Generic(value))
        };

        match (message_type.family, message_type.msg_type.as_str()) {
            (MessageFamilies::Routing, A2AMessage::FORWARD) => {
                Forward::deserialize(value)
                    .map(|msg| A2AMessage::Forward(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Connections, A2AMessage::CONNECTION_INVITATION) => {
                PairwiseInvitation::deserialize(value.clone())
                    .map_or(PublicInvitation::deserialize(value)
                            .map(|msg| A2AMessage::ConnectionInvitationPublic(msg)), |msg| Ok(A2AMessage::ConnectionInvitationPairwise(msg)))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Connections, A2AMessage::CONNECTION_REQUEST) => {
                Request::deserialize(value)
                    .map(|msg| A2AMessage::ConnectionRequest(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Connections, A2AMessage::CONNECTION_RESPONSE) => {
                SignedResponse::deserialize(value)
                    .map(|msg| A2AMessage::ConnectionResponse(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::TrustPing, A2AMessage::PING) => {
                Ping::deserialize(value)
                    .map(|msg| A2AMessage::Ping(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::TrustPing, A2AMessage::PING_RESPONSE) => {
                PingResponse::deserialize(value)
                    .map(|msg| A2AMessage::PingResponse(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Connections, A2AMessage::CONNECTION_PROBLEM_REPORT) => {
                ConnectionProblemReport::deserialize(value)
                    .map(|msg| A2AMessage::ConnectionProblemReport(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Notification, A2AMessage::ACK) => {
                Ack::deserialize(value)
                    .map(|msg| A2AMessage::Ack(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::ReportProblem, A2AMessage::PROBLEM_REPORT) => {
                CommonProblemReport::deserialize(value)
                    .map(|msg| A2AMessage::CommonProblemReport(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::CredentialIssuance, A2AMessage::CREDENTIAL) => {
                Credential::deserialize(value)
                    .map(|msg| A2AMessage::Credential(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::CredentialIssuance, A2AMessage::PROPOSE_CREDENTIAL) => {
                CredentialProposal::deserialize(value)
                    .map(|msg| A2AMessage::CredentialProposal(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::CredentialIssuance, A2AMessage::CREDENTIAL_OFFER) => {
                CredentialOffer::deserialize(value)
                    .map(|msg| A2AMessage::CredentialOffer(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::CredentialIssuance, A2AMessage::REQUEST_CREDENTIAL) => {
                CredentialRequest::deserialize(value)
                    .map(|msg| A2AMessage::CredentialRequest(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::CredentialIssuance, A2AMessage::ACK) => {
                Ack::deserialize(value)
                    .map(|msg| A2AMessage::CredentialAck(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::PresentProof, A2AMessage::PROPOSE_PRESENTATION) => {
                PresentationProposal::deserialize(value)
                    .map(|msg| A2AMessage::PresentationProposal(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::PresentProof, A2AMessage::REQUEST_PRESENTATION) => {
                PresentationRequest::deserialize(value)
                    .map(|msg| A2AMessage::PresentationRequest(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::PresentProof, A2AMessage::PRESENTATION) => {
                Presentation::deserialize(value)
                    .map(|msg| A2AMessage::Presentation(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::PresentProof, A2AMessage::ACK) => {
                Ack::deserialize(value)
                    .map(|msg| A2AMessage::PresentationAck(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::DiscoveryFeatures, A2AMessage::QUERY) => {
                Query::deserialize(value)
                    .map(|msg| A2AMessage::Query(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::DiscoveryFeatures, A2AMessage::DISCLOSE) => {
                Disclose::deserialize(value)
                    .map(|msg| A2AMessage::Disclose(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Basicmessage, A2AMessage::BASIC_MESSAGE) => {
                BasicMessage::deserialize(value)
                    .map(|msg| A2AMessage::BasicMessage(msg))
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::OutOfBand, A2AMessage::OUT_OF_BAND) => {
                OutOfBand::deserialize(value)
                    .map(|msg| A2AMessage::OutOfBand(msg))
                    .map_err(de::Error::custom)
            }
            (_, other_type) => {
                warn!("Unexpected @type field structure: {}", other_type);
                Ok(A2AMessage::Generic(value))
            }
        }
    }
}

fn set_a2a_message_type<T>(msg: T, family: MessageFamilies, name: &str) -> Result<serde_json::Value, serde_json::Error> where T: Serialize {
    let mut value = ::serde_json::to_value(msg)?;
    let type_ = ::serde_json::to_value(MessageType::build(family, name))?;
    value.as_object_mut().unwrap().insert("@type".into(), type_);
    Ok(value)
}

impl Serialize for A2AMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let value = match self {
            A2AMessage::Forward(msg) => set_a2a_message_type(msg, MessageFamilies::Routing, A2AMessage::FORWARD),
            A2AMessage::ConnectionInvitationPairwise(msg) => set_a2a_message_type(msg, MessageFamilies::Connections, A2AMessage::CONNECTION_INVITATION),
            A2AMessage::ConnectionInvitationPublic(msg) => set_a2a_message_type(msg, MessageFamilies::Connections, A2AMessage::CONNECTION_INVITATION),
            A2AMessage::ConnectionRequest(msg) => set_a2a_message_type(msg, MessageFamilies::Connections, A2AMessage::CONNECTION_REQUEST),
            A2AMessage::ConnectionResponse(msg) => set_a2a_message_type(msg, MessageFamilies::Connections, A2AMessage::CONNECTION_RESPONSE),
            A2AMessage::ConnectionProblemReport(msg) => set_a2a_message_type(msg, MessageFamilies::Connections, A2AMessage::CONNECTION_PROBLEM_REPORT),
            A2AMessage::Ping(msg) => set_a2a_message_type(msg, MessageFamilies::TrustPing, A2AMessage::PING),
            A2AMessage::PingResponse(msg) => set_a2a_message_type(msg, MessageFamilies::TrustPing, A2AMessage::PING_RESPONSE),
            A2AMessage::Ack(msg) => set_a2a_message_type(msg, MessageFamilies::Notification, A2AMessage::ACK),
            A2AMessage::CommonProblemReport(msg) => set_a2a_message_type(msg, MessageFamilies::ReportProblem, A2AMessage::PROBLEM_REPORT),
            A2AMessage::CredentialOffer(msg) => set_a2a_message_type(msg, MessageFamilies::CredentialIssuance, A2AMessage::CREDENTIAL_OFFER),
            A2AMessage::Credential(msg) => set_a2a_message_type(msg, MessageFamilies::CredentialIssuance, A2AMessage::CREDENTIAL),
            A2AMessage::CredentialProposal(msg) => set_a2a_message_type(msg, MessageFamilies::CredentialIssuance, A2AMessage::PROPOSE_CREDENTIAL),
            A2AMessage::CredentialRequest(msg) => set_a2a_message_type(msg, MessageFamilies::CredentialIssuance, A2AMessage::REQUEST_CREDENTIAL),
            A2AMessage::CredentialAck(msg) => set_a2a_message_type(msg, MessageFamilies::CredentialIssuance, A2AMessage::ACK),
            A2AMessage::PresentationProposal(msg) => set_a2a_message_type(msg, MessageFamilies::PresentProof, A2AMessage::PROPOSE_PRESENTATION),
            A2AMessage::PresentationRequest(msg) => set_a2a_message_type(msg, MessageFamilies::PresentProof, A2AMessage::REQUEST_PRESENTATION),
            A2AMessage::Presentation(msg) => set_a2a_message_type(msg, MessageFamilies::PresentProof, A2AMessage::PRESENTATION),
            A2AMessage::PresentationAck(msg) => set_a2a_message_type(msg, MessageFamilies::PresentProof, A2AMessage::ACK),
            A2AMessage::Query(msg) => set_a2a_message_type(msg, MessageFamilies::DiscoveryFeatures, A2AMessage::QUERY),
            A2AMessage::Disclose(msg) => set_a2a_message_type(msg, MessageFamilies::DiscoveryFeatures, A2AMessage::DISCLOSE),
            A2AMessage::BasicMessage(msg) => set_a2a_message_type(msg, MessageFamilies::Basicmessage, A2AMessage::BASIC_MESSAGE),
            A2AMessage::OutOfBand(msg) => set_a2a_message_type(msg, MessageFamilies::OutOfBand, A2AMessage::OUT_OF_BAND),
            A2AMessage::Generic(msg) => Ok(msg.clone())
        }.map_err(ser::Error::custom)?;

        value.serialize(serializer)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MessageId(pub String);

impl MessageId {
    pub fn id() -> MessageId {
        MessageId(String::from("testid"))
    }

    pub fn new() -> MessageId {
        MessageId::default()
    }
}

impl Default for MessageId {
    #[cfg(test)]
    fn default() -> MessageId {
        MessageId::id()
    }

    #[cfg(not(test))]
    fn default() -> MessageId {
        use crate::utils::uuid;
        MessageId(uuid::uuid())
    }
}

impl A2AMessage {
    const FORWARD: &'static str = "forward";
    const CONNECTION_INVITATION: &'static str = "invitation";
    const CONNECTION_REQUEST: &'static str = "request";
    const CONNECTION_RESPONSE: &'static str = "response";
    const CONNECTION_PROBLEM_REPORT: &'static str = "problem_report";
    const PING: &'static str = "ping";
    const PING_RESPONSE: &'static str = "ping_response";
    const ACK: &'static str = "ack";
    const PROBLEM_REPORT: &'static str = "problem-report";
    const CREDENTIAL_OFFER: &'static str = "offer-credential";
    const CREDENTIAL: &'static str = "issue-credential";
    const PROPOSE_CREDENTIAL: &'static str = "propose-credential";
    const REQUEST_CREDENTIAL: &'static str = "request-credential";
    const PROPOSE_PRESENTATION: &'static str = "propose-presentation";
    const REQUEST_PRESENTATION: &'static str = "request-presentation";
    const PRESENTATION: &'static str = "presentation";
    const QUERY: &'static str = "query";
    const DISCLOSE: &'static str = "disclose";
    const BASIC_MESSAGE: &'static str = "message";
    const OUT_OF_BAND: &'static str = "out-of-band";
}

#[cfg(test)]
pub mod test_a2a_serialization {
    use serde_json::Value;

    use crate::messages::a2a::{A2AMessage, MessageId};
    use crate::messages::ack::{Ack, AckStatus};
    use crate::messages::connection::request::Request;
    use crate::utils::devsetup::SetupDefaults;
    use crate::messages::forward::Forward;

    #[test]
    #[cfg(feature = "general_test")]
    fn test_serialization_deserialization_connection_request() {
        let _setup = SetupDefaults::init();
        let a2a_msg = A2AMessage::ConnectionRequest(Request {
            id: Default::default(),
            label: "foobar".to_string(),
            connection: Default::default(),
            thread: None
        });
        let serialized = serde_json::to_string(&a2a_msg).unwrap();

        // serialization
        let val: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(val["@id"], Value::String("testid".into()));
        assert_eq!(val["@type"], Value::String("https://didcomm.org/connections/1.0/request".into()));
        assert_eq!(val["label"], Value::String("foobar".into()));
        assert_eq!(val["connection"]["DID"], Value::String("".into()));
        assert!(val["connection"]["DIDDoc"].is_object());

        // deserialization back
        let a2a_msg: A2AMessage = serde_json::from_str(&serialized).unwrap();
        if let A2AMessage::ConnectionRequest(request) = &a2a_msg {
            assert_eq!(request.id, MessageId("testid".into()));
            assert_eq!(request.label, "foobar");
        } else {
            panic!("The message was expected to be deserialized as ConnectionRequest, but was not. Deserialized: {:?} ", a2a_msg)
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_serialize_deserialize_connection_ack() {
        let _setup = SetupDefaults::init();
        let a2a_msg = A2AMessage::Ack(Ack::create().set_status(AckStatus::Ok).set_thread_id("threadid"));
        let serialized = serde_json::to_string(&a2a_msg).unwrap();

        // serialization
        let val: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(val["@id"], Value::String("testid".into()));
        assert_eq!(val["@type"], Value::String("https://didcomm.org/notification/1.0/ack".into()));
        assert_eq!(val["status"], Value::String("OK".into()));
        assert!(val["~thread"].is_object());

        // deserialization back
        let a2a_msg: A2AMessage = serde_json::from_str(&serialized).unwrap();
        if let A2AMessage::Ack(ack) = &a2a_msg {
            assert_eq!(ack.id, MessageId("testid".into()));
            assert_eq!(ack.thread.sender_order, 0);
        } else {
            panic!("The message was expected to be deserialized as ConnectionRequest, but was not. Deserialized: {:?} ", a2a_msg)
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    // todo: Add support for aries @type-ed messages on vcxagency-node, then we can stop giving fwd messages special treatment, delete this test
    fn test_serialize_forward_message_to_legacy_format() {
        let _setup = SetupDefaults::init();
        let a2a_msg = A2AMessage::Forward(Forward::new("BzCbsNYhMrjHiqZDTUASHg".into(),  "{}".as_bytes().to_vec()).unwrap());
        let serialized = serde_json::to_string(&a2a_msg).unwrap();

        // serialization
        let val: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(val["@type"], Value::String("did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/routing/1.0/forward".into()));
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_deserialize_connection_ack_legacy() {
        let _setup = SetupDefaults::init();
        let msg =
            r#"{
            "@id": "testid",
            "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/notification/1.0/ack",
            "status": "OK",
            "~thread": {
                "received_orders": {},
                "sender_order": 0
            }
        }"#;
        let a2a_msg: A2AMessage = serde_json::from_str(msg).unwrap();
        if let A2AMessage::Ack(ack) = &a2a_msg {
            assert_eq!(ack.id, MessageId("testid".into()));
            assert_eq!(ack.thread.sender_order, 0);
        } else {
            panic!("The message was expected to be deserialized as Ack, but was not.")
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    fn test_deserialization_connection_request_legacy() {
        let _setup = SetupDefaults::init();
        let msg =
            r#"{
            "@id": "testid",
            "@type": "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/connections/1.0/request",
            "connection": {
                "DID": "",
                "DIDDoc": {
                    "@context": "https://w3id.org/did/v1",
                    "authentication": [],
                    "id": "",
                    "publicKey": [],
                    "service": [
                        {
                            "id": "did:example:123456789abcdefghi;indy",
                            "priority": 0,
                            "recipientKeys": [],
                            "routingKeys": [],
                            "serviceEndpoint": "",
                            "type": "IndyAgent"
                        }
                    ]
                }
            },
            "label": "foofoo"
        }"#;
        let a2a_msg: A2AMessage = serde_json::from_str(msg).unwrap();
        if let A2AMessage::ConnectionRequest(request) = &a2a_msg {
            assert_eq!(request.id, MessageId("testid".into()));
            assert_eq!(request.label, "foofoo");
        } else {
            panic!("The message was expected to be deserialized as Connection Request, but was not.")
        }
    }
}

#[macro_export]
macro_rules! a2a_message {
    ($type:ident) => (
        impl $type {
            pub fn to_a2a_message(&self) -> A2AMessage {
                A2AMessage::$type(self.clone()) // TODO: THINK how to avoid clone
            }
        }
    );

    ($type:ident, $a2a_message_kind:ident) => (
        impl $type {
            pub fn to_a2a_message(&self) -> A2AMessage {
                A2AMessage::$a2a_message_kind(self.clone()) // TODO: THINK how to avoid clone
            }
        }
    );
}
