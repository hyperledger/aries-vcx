// https://www.w3.org/TR/did-core/#did-syntax
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{alphanumeric1, char, satisfy},
    combinator::recognize,
    multi::many1,
    sequence::{delimited, tuple},
    AsChar, IResult,
};

use super::DidPart;

fn hexadecimal_digit(input: &str) -> IResult<&str, &str> {
    fn is_hexadecimal_digit(c: char) -> bool {
        c.is_ascii_hexdigit()
    }

    recognize(satisfy(is_hexadecimal_digit))(input)
}

// todo: is this better?
// fn hexadecimal_digit(input: &str) -> IResult<&str, &str> {
//     recognize(alt((nom::character::is_hex_digit, nom::character::is_hex_digit)))(input)
// }

fn is_lowercase_alphanumeric(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_dec_digit()
}

// pct-encoded = "%" HEXDIG HEXDIG
fn pct_encoded(input: &str) -> IResult<&str, &str> {
    recognize(tuple((tag("%"), hexadecimal_digit, hexadecimal_digit)))(input)
}

// idchar = ALPHA / DIGIT / "." / "-" / "_" / pct-encoded
pub(super) fn idchar(input: &str) -> IResult<&str, &str> {
    alt((alphanumeric1, tag("."), tag("-"), tag("_"), pct_encoded))(input)
}

// method-name = 1*method-char
// method-char = %x61-7A / DIGIT
fn method_name(input: &str) -> IResult<&str, &str> {
    delimited(char(':'), take_while1(is_lowercase_alphanumeric), char(':'))(input)
}

// method-specific-id = *namespace 1*idchar
fn method_specific_id(input: &str) -> IResult<&str, &str> {
    recognize(many1(idchar))(input)
}

// did = "did:" method-name ":" method-specific-id
pub(super) fn parse_qualified_did(input: &str) -> IResult<&str, DidPart> {
    let (input_left, (prefix, method, id)) =
        tuple((tag("did"), method_name, method_specific_id))(input)?;

    Ok((input_left, (prefix, method, None, id)))
}
