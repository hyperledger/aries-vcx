mod did_cheqd;
mod did_core;
mod did_key;
mod did_peer_4;
mod did_sov;
mod did_web;

use did_cheqd::parse_did_cheqd;
use nom::{
    branch::alt,
    combinator::{all_consuming, map},
    IResult,
};

use self::{
    did_core::parse_qualified_did,
    did_key::parse_did_key,
    did_sov::{parse_qualified_sovrin_did, parse_unqualified_sovrin_did},
    did_web::parse_did_web,
};
use crate::{did::parsing::did_peer_4::parse_did_peer_4, Did, DidRange, ParseError};

type DidPart<'a> = (&'a str, &'a str, Option<&'a str>, &'a str);
pub type DidRanges = (Option<DidRange>, Option<DidRange>, Option<DidRange>);

static BASE58CHARS: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
static HEX_DIGIT_CHARS: &str = "0123456789abcdefABCDEF";

fn to_id_range(id: &str) -> DidRanges {
    (None, None, Some(0..id.len()))
}

fn to_did_ranges((did_prefix, method, namespace, id): DidPart) -> DidRanges {
    let mut next_start = if !did_prefix.is_empty() {
        did_prefix.len() + 1
    } else {
        0
    };

    let method_range = if !method.is_empty() {
        let method_start = next_start;
        let method_end = method_start + method.len();
        next_start = method_end + 1;
        Some(method_start..method_end)
    } else {
        next_start = 0;
        None
    };

    let namespace_range = namespace.map(|ns| {
        let namespace_start = next_start;
        let namespace_end = namespace_start + ns.len();
        next_start = namespace_end + 1;
        namespace_start..namespace_end
    });

    let id_start = next_start;
    let id_end = id_start + id.len();
    let id_range = match id_start..id_end {
        range if range.is_empty() => None,
        range => Some(range),
    };

    (method_range, namespace_range, id_range)
}

pub fn parse_did_ranges(input: &str) -> IResult<&str, DidRanges> {
    alt((
        map(parse_did_peer_4, to_did_ranges),
        map(parse_did_web, to_did_ranges),
        map(parse_did_key, to_did_ranges),
        map(parse_did_cheqd, to_did_ranges),
        map(parse_qualified_sovrin_did, to_did_ranges),
        map(parse_qualified_did, to_did_ranges),
        map(parse_unqualified_sovrin_did, to_id_range),
    ))(input)
}

pub fn parse_did(did: String) -> Result<Did, ParseError> {
    if did.is_empty() {
        return Err(ParseError::InvalidInput("Empty input"));
    }

    let (_, (method, namespace, id)) = all_consuming(parse_did_ranges)(&did)?;
    let id = id.ok_or_else(|| ParseError::InvalidInput("Invalid DID"))?;

    if id.end > did.len() {
        return Err(ParseError::InvalidInput("Invalid DID"));
    }

    Ok(Did {
        did,
        method,
        namespace,
        id,
    })
}
