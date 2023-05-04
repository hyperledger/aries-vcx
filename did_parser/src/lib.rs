mod error;
mod parsed_did;
mod parsed_did_url;
mod utils;

use std::ops::Range;

type DidRange = Range<usize>;

pub use error::ParseError;
pub use parsed_did::ParsedDid;
pub use parsed_did_url::DidUrl;
