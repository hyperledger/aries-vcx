use messages::msg_fields::protocols::connection::ConnectionData;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InviterRequested {
    pub(crate) con_data: ConnectionData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InviterResponded {
    pub(crate) con_data: ConnectionData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InviterComplete {
    pub(crate) con_data: ConnectionData,
}
