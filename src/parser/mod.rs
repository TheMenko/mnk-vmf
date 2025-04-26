pub(crate) mod error;

use chumsky::{
    combinator::Repeated,
    error::Rich,
    extra,
    prelude::{just, none_of, one_of},
    primitive::OneOf,
    span::SimpleSpan,
    text, IterParser, Parser,
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

/// A trait that should be implemented on all VMF block types.
/// We woulc then simply call parser().parse() on it and get the structure.
///
/// example: `let version_info = VersionInfo::parser().parse();`
pub(crate) trait VMFParser<I>: Sized {
    fn parser<'src>() -> impl Parser<'src, &'src str, I, extra::Err<Rich<'src, char>>>;
}

/// Parse any number that can be parsed from a string.
fn number<'a, T>() -> impl Parser<'a, &'a str, T, extra::Err<Rich<'a, char>>>
where
    T: std::str::FromStr,
{
    text::int(10).try_map(|s: &str, span| {
        s.parse::<T>()
            .map_err(|_| Rich::custom(span, "integer out of range"))
    })
}

/// Parse a boolean literal: `true` or `false`.
fn boolean<'a>() -> impl Parser<'a, &'a str, bool, extra::Err<Rich<'a, char>>> {
    just("true").to(true).or(just("false").to(false))
}

/// Parses a white space (or many).
pub(crate) fn whitespace<'src>() -> impl Parser<'src, &'src str, (), extra::Err<Rich<'src, char>>> {
    one_of(" \t\n\r").repeated().ignored()
}

/// Parses any string, that is surrounded by quotes.
pub(crate) fn any_quoted_string<'src>(
) -> impl Parser<'src, &'src str, String, extra::Err<Rich<'src, char>>> {
    just('"')
        .ignore_then(none_of('"').repeated().collect::<String>())
        .then_ignore(just('"'))
}

/// Parses an exact string `input`, that is surrounded by quotes.
/// This is usefull when searching for strings, or whne looking up a key-value pair.
pub(crate) fn quoted_string(input: &str) -> impl Parser<&str, String, extra::Err<Rich<'_, char>>> {
    just('"')
        .ignore_then(just(input))
        .then_ignore(just('"'))
        .map(|value| value.to_string())
}

/// Takes a `key` string value, and tries to get a value.
/// The format of this is: "key" "string".
pub(crate) fn key_value(key: &str) -> impl Parser<&str, String, extra::Err<Rich<'_, char>>> {
    quoted_string(key).padded().ignore_then(any_quoted_string())
}

/// Takes a `key` string value, and tries to get a number value.
/// The format of this is: "key" "10"
pub(crate) fn key_value_numeric<T>(key: &str) -> impl Parser<&str, T, extra::Err<Rich<'_, char>>>
where
    T: std::str::FromStr,
{
    quoted_string(key).padded().ignore_then(number::<T>())
}

/// Starts a parser on VMF blocks. VMF block usually starts with a key, then new line and open
/// bracket.
///
/// example:
/// versioninfo
/// {
pub(crate) fn open_block(block: &str) -> impl Parser<&str, (), extra::Err<Rich<'_, char>>> {
    just(block).padded().ignore_then(just('{')).ignored()
}

/// Closes a previously [`open_block`]. It just ignores the whitespace and the closing bracket.
pub(crate) fn close_block<'src>() -> impl Parser<'src, &'src str, (), extra::Err<Rich<'src, char>>>
{
    whitespace().ignore_then(just('}')).ignored()
}
