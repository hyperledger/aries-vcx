use messages::msg_fields::protocols::connection::ConnectionData;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InviteeRequested {
    pub(crate) recipient_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InviteeResponded {
    pub(crate) con_data: ConnectionData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InviteeComplete {
    pub(crate) con_data: ConnectionData,
}
