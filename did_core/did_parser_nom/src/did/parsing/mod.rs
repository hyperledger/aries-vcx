mod did_core;
mod did_sov;
mod did_web;

use nom::{
    branch::alt,
    combinator::{all_consuming, map},
    IResult,
};

use crate::{Did, DidRange, ParseError};

use self::{
    did_core::{parse_qualified_did, to_did_ranges, to_id_range},
    did_sov::{parse_qualified_sovrin_did, parse_unqualified_sovrin_did},
    did_web::parse_did_web,
};

type DidPart<'a> = (&'a str, &'a str, Option<&'a str>, &'a str);
pub type DidRanges = (Option<DidRange>, Option<DidRange>, Option<DidRange>);

pub fn parse_did_ranges(input: &str) -> IResult<&str, DidRanges> {
    alt((
        map(parse_did_web, to_did_ranges),
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
