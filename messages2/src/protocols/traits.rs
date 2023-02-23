use std::fmt::Debug;

use crate::{message_type::MessageType, decorators::Thread};

pub trait ConcreteMessage {
    type Kind: Into<MessageType> + PartialEq + Debug;

    fn kind() -> Self::Kind;
}

pub trait Threadlike {
    fn thread(&self) -> &Thread;

    fn thread_id(&self) -> &str {
        &self.thread().thid
    }

    fn matches_thread(&self) -> bool;
}

pub trait ThreadlikeOptional {
    fn opt_thread(&self) -> Option<&Thread>;

    fn opt_thread_id(&self) -> Option<&str> {
        self.opt_thread().map(|t| t.thid.as_str())
    }
}
