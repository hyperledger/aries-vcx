use std::{
    fmt::Display,
    sync::{Arc, RwLock},
};

use aries_vcx_agent::aries_vcx::messages::{
    a2a::A2AMessage,
    connection::{
        problem_report::ProblemReport as ConnectionProblemReport, request::Request as ConnectionRequest,
        response::SignedResponse as ConnectionResponse,
    }, ack::Ack,
};
use serde_json::Value;

use crate::agent::CliAriesAgent;

pub enum MessagesCommand {
    ConnectionRequest(ConnectionRequest),
    ConnectionResponse(ConnectionResponse),
    ConnectionProblemReport(ConnectionProblemReport),
    Ack(Ack),
    Generic(Value),
    GoBack,
}

impl From<&A2AMessage> for MessagesCommand {
    fn from(message: &A2AMessage) -> Self {
        match message {
            A2AMessage::ConnectionRequest(request) => Self::ConnectionRequest(request.clone()),
            A2AMessage::ConnectionResponse(response) => Self::ConnectionResponse(response.clone()),
            A2AMessage::ConnectionProblemReport(problem_report) => {
                Self::ConnectionProblemReport(problem_report.clone())
            }
            A2AMessage::Ack(ack) => Self::Ack(ack.clone()),
            _ => Self::Generic(serde_json::to_value(message).unwrap()),
        }
    }
}

impl Display for MessagesCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionRequest(m) => f.write_fmt(format_args!("Connection Request: {:?}", m)),
            Self::ConnectionResponse(m) => f.write_fmt(format_args!("Connection Response: {:?}", m)),
            Self::ConnectionProblemReport(m) => f.write_fmt(format_args!("Connection Problem Report: {:?}", m)),
            Self::Ack(m) => f.write_fmt(format_args!("Ack: {:?}", m)),
            Self::Generic(m) => f.write_fmt(format_args!("Generic: {:?}", m)),
            Self::GoBack => f.write_str("Back"),
        }
    }
}

pub fn get_messages(agent: &Arc<RwLock<CliAriesAgent>>) -> Vec<MessagesCommand> {
    let mut msgs: Vec<MessagesCommand> = agent
        .read()
        .unwrap()
        .messages()
        .iter()
        .map(MessagesCommand::from)
        .collect();
    msgs.push(MessagesCommand::GoBack);
    msgs
}
