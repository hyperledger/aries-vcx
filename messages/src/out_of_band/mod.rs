pub mod handshake_reuse;
pub mod handshake_reuse_accepted;
pub mod invitation;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum GoalCode {
    #[serde(rename = "issue-vc")]
    IssueVC,
    #[serde(rename = "request-proof")]
    RequestProof,
    #[serde(rename = "create-account")]
    CreateAccount,
    #[serde(rename = "p2p-messaging")]
    P2PMessaging,
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum HandshakeProtocol {
    ConnectionV1,
    DidExchangeV1,
}
