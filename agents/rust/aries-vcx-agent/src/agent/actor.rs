use actix::{Actor, Context, Handler, ResponseFuture};
use aries_vcx::messages::a2a::A2AMessage;

use crate::Agent;
use crate::agent::a2a_msg_actix::A2AMessageActix;

impl Actor for Agent {
    type Context = Context<Self>;
}

impl Handler<A2AMessageActix> for Agent {
    type Result = ResponseFuture<Result<(), String>>;

    fn handle(&mut self, msg: A2AMessageActix, _ctx: &mut Context<Self>) -> Self::Result {
        match self.received_messages().write() {
            Ok(mut g) => g.push_back(msg.clone().into()),
            Err(e) => warn!("Error ackquiring lock: {}", e)
        };
        match msg.0 {
            A2AMessage::ConnectionRequest(request) => {
                let conns = self.connections().clone();
                Box::pin(async move {
                    conns
                        .accept_request(&request.get_thread_id(), request)
                        .await
                        .map_err(|err| err.to_string())
                })
            }
            A2AMessage::ConnectionResponse(response) => {
                let conns = self.connections().clone();
                Box::pin(async move {
                    conns
                        .accept_response(&response.get_thread_id(), response)
                        .await
                        .map_err(|err| err.to_string())
                })
            }
            A2AMessage::Ack(ack) => {
                let conns = self.connections().clone();
                Box::pin(async move {
                    conns
                        .process_ack(&ack.get_thread_id(), ack)
                        .await
                        .map_err(|err| err.to_string())
                })
            }
            _ => Box::pin(async move { Ok(()) })
        }
    }
}
