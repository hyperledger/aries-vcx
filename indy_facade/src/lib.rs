pub extern crate indy_sys;
pub extern crate indyrs as indy;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

pub mod error;
pub mod wallet;
mod utils;