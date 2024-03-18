use nom::{
    bytes::complete::tag,
    character::complete::char,
    combinator::{cut, peek, recognize},
    multi::{many0, many1},
    sequence::{delimited, terminated, tuple},
    IResult,
};

use super::DidPart;
use crate::did::parsing::did_core::idchar;

fn did_peer_method(input: &str) -> IResult<&str, &str> {
    delimited(char(':'), tag("peer"), char(':'))(input)
}

fn did_peer_4_id(input: &str) -> IResult<&str, &str> {
    let (input, did_id) = recognize(tuple((
            tag("4"),
            many0(terminated(many0(idchar), char(':'))), // First half of DID Syntax ABNF rule method-specific-id: *( *idchar ":" )
            many1(idchar) // Second half of DID Syntax ABNF rule method-specific-id: 1*idchar
        )))(input)?;
    Ok((input, did_id))
}

fn check_4(input: &str) -> IResult<&str, &str> {
    peek(tag("4"))(input)
}

pub(super) fn parse_did_peer_4(input: &str) -> IResult<&str, DidPart> {
    let (_input_left, (prefix, method, _peek, id)) =
        tuple((tag("did"), did_peer_method, check_4, cut(did_peer_4_id)))(input)?;

    let input_left = input;
    Ok((input_left, (prefix, method, None, id)))
}
