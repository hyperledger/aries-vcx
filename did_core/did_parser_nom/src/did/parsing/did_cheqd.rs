//! https://docs.cheqd.io/product/architecture/adr-list/adr-001-cheqd-did-method#syntax-for-did-cheqd-method

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, char, one_of},
    combinator::{cut, recognize},
    multi::count,
    sequence::{delimited, terminated, tuple},
    IResult,
};

use super::{did_sov::parse_unqualified_sovrin_did, DidPart, HEX_DIGIT_CHARS};

// namespace = 1*namespace-char ":" ...
fn did_cheqd_namespace(input: &str) -> IResult<&str, &str> {
    terminated(alphanumeric1, tag(":"))(input)
}

// Parser for a single hexDigit
fn hex_digit_char(input: &str) -> IResult<&str, char> {
    one_of(HEX_DIGIT_CHARS)(input)
}

// Parser for hexOctet (2 hex digits)
fn parse_hex_octet(input: &str) -> IResult<&str, &str> {
    recognize(count(hex_digit_char, 2))(input)
}

// https://datatracker.ietf.org/doc/html/rfc4122#section-3
fn parse_uuid(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        count(parse_hex_octet, 4), // time-low
        tag("-"),
        count(parse_hex_octet, 2), // time mid
        tag("-"),
        count(parse_hex_octet, 2), // time high & version
        tag("-"),
        count(parse_hex_octet, 1), // clock sequence and reserved
        count(parse_hex_octet, 1), // clock sequence low
        tag("-"),
        count(parse_hex_octet, 6), // node
    )))(input)
}

// unique-id       = *id-char / UUID
// id-char         = ALPHA / DIGIT
// > Note: The *id-char unique-id must be 16 bytes of Indy-style base58 encoded identifier.
fn parse_did_cheqd_unique_id(input: &str) -> IResult<&str, &str> {
    alt((
        recognize(parse_unqualified_sovrin_did), // indy-style DID ID
        recognize(parse_uuid),                   // UUID-style DID ID
    ))(input)
}

pub(super) fn parse_did_cheqd(input: &str) -> IResult<&str, DidPart> {
    fn did_cheqd_method(input: &str) -> IResult<&str, &str> {
        delimited(char(':'), tag("cheqd"), char(':'))(input)
    }
    let (input_left, (prefix, method, namespace, id)) = tuple((
        tag("did"),
        did_cheqd_method,
        cut(did_cheqd_namespace),
        cut(parse_did_cheqd_unique_id),
    ))(input)?;

    Ok((input_left, (prefix, method, Some(namespace), id)))
}
