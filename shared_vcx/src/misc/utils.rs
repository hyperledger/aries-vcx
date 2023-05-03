/// Wrapper used for allowing borrowing behavior on [`Cow<'_, str>`] where possible.
use std::borrow::Cow;

use serde::Deserialize;
/// See: <https://github.com/serde-rs/serde/issues/1852>
#[derive(Debug, PartialEq, Deserialize)]
#[serde(transparent)]
pub struct CowStr<'a>(#[serde(borrow)] pub Cow<'a, str>);
