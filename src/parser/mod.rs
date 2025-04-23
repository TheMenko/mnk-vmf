pub(crate) mod error;

use chumsky::{
    combinator::Repeated,
    error::Rich,
    prelude::{just, none_of, one_of},
    primitive::OneOf,
    IterParser, Parser,
};
use error::VMFParserError;

#[derive(Debug, PartialEq)]
pub enum VmfKeyValue {
    String(String),
    Boolean(bool),
    Int(i64),
    Float(f64),
    Array(Vec<VmfKeyValue>),
}

pub trait VMFParser<I>: Sized {
    fn parser<'src>() -> impl Parser<'src, &'src str, I>;
}

pub fn whitespace<'src>() -> impl Parser<'src, &'src str, ()> {
    one_of(" \t\n\r").repeated().ignored()
}

pub fn any_quoted_string<'src>() -> impl Parser<'src, &'src str, String> {
    just('"')
        .ignore_then(none_of('"').repeated().collect::<String>())
        .then_ignore(just('"'))
}

pub fn quoted_string<'src>(input: &'src str) -> impl Parser<'src, &'src str, String> {
    just('"')
        .ignore_then(just(input))
        .then_ignore(just('"'))
        .map(|value| value.to_string())
}

pub fn key_value<'src>(key: &'src str) -> impl Parser<'src, &'src str, String> {
    quoted_string(key)
        .ignored()
        .then_ignore(whitespace())
        .ignore_then(any_quoted_string())
}
