use std::fmt::Debug;

use crate::{decorators::Thread, message_type::MessageType};

pub trait MessageKind {
    type Kind: Into<MessageType> + PartialEq + Debug;

    fn kind() -> Self::Kind;
}

pub trait Threadlike {
    fn msg_id(&self) -> &str;

    fn opt_thread(&self) -> Option<&Thread>;

    fn thread_id(&self) -> &str {
        self.opt_thread()
            .map(|t| t.thid.as_str())
            .unwrap_or_else(|| self.msg_id())
    }

    fn matches_thread<T>(&self, thread_id: T) -> bool
    where
        T: AsRef<str>,
    {
        self.thread_id() == thread_id.as_ref()
    }
}
