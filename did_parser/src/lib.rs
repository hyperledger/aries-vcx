mod did;
mod did_url;
mod error;
mod utils;

use std::ops::Range;

type DidRange = Range<usize>;

pub use did::Did;
pub use did_url::DidUrl;
pub use error::ParseError;
