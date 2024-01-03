use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::char,
    combinator::{all_consuming, cut, map, recognize, success, value},
    multi::{many0, separated_list0, separated_list1},
    sequence::{preceded, separated_pair},
    IResult,
};

use crate::{
    did::utils::{parse_qualified_did, parse_unqualified_sovrin_did, to_did_ranges, to_id_range},
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

// pchar = unreserved / pct-encoded / sub-delims / ":" / "@"
fn is_pchar(c: char) -> bool {
    is_unreserved(c) || is_sub_delims(c) || ":@".contains(c)
}

// segment = *pchar
fn segment(input: &str) -> IResult<&str, &str> {
    take_while1(is_pchar)(input)
}

// path-abempty = *( "/" segment )
fn path_abempty(input: &str) -> IResult<&str, &str> {
    recognize(many0(preceded(tag("/"), segment)))(input)
}

// fragment = *( pchar / "/" / "?" )
fn fragment(input: &str) -> IResult<&str, &str> {
    fn is_fragment_char(c: char) -> bool {
        is_pchar(c) || "/?".contains(c)
    }

    take_while1(is_fragment_char)(input)
}

// query = *( pchar / "/" / "?" )
fn query_key_value_pair(input: &str) -> IResult<&str, (&str, &str)> {
    fn is_query_char(c: char) -> bool {
        is_pchar(c) || "/?".contains(c)
    }

    let (remaining, (key, value)) = cut(separated_pair(
        take_while1(|c| !"=&#;".contains(c)),
        char('='),
        alt((take_while1(|c| !"&#;".contains(c)), success(""))),
    ))(input)?;

    cut(all_consuming(take_while1(is_query_char)))(key)?;
    if !value.is_empty() {
        cut(all_consuming(take_while1(is_query_char)))(value)?;
    }

    Ok((remaining, (key, value)))
}

fn query_parser(input: &str) -> IResult<&str, Vec<(&str, &str)>> {
    separated_list0(char('&'), query_key_value_pair)(input)
}

fn parse_did_part(
    input: &str,
) -> IResult<&str, (Option<DidRange>, Option<DidRange>, Option<DidRange>)> {
    alt((
        map(parse_qualified_did, to_did_ranges),
        map(parse_unqualified_sovrin_did, to_id_range),
        success(Default::default()), // Relative DID URL
    ))(input)
}

// did-url-remaining = path-abempty [ "?" query ] [ "#" fragment ]
fn parse_url_part(input: &str) -> IResult<&str, (&str, Vec<(&str, &str)>, Vec<&str>)> {
    let (input, path) = path_abempty(input)?;
    let (input, queries) = alt((preceded(tag("?"), query_parser), value(vec![], tag(""))))(input)?;
    let (input, fragments) = alt((
        preceded(tag("#"), separated_list1(tag("#"), fragment)),
        value(vec![], tag("")),
    ))(input)?;
    Ok((input, (path, queries, fragments)))
}

fn to_did_url_ranges(
    id_range: Option<DidRange>,
    (path, queries, fragments): (&str, Vec<(&str, &str)>, Vec<&str>),
) -> (
    Option<DidRange>,
    HashMap<DidRange, DidRange>,
    Option<DidRange>,
) {
    let id_end = id_range.clone().unwrap_or_default().end;
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
    for (key, value) in queries {
        let key_start = current_last_position;
        let key_end = key_start + key.len();
        let value_start = key_end + 1;
        let value_end = value_start + value.len();
        current_last_position = value_end + 1;
        query_map.insert(key_start..key_end, value_start..value_end);
    }

    let fragment_range = if fragments.is_empty() {
        None
    } else {
        // TODO: Potential bug, multiple fragments are separated by #
        let fragment_end = fragments.iter().map(|f| f.len()).sum::<usize>() + current_last_position;
        Some(current_last_position..fragment_end)
    };

    (path_range, query_map, fragment_range)
}

fn check_result(
    parsed_remaining: &(&str, Vec<(&str, &str)>, Vec<&str>),
    did_ranges: &(Option<DidRange>, Option<DidRange>, Option<DidRange>),
) -> Result<(), ParseError> {
    if parsed_remaining == &Default::default() && did_ranges == &Default::default() {
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

    let (remaining, did_ranges) =
        parse_did_part(&did_url).map_err(|err| ParseError::ParserError(err.to_owned().into()))?;

    let (_, url_ranges) =
        parse_url_part(remaining).map_err(|err| ParseError::ParserError(err.to_owned().into()))?;

    check_result(&url_ranges, &did_ranges)?;

    let (method, namespace, id) = did_ranges;
    let (path, queries, fragment) = to_did_url_ranges(id.clone(), url_ranges);

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
