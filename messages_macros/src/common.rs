use std::fmt::Display;

use proc_macro2::Span;
use syn::{spanned::Spanned, Error, Result as SynResult};

pub fn next_or_panic<I, T>(iter: &mut I, msg: &str) -> T
where
    I: Iterator<Item = T>,
{
    iter.next().expect(msg)
}

pub fn next_or_err<I, T, S>(iter: &mut I, span: Span, msg: S) -> SynResult<T>
where
    I: Iterator<Item = T>,
    S: Display,
{
    iter.next().ok_or_else(|| Error::new(span, msg))
}

pub fn end_or_err<I, T, S>(iter: &mut I, msg: S) -> SynResult<()>
where
    I: Iterator<Item = T>,
    T: Spanned,
    S: Display,
{
    match iter.next() {
        Some(v) => Err(Error::new(v.span(), msg)),
        None => Ok(()),
    }
}
