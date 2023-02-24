macro_rules! msg_id {
    () => {
        fn msg_id(&self) -> &str {
            self.id.as_str()
        }
    };
}

macro_rules! threadlike_ack {
    ($type:ident) => {
        impl $crate::protocols::traits::Threadlike for $type {
            fn msg_id(&self) -> &str {
                self.0.id.as_str()
            }

            fn opt_thread(&self) -> Option<&$crate::decorators::Thread> {
                Some(&self.0.thread)
            }
        }
    }
}

macro_rules! threadlike_impl {
    ($type:ident) => {
        impl $crate::protocols::traits::Threadlike for $type {
            $crate::macros::msg_id!();

            fn opt_thread(&self) -> Option<&Thread> {
                Some(&self.thread)
            }
        }
    };
}

macro_rules! threadlike_opt_impl {
    ($type:ident) => {
        impl $crate::protocols::traits::Threadlike for $type {
            $crate::macros::msg_id!();

            fn opt_thread(&self) -> Option<&Thread> {
                self.thread.as_ref()
            }
        }
    };
}

pub(crate) use threadlike_impl;
pub(crate) use threadlike_opt_impl;
pub(crate) use msg_id;
pub(crate) use threadlike_ack;
