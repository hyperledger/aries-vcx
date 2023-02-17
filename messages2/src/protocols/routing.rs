use serde::{Deserialize, Serialize};

use crate::message_type::message_family::{
    routing::{Routing, RoutingV1, RoutingV1_0},
};

use super::traits::ConcreteMessage;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Forward;

impl ConcreteMessage for Forward {
    type Kind = Routing;

    fn kind() -> Self::Kind {
        Self::Kind::V1(RoutingV1::V1_0(RoutingV1_0::Forward))
    }
}
