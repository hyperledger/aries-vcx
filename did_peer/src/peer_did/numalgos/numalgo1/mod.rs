use crate::peer_did::numalgos::Numalgo;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Numalgo1;

impl Numalgo for Numalgo1 {
    const NUMALGO_CHAR: char = '1';
}
