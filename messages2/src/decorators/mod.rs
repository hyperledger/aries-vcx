mod attachment;
mod localization;
mod please_ack;
mod thread;
mod timing;

pub use attachment::Attachment;
pub use localization::{FieldLocalization, Locale, MsgLocalization};
pub use please_ack::PleaseAck;
pub use thread::Thread;
pub use timing::Timing;

/// Trait used for easy and consistent conditional serialization
/// of data structures where all fields can be somehow empty (None, length 0, etc).
///
/// E.g:
/// ```
/// #[serde(skip_serializing_if = "EmptyDecorator::is_empty")]
/// ```
pub trait EmptyDecorator {
    fn is_empty(&self) -> bool;
}
