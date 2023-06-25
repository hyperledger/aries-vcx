// TODO: Where else should we move this file?
// depends on: multibase, bs58, unsigned_varint
// two of which are also deps of did_doc, but did_doc does not use this abstraction

mod key;
mod key_type;
mod verification_method;

pub use key::Key;
pub use key_type::KeyType;
pub use verification_method::{get_key_by_verification_method, get_verification_methods_by_key};
