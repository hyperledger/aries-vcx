use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, one_of},
    combinator::{all_consuming, cut, opt, recognize},
    multi::{many1, many_m_n},
    sequence::{delimited, tuple},
    IResult,
};

use super::{DidPart, BASE58CHARS};
use crate::did::parsing::did_core::idchar;

fn base58char(input: &str) -> IResult<&str, &str> {
    recognize(one_of(BASE58CHARS))(input)
}

// namespace =  *idchar ":"
fn did_sov_namespace(input: &str) -> IResult<&str, &str> {
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

// idstring = 21*22(base58char)
pub(super) fn parse_unqualified_sovrin_did(input: &str) -> IResult<&str, &str> {
    recognize(many_m_n(21, 22, base58char))(input)
}

// The specification seems to contradict practice?
// sovrin-did = "did:sov:" idstring *(":" subnamespace)
// subnamespace = ALPHA *(ALPHA / DIGIT / "_" / "-")
pub(super) fn parse_qualified_sovrin_did(input: &str) -> IResult<&str, DidPart> {
    fn did_sov_method(input: &str) -> IResult<&str, &str> {
        delimited(char(':'), tag("sov"), char(':'))(input)
    }
    let (input_left, (prefix, method, namespace, id)) = tuple((
        tag("did"),
        did_sov_method,
        opt(did_sov_namespace),
        cut(parse_unqualified_sovrin_did),
    ))(input)?;

    Ok((input_left, (prefix, method, namespace, id)))
}
