use aries_vcx_agent::{Agent as AriesAgent, aries_vcx::messages::a2a::A2AMessage};

pub struct CliAriesAgent {
    agent: AriesAgent,
    messages: Vec<A2AMessage>
}

impl CliAriesAgent {
    pub fn new(agent: AriesAgent) -> Self { Self { agent, messages: Vec::new() } }

    pub fn agent(&self) -> &AriesAgent {
        &self.agent
    }

    pub fn push_message(&mut self, message: A2AMessage) {
        self.messages.push(message);
    }

    pub fn messages(&self) -> &[A2AMessage] {
        self.messages.as_ref()
    }
}
