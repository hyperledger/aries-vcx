use nom::{
    bytes::complete::{tag, take_while1},
    character::complete::char,
    combinator::{fail, opt},
    sequence::{delimited, tuple},
    IResult,
};

use super::DidPart;

// mb-value       := z[a-km-zA-HJ-NP-Z1-9]+
fn is_multibase_value(c: char) -> bool {
    "abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ123456789".contains(c)
}

// did-key-format := did:key:<mb-value>
pub(super) fn parse_did_key(input: &str) -> IResult<&str, DidPart> {
    fn did_key_method(input: &str) -> IResult<&str, &str> {
        delimited(char(':'), tag("key"), char(':'))(input)
    }

    tuple((
        tag("did"),
        did_key_method,
        opt(fail::<_, &str, _>),
        take_while1(is_multibase_value),
    ))(input)
}
