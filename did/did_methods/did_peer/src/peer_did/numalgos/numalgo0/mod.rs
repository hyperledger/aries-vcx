use crate::peer_did::numalgos::Numalgo;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Numalgo0;

impl Numalgo for Numalgo0 {
    const NUMALGO_CHAR: char = '0';
}
