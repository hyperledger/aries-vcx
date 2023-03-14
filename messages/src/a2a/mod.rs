use serde::{de, ser, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use self::{message_family::MessageFamilies, message_type::MessageType};
use crate::{
    concepts::{ack::Ack, problem_report::ProblemReport as CommonProblemReport},
    protocols::{
        basic_message::message::BasicMessage,
        connection::{
            invite::{PairwiseInvitation, PublicInvitation},
            problem_report::ProblemReport as ConnectionProblemReport,
            request::Request,
            response::SignedResponse,
        },
        discovery::{disclose::Disclose, query::Query},
        issuance::{
            credential::Credential, credential_offer::CredentialOffer, credential_proposal::CredentialProposal,
            credential_request::CredentialRequest,
        },
        out_of_band::{
            handshake_reuse::OutOfBandHandshakeReuse, handshake_reuse_accepted::OutOfBandHandshakeReuseAccepted,
            invitation::OutOfBandInvitation,
        },
        proof_presentation::{
            presentation::Presentation, presentation_proposal::PresentationProposal,
            presentation_request::PresentationRequest,
        },
        revocation_notification::{revocation_ack::RevocationAck, revocation_notification::RevocationNotification},
        routing::forward::Forward,
        trust_ping::{ping::Ping, ping_response::PingResponse},
    },
};

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
    RevocationNotification(RevocationNotification),
    RevocationAck(Ack),
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
    OutOfBandInvitation(OutOfBandInvitation),
    OutOfBandHandshakeReuse(OutOfBandHandshakeReuse),
    OutOfBandHandshakeReuseAccepted(OutOfBandHandshakeReuseAccepted),

    /// Any Raw Message
    Generic(Value),
}

impl A2AMessage {
    pub fn thread_id_matches(&self, thread_id: &str) -> bool {
        match self {
            Self::Presentation(presentation) => presentation.from_thread(thread_id),
            // Self::PingResponse(ping_response) => ping_response.from_thread(thread_id),
            Self::ConnectionProblemReport(connection_problem_report) => {
                connection_problem_report.from_thread(thread_id)
            }
            Self::ConnectionRequest(request) => request.from_thread(thread_id),
            Self::CommonProblemReport(common_problem_report) => common_problem_report.from_thread(thread_id),
            Self::CredentialOffer(credential_offer) => credential_offer.from_thread(thread_id),
            Self::CredentialProposal(credential_proposal) => credential_proposal.from_thread(thread_id),
            Self::Credential(credential) => credential.from_thread(thread_id),
            Self::PresentationProposal(presentation_proposal) => presentation_proposal.from_thread(thread_id),
            Self::RevocationNotification(m) => m.from_thread(thread_id),
            Self::PresentationAck(ack) | Self::CredentialAck(ack) | Self::RevocationAck(ack) | Self::Ack(ack) => {
                ack.from_thread(thread_id)
            }
            Self::Ping(ping) => ping.from_thread(thread_id),
            Self::PingResponse(ping) => ping.from_thread(thread_id),
            Self::ConnectionResponse(m) => m.from_thread(thread_id),
            Self::CredentialRequest(m) => m.from_thread(thread_id),
            Self::PresentationRequest(m) => m.from_thread(thread_id),
            Self::Disclose(m) => m.from_thread(thread_id),
            Self::OutOfBandHandshakeReuse(m) => m.from_thread(thread_id),
            Self::OutOfBandHandshakeReuseAccepted(m) => m.from_thread(thread_id),
            Self::Forward(_) => false,
            Self::ConnectionInvitationPairwise(_) => false,
            Self::ConnectionInvitationPublic(_) => false,
            Self::Query(_) => false,
            Self::OutOfBandInvitation(_) => false,
            Self::BasicMessage(m) => m.from_thread(thread_id),
            Self::Generic(m) => {
                return match m.as_object() {
                    None => false,
                    Some(msg) => match msg.get("~thread") {
                        None => false,
                        Some(thread) => match thread["thid"].as_str() {
                            None => false,
                            Some(thid) => thid == thread_id,
                        },
                    },
                }
            }
        }
    }
}

impl<'de> Deserialize<'de> for A2AMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer).map_err(de::Error::custom)?;

        let message_type: MessageType = match serde_json::from_value(value["@type"].clone()) {
            Ok(message_type) => message_type,
            Err(_) => return Ok(A2AMessage::Generic(value)),
        };

        match (message_type.family, message_type.msg_type.as_str()) {
            (MessageFamilies::Routing, A2AMessage::FORWARD) => Forward::deserialize(value)
                .map(A2AMessage::Forward)
                .map_err(de::Error::custom),
            (MessageFamilies::Connections, A2AMessage::CONNECTION_INVITATION) => {
                PairwiseInvitation::deserialize(value.clone())
                    .map_or(
                        PublicInvitation::deserialize(value).map(A2AMessage::ConnectionInvitationPublic),
                        |msg| Ok(A2AMessage::ConnectionInvitationPairwise(msg)),
                    )
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Connections, A2AMessage::CONNECTION_REQUEST) => Request::deserialize(value)
                .map(A2AMessage::ConnectionRequest)
                .map_err(de::Error::custom),
            (MessageFamilies::Connections, A2AMessage::CONNECTION_RESPONSE) => SignedResponse::deserialize(value)
                .map(A2AMessage::ConnectionResponse)
                .map_err(de::Error::custom),
            (MessageFamilies::TrustPing, A2AMessage::PING) => Ping::deserialize(value)
                .map(A2AMessage::Ping)
                .map_err(de::Error::custom),
            (MessageFamilies::TrustPing, A2AMessage::PING_RESPONSE) => PingResponse::deserialize(value)
                .map(A2AMessage::PingResponse)
                .map_err(de::Error::custom),
            (MessageFamilies::Connections, A2AMessage::CONNECTION_PROBLEM_REPORT) => {
                ConnectionProblemReport::deserialize(value)
                    .map(A2AMessage::ConnectionProblemReport)
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::Notification, A2AMessage::ACK) => {
                Ack::deserialize(value).map(A2AMessage::Ack).map_err(de::Error::custom)
            }
            (MessageFamilies::ReportProblem, A2AMessage::PROBLEM_REPORT) => CommonProblemReport::deserialize(value)
                .map(A2AMessage::CommonProblemReport)
                .map_err(de::Error::custom),
            (MessageFamilies::CredentialIssuance, A2AMessage::CREDENTIAL) => Credential::deserialize(value)
                .map(A2AMessage::Credential)
                .map_err(de::Error::custom),
            (MessageFamilies::CredentialIssuance, A2AMessage::PROPOSE_CREDENTIAL) => {
                CredentialProposal::deserialize(value)
                    .map(A2AMessage::CredentialProposal)
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::CredentialIssuance, A2AMessage::CREDENTIAL_OFFER) => CredentialOffer::deserialize(value)
                .map(A2AMessage::CredentialOffer)
                .map_err(de::Error::custom),
            (MessageFamilies::CredentialIssuance, A2AMessage::REQUEST_CREDENTIAL) => {
                CredentialRequest::deserialize(value)
                    .map(A2AMessage::CredentialRequest)
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::CredentialIssuance, A2AMessage::ACK) => Ack::deserialize(value)
                .map(A2AMessage::CredentialAck)
                .map_err(de::Error::custom),
            (MessageFamilies::PresentProof, A2AMessage::PROPOSE_PRESENTATION) => {
                PresentationProposal::deserialize(value)
                    .map(A2AMessage::PresentationProposal)
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::PresentProof, A2AMessage::REQUEST_PRESENTATION) => {
                PresentationRequest::deserialize(value)
                    .map(A2AMessage::PresentationRequest)
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::PresentProof, A2AMessage::PRESENTATION) => Presentation::deserialize(value)
                .map(A2AMessage::Presentation)
                .map_err(de::Error::custom),
            (MessageFamilies::PresentProof, A2AMessage::ACK) => Ack::deserialize(value)
                .map(A2AMessage::PresentationAck)
                .map_err(de::Error::custom),
            (MessageFamilies::DiscoveryFeatures, A2AMessage::QUERY) => Query::deserialize(value)
                .map(A2AMessage::Query)
                .map_err(de::Error::custom),
            (MessageFamilies::DiscoveryFeatures, A2AMessage::DISCLOSE) => Disclose::deserialize(value)
                .map(A2AMessage::Disclose)
                .map_err(de::Error::custom),
            (MessageFamilies::Basicmessage, A2AMessage::BASIC_MESSAGE) => BasicMessage::deserialize(value)
                .map(A2AMessage::BasicMessage)
                .map_err(de::Error::custom),
            (MessageFamilies::OutOfBand, A2AMessage::OUT_OF_BAND_INVITATION) => OutOfBandInvitation::deserialize(value)
                .map(A2AMessage::OutOfBandInvitation)
                .map_err(de::Error::custom),
            (MessageFamilies::OutOfBand, A2AMessage::OUT_OF_BAND_HANDSHAKE_REUSE) => {
                OutOfBandHandshakeReuse::deserialize(value)
                    .map(A2AMessage::OutOfBandHandshakeReuse)
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::OutOfBand, A2AMessage::OUT_OF_BAND_HANDSHAKE_REUSE_ACCEPTED) => {
                OutOfBandHandshakeReuseAccepted::deserialize(value)
                    .map(A2AMessage::OutOfBandHandshakeReuseAccepted)
                    .map_err(de::Error::custom)
            }
            (MessageFamilies::RevocationNotification, A2AMessage::REVOKE) => RevocationNotification::deserialize(value)
                .map(A2AMessage::RevocationNotification)
                .map_err(de::Error::custom),
            (MessageFamilies::RevocationNotification, A2AMessage::ACK) => RevocationAck::deserialize(value)
                .map(A2AMessage::RevocationAck)
                .map_err(de::Error::custom),
            (_, _) => Ok(A2AMessage::Generic(value)),
        }
    }
}

fn set_a2a_message_type<T>(msg: T, family: MessageFamilies, name: &str) -> Result<serde_json::Value, serde_json::Error>
where
    T: Serialize,
{
    let mut value = ::serde_json::to_value(msg)?;
    let _value = value.clone();
    let type_ = ::serde_json::to_value(MessageType::build(family, name))?;
    value
        .as_object_mut()
        .ok_or(ser::Error::custom(format!(
            "failed to interpret Value as an Object: {}",
            _value
        )))?
        .insert("@type".into(), type_);
    Ok(value)
}

impl Serialize for A2AMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            A2AMessage::Forward(msg) => set_a2a_message_type(msg, MessageFamilies::Routing, A2AMessage::FORWARD),
            A2AMessage::ConnectionInvitationPairwise(msg) => {
                set_a2a_message_type(msg, MessageFamilies::Connections, A2AMessage::CONNECTION_INVITATION)
            }
            A2AMessage::ConnectionInvitationPublic(msg) => {
                set_a2a_message_type(msg, MessageFamilies::Connections, A2AMessage::CONNECTION_INVITATION)
            }
            A2AMessage::ConnectionRequest(msg) => {
                set_a2a_message_type(msg, MessageFamilies::Connections, A2AMessage::CONNECTION_REQUEST)
            }
            A2AMessage::ConnectionResponse(msg) => {
                set_a2a_message_type(msg, MessageFamilies::Connections, A2AMessage::CONNECTION_RESPONSE)
            }
            A2AMessage::ConnectionProblemReport(msg) => {
                set_a2a_message_type(msg, MessageFamilies::Connections, A2AMessage::CONNECTION_PROBLEM_REPORT)
            }
            A2AMessage::Ping(msg) => set_a2a_message_type(msg, MessageFamilies::TrustPing, A2AMessage::PING),
            A2AMessage::PingResponse(msg) => {
                set_a2a_message_type(msg, MessageFamilies::TrustPing, A2AMessage::PING_RESPONSE)
            }
            A2AMessage::Ack(msg) => set_a2a_message_type(msg, MessageFamilies::Notification, A2AMessage::ACK),
            A2AMessage::CommonProblemReport(msg) => {
                set_a2a_message_type(msg, MessageFamilies::ReportProblem, A2AMessage::PROBLEM_REPORT)
            }
            A2AMessage::CredentialOffer(msg) => {
                set_a2a_message_type(msg, MessageFamilies::CredentialIssuance, A2AMessage::CREDENTIAL_OFFER)
            }
            A2AMessage::Credential(msg) => {
                set_a2a_message_type(msg, MessageFamilies::CredentialIssuance, A2AMessage::CREDENTIAL)
            }
            A2AMessage::CredentialProposal(msg) => {
                set_a2a_message_type(msg, MessageFamilies::CredentialIssuance, A2AMessage::PROPOSE_CREDENTIAL)
            }
            A2AMessage::CredentialRequest(msg) => {
                set_a2a_message_type(msg, MessageFamilies::CredentialIssuance, A2AMessage::REQUEST_CREDENTIAL)
            }
            A2AMessage::CredentialAck(msg) => {
                set_a2a_message_type(msg, MessageFamilies::CredentialIssuance, A2AMessage::ACK)
            }
            A2AMessage::RevocationNotification(msg) => {
                set_a2a_message_type(msg, MessageFamilies::RevocationNotification, A2AMessage::REVOKE)
            }
            A2AMessage::RevocationAck(msg) => {
                set_a2a_message_type(msg, MessageFamilies::RevocationNotification, A2AMessage::ACK)
            }
            A2AMessage::PresentationProposal(msg) => {
                set_a2a_message_type(msg, MessageFamilies::PresentProof, A2AMessage::PROPOSE_PRESENTATION)
            }
            A2AMessage::PresentationRequest(msg) => {
                set_a2a_message_type(msg, MessageFamilies::PresentProof, A2AMessage::REQUEST_PRESENTATION)
            }
            A2AMessage::Presentation(msg) => {
                set_a2a_message_type(msg, MessageFamilies::PresentProof, A2AMessage::PRESENTATION)
            }
            A2AMessage::PresentationAck(msg) => {
                set_a2a_message_type(msg, MessageFamilies::PresentProof, A2AMessage::ACK)
            }
            A2AMessage::Query(msg) => set_a2a_message_type(msg, MessageFamilies::DiscoveryFeatures, A2AMessage::QUERY),
            A2AMessage::Disclose(msg) => {
                set_a2a_message_type(msg, MessageFamilies::DiscoveryFeatures, A2AMessage::DISCLOSE)
            }
            A2AMessage::BasicMessage(msg) => {
                set_a2a_message_type(msg, MessageFamilies::Basicmessage, A2AMessage::BASIC_MESSAGE)
            }
            A2AMessage::OutOfBandInvitation(msg) => {
                set_a2a_message_type(msg, MessageFamilies::OutOfBand, A2AMessage::OUT_OF_BAND_INVITATION)
            }
            A2AMessage::OutOfBandHandshakeReuse(msg) => {
                set_a2a_message_type(msg, MessageFamilies::OutOfBand, A2AMessage::OUT_OF_BAND_HANDSHAKE_REUSE)
            }
            A2AMessage::OutOfBandHandshakeReuseAccepted(msg) => set_a2a_message_type(
                msg,
                MessageFamilies::OutOfBand,
                A2AMessage::OUT_OF_BAND_HANDSHAKE_REUSE_ACCEPTED,
            ),
            A2AMessage::Generic(msg) => Ok(msg.clone()),
        }
        .map_err(ser::Error::custom)?;

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
    #[cfg(not(feature = "test_utils"))]
    fn default() -> MessageId {
        MessageId(uuid::Uuid::new_v4().to_string())
    }

    #[cfg(not(test))]
    #[cfg(feature = "test_utils")]
    fn default() -> MessageId {
        MessageId::id()
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
    const REVOKE: &'static str = "revoke";
    const PROPOSE_PRESENTATION: &'static str = "propose-presentation";
    const REQUEST_PRESENTATION: &'static str = "request-presentation";
    const PRESENTATION: &'static str = "presentation";
    const QUERY: &'static str = "query";
    const DISCLOSE: &'static str = "disclose";
    const BASIC_MESSAGE: &'static str = "message";
    const OUT_OF_BAND_INVITATION: &'static str = "invitation";
    const OUT_OF_BAND_HANDSHAKE_REUSE: &'static str = "handshake-reuse";
    const OUT_OF_BAND_HANDSHAKE_REUSE_ACCEPTED: &'static str = "handshake-reuse-accepted";
}

#[cfg(test)]
#[cfg(feature = "general_test")]
pub mod test_a2a_serialization {
    use serde_json::Value;

    use crate::{
        a2a::{A2AMessage, MessageId},
        concepts::ack::{Ack, AckStatus},
        protocols::{connection::request::Request, routing::forward::Forward},
    };

    #[test]
    fn test_serialization_deserialization_connection_request() {
        let a2a_msg = A2AMessage::ConnectionRequest(Request {
            id: Default::default(),
            label: "foobar".to_string(),
            connection: Default::default(),
            thread: None,
            timing: None,
        });
        let serialized = serde_json::to_string(&a2a_msg).unwrap();

        // serialization
        let val: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(val["@id"], Value::String("testid".into()));
        assert_eq!(
            val["@type"],
            Value::String("https://didcomm.org/connections/1.0/request".into())
        );
        assert_eq!(val["label"], Value::String("foobar".into()));
        assert_eq!(val["connection"]["DID"], Value::String("".into()));
        assert!(val["connection"]["DIDDoc"].is_object());

        // deserialization back
        let a2a_msg: A2AMessage = serde_json::from_str(&serialized).unwrap();
        if let A2AMessage::ConnectionRequest(request) = &a2a_msg {
            assert_eq!(request.id, MessageId("testid".into()));
            assert_eq!(request.label, "foobar");
        } else {
            panic!(
                "The message was expected to be deserialized as ConnectionRequest, but was not. Deserialized: {:?} ",
                a2a_msg
            )
        }
    }

    #[test]
    fn test_serialize_deserialize_connection_ack() {
        let a2a_msg = A2AMessage::Ack(Ack::create().set_status(AckStatus::Ok).set_thread_id("threadid"));
        let serialized = serde_json::to_string(&a2a_msg).unwrap();

        // serialization
        let val: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(val["@id"], Value::String("testid".into()));
        assert_eq!(
            val["@type"],
            Value::String("https://didcomm.org/notification/1.0/ack".into())
        );
        assert_eq!(val["status"], Value::String("OK".into()));
        assert!(val["~thread"].is_object());

        // deserialization back
        let a2a_msg: A2AMessage = serde_json::from_str(&serialized).unwrap();
        if let A2AMessage::Ack(ack) = &a2a_msg {
            assert_eq!(ack.id, MessageId("testid".into()));
            assert_eq!(ack.thread.sender_order, 0);
        } else {
            panic!(
                "The message was expected to be deserialized as ConnectionRequest, but was not. Deserialized: {:?} ",
                a2a_msg
            )
        }
    }

    #[test]
    #[cfg(feature = "general_test")]
    // todo: Add support for aries @type-ed messages on vcxagency-node, then we can stop giving fwd
    // messages special treatment, delete this test
    fn test_serialize_forward_message_to_legacy_format() {
        let a2a_msg =
            A2AMessage::Forward(Forward::new("BzCbsNYhMrjHiqZDTUASHg".into(), "{}".as_bytes().to_vec()).unwrap());
        let serialized = serde_json::to_string(&a2a_msg).unwrap();

        // serialization
        let val: Value = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            val["@type"],
            Value::String("did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/routing/1.0/forward".into())
        );
        assert_eq!(val["@id"], Value::String("testid".into()));
    }

    #[test]
    fn test_deserialize_connection_ack_legacy() {
        let msg = r#"{
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
    fn test_deserialization_connection_request_legacy() {
        let msg = r#"{
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
    ($type:ident) => {
        impl $type {
            pub fn to_a2a_message(&self) -> A2AMessage {
                A2AMessage::$type(self.clone()) // TODO: THINK how to avoid clone
            }
        }
    };

    ($type:ident, $a2a_message_kind:ident) => {
        impl $type {
            pub fn to_a2a_message(&self) -> A2AMessage {
                A2AMessage::$a2a_message_kind(self.clone()) // TODO: THINK how to avoid clone
            }
        }
    };
}
