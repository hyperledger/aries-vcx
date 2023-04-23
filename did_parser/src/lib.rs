extern crate once_cell;
extern crate regex;

mod error;
mod parsed_did;
mod parsed_did_url;
mod utils;

use std::ops::Range;

type DIDRange = Range<usize>;

pub use error::ParseError;
pub use parsed_did::ParsedDID;
pub use parsed_did_url::ParsedDIDUrl;
pub use utils::validate::{is_valid_did, is_valid_did_url};
