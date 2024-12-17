use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, one_of, satisfy},
    combinator::{all_consuming, cut, opt, recognize, success},
    multi::{many0, many1, separated_list0},
    sequence::{preceded, separated_pair, tuple},
    AsChar, IResult,
};

type UrlPart<'a> = (&'a str, Option<Vec<(&'a str, &'a str)>>, Option<&'a str>);

use crate::{
    did::parsing::{parse_did_ranges, DidRanges},
    DidRange, DidUrl, ParseError,
};

// unreserved  = ALPHA / DIGIT / "-" / "." / "_" / "~"
fn is_unreserved(c: char) -> bool {
    c.is_ascii_alphabetic() || c.is_ascii_digit() || "-._~".contains(c)
}

// sub-delims = "!" / "$" / "&" / "'" / "(" / ")" / "*" / "+" / "," / ";" / "="
fn is_sub_delims(c: char) -> bool {
    "!$&'()*+,;=".contains(c)
}

// pct-encoded = "%" HEXDIG HEXDIG
fn pct_encoded(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        tag("%"),
        satisfy(|c| c.is_hex_digit()),
        satisfy(|c| c.is_hex_digit()),
    )))(input)
}

// pchar = unreserved / pct-encoded / sub-delims / ":" / "@"
fn pchar(input: &str) -> IResult<&str, &str> {
    alt((
        recognize(satisfy(is_unreserved)),
        pct_encoded,
        recognize(satisfy(is_sub_delims)),
        tag(":"),
        tag("@"),
    ))(input)
}

// segment = *pchar
fn segment(input: &str) -> IResult<&str, &str> {
    recognize(many1(pchar))(input)
}

// path-abempty = *( "/" segment )
fn path_abempty(input: &str) -> IResult<&str, &str> {
    recognize(many0(preceded(tag("/"), segment)))(input)
}

// fragment = *( pchar / "/" / "?" )
pub(super) fn fragment_parser(input: &str) -> IResult<&str, &str> {
    fn fragment_element(input: &str) -> IResult<&str, &str> {
        alt(((pchar), tag("/"), tag("?")))(input)
    }

    recognize(many1(fragment_element))(input)
}

// query = *( pchar / "/" / "?" )
fn query_key_value_pair(input: &str) -> IResult<&str, (&str, &str)> {
    fn query_element(input: &str) -> IResult<&str, &str> {
        alt(((pchar), tag("/"), tag("?")))(input)
    }

    let (remaining, (key, value)) = cut(separated_pair(
        take_while1(|c| !"=&#".contains(c)),
        char('='),
        alt((take_while1(|c| !"&#?".contains(c)), success(""))),
    ))(input)?;

    cut(all_consuming(many1(query_element)))(key)?;
    if !value.is_empty() {
        cut(all_consuming(many1(query_element)))(value)?;
    }

    Ok((remaining, (key, value)))
}

fn query_parser(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
    separated_list0(one_of("&?"), query_key_value_pair)(input)
}

fn parse_did_ranges_with_empty_allowed(input: &str) -> IResult<&str, DidRanges> {
    alt((
        parse_did_ranges,
        success(Default::default()), // Relative DID URL
    ))(input)
}

// did-url-remaining = path-abempty [ "?" query ] [ "#" fragment ]
fn parse_url_part(input: &str) -> IResult<&str, UrlPart> {
    let (remaining, path) = path_abempty(input)?;
    let (remaining, queries) = opt(preceded(tag("?"), cut(query_parser)))(remaining)?;
    let (remaining, fragment) =
        opt(preceded(tag("#"), cut(all_consuming(fragment_parser))))(remaining)?;
    Ok((remaining, (path, queries, fragment)))
}

fn to_did_url_ranges(
    id_range: Option<DidRange>,
    (path, queries, fragment): UrlPart,
) -> (
    Option<DidRange>,
    HashMap<DidRange, DidRange>,
    Option<DidRange>,
) {
    let id_end = id_range.unwrap_or_default().end;
    let path_range = if path.is_empty() {
        None
    } else {
        let path_start = id_end;
        let path_end = path_start + path.len();
        Some(path_start..path_end)
    };

    let mut current_last_position = path_range
        .clone()
        .map(|range| range.end + 1)
        .unwrap_or(id_end + 1);

    let mut query_map = HashMap::<DidRange, DidRange>::new();
    for (key, value) in queries.unwrap_or_default() {
        let key_start = current_last_position;
        let key_end = key_start + key.len();
        let value_start = key_end + 1;
        let value_end = value_start + value.len();
        current_last_position = value_end + 1;
        query_map.insert(key_start..key_end, value_start..value_end);
    }

    let fragment_range = fragment.map(|fragment| {
        let fragment_end = fragment.len() + current_last_position;
        current_last_position..fragment_end
    });

    (path_range, query_map, fragment_range)
}

fn validate_result_not_empty(url_part: &UrlPart, did_ranges: &DidRanges) -> Result<(), ParseError> {
    if (url_part, did_ranges) == (&Default::default(), &Default::default()) {
        Err(ParseError::InvalidInput("Invalid input"))
    } else {
        Ok(())
    }
}

// did-url = did path-abempty [ "?" query ] [ "#" fragment ]
pub fn parse_did_url(did_url: String) -> Result<DidUrl, ParseError> {
    if did_url.is_empty() {
        return Err(ParseError::InvalidInput("Empty input"));
    }

    let (remaining, did_ranges) = parse_did_ranges_with_empty_allowed(&did_url)?;

    let (_, url_part) = all_consuming(parse_url_part)(remaining)?;

    validate_result_not_empty(&url_part, &did_ranges)?;

    let (method, namespace, id) = did_ranges;
    let (path, queries, fragment) = to_did_url_ranges(id.clone(), url_part);

    Ok(DidUrl {
        did_url,
        did: id.clone().map(|range| 0..range.end),
        method,
        namespace,
        id,
        path,
        fragment,
        queries,
    })
}
