use nom::{
    bytes::complete::{tag, take_till},
    character::complete::char,
    combinator::{fail, opt},
    sequence::{delimited, tuple},
    IResult,
};

use super::DidPart;

pub(super) fn parse_did_web(input: &str) -> IResult<&str, DidPart> {
    fn did_web_method(input: &str) -> IResult<&str, &str> {
        delimited(char(':'), tag("web"), char(':'))(input)
    }

    tuple((
        tag("did"),
        did_web_method,
        opt(fail::<_, &str, _>),
        take_till(|c| "?/#".contains(c)),
    ))(input)
}
