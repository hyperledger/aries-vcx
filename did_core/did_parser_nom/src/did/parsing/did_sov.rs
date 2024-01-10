use nom::{
    bytes::complete::tag,
    character::complete::{char, one_of},
    combinator::{cut, opt, recognize},
    multi::many_m_n,
    sequence::{delimited, tuple},
    IResult,
};

use crate::did::parsing::did_core::namespace;

use super::DidPart;

fn base58char(input: &str) -> IResult<&str, &str> {
    recognize(one_of(
        "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz",
    ))(input)
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
    tuple((
        tag("did"),
        did_sov_method,
        opt(namespace),
        cut(parse_unqualified_sovrin_did),
    ))(input)
}
