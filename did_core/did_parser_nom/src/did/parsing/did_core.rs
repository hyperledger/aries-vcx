// https://www.w3.org/TR/did-core/#did-syntax
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{alphanumeric1, char, satisfy},
    combinator::{all_consuming, cut, opt, recognize},
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

fn is_lowercase_alphanumeric(c: char) -> bool {
    c.is_ascii_lowercase() || c.is_dec_digit()
}

// pct-encoded = "%" HEXDIG HEXDIG
fn pct_encoded(input: &str) -> IResult<&str, &str> {
    recognize(tuple((tag("%"), hexadecimal_digit, hexadecimal_digit)))(input)
}

// idchar = ALPHA / DIGIT / "." / "-" / "_" / pct-encoded
fn idchar(input: &str) -> IResult<&str, &str> {
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

// namespace =  *idchar ":"
pub(super) fn namespace(input: &str) -> IResult<&str, &str> {
    if let Some((before_last_colon, after_last_colon)) = input.rsplit_once(':') {
        match cut(all_consuming(many1(alt((idchar, tag(":"))))))(before_last_colon) {
            Ok(_) => Ok((after_last_colon, before_last_colon)),
            Err(err) => Err(err),
        }
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )))
    }
}

// did = "did:" method-name ":" method-specific-id
pub(super) fn parse_qualified_did(input: &str) -> IResult<&str, DidPart> {
    tuple((tag("did"), method_name, opt(namespace), method_specific_id))(input)
}
