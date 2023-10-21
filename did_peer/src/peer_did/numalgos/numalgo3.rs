use super::traits::Numalgo;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Numalgo3;

impl Numalgo for Numalgo3 {
    const NUMALGO_CHAR: char = '3';
}
