#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate strum_macros;

#[macro_use]
pub mod a2a;

#[macro_use]
pub mod concepts;

pub extern crate diddoc;

pub mod status;
pub mod actors;
pub mod utils;
pub mod protocols;
pub mod errors;
