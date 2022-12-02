use aries_vcx::messages::a2a::A2AMessage;
use actix::Message;

#[derive(Debug, PartialEq, Clone, Message)]
#[rtype(result = "Result<(), String>")]
pub struct A2AMessageActix(pub A2AMessage);

impl Into<A2AMessage> for A2AMessageActix {
    fn into(self) -> A2AMessage {
        self.0
    }
}
