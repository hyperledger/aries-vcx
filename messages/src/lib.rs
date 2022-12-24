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

pub mod actors;
pub mod errors;
pub mod protocols;
pub mod status;
pub mod utils;
