pub(crate) mod error;

use chumsky::{
    combinator::Repeated,
    error::{Rich, RichReason},
    extra,
    prelude::{just, none_of, one_of},
    primitive::OneOf,
    span::SimpleSpan,
    text, IterParser, Parser as ChumskyParser,
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

/// An internal trait for implementing chumsky parsers.
/// We would then simply call parser().parse(input) on it and get the structure.
pub(crate) trait InternalParser<'src, I>: Sized {
    fn parser() -> impl ChumskyParser<'src, &'src str, I, extra::Err<Rich<'src, char>>>;
}

/// A trait that should be implemented on all VMF block types.
///
/// example: `let version_info = VersionInfo::parse(input);`
///
// We don't expect anyone to implement VMF parsing outside of this crate,
// so we have the Parser require InternalParser.
#[allow(private_bounds)]
pub trait Parser<'src, I>: InternalParser<'src, I> {
    fn parse(src: &'src str) -> Result<I, Vec<RichReason<'src, char>>> {
        let result = <Self as InternalParser<'src, I>>::parser().parse(src);
        if result.has_errors() {
            Err(result.errors().map(|e| e.reason().clone()).collect())
        } else {
            Ok(result.unwrap())
        }
    }
}

/// Parse a number `T` from either `123` or `"123"`.
pub(crate) fn number<'a, T>() -> impl ChumskyParser<'a, &'a str, T, extra::Err<Rich<'a, char>>>
where
    T: std::str::FromStr,
{
    let bare = text::int(10).try_map(|s: &str, span| {
        s.parse::<T>()
            .map_err(|_| Rich::custom(span, "integer out of range"))
    });

    let quoted = just('"').ignore_then(bare).then_ignore(just('"'));

    quoted.or(bare)
}

/// Parse a boolean literal: `true` or `false`.
pub(crate) fn boolean<'a>() -> impl ChumskyParser<'a, &'a str, bool, extra::Err<Rich<'a, char>>> {
    just("true").to(true).or(just("false").to(false))
}

/// Parses a white space (or many).
pub(crate) fn whitespace<'src>(
) -> impl ChumskyParser<'src, &'src str, (), extra::Err<Rich<'src, char>>> {
    one_of(" \t\n\r").repeated().ignored()
}

/// Parses any string, that is surrounded by quotes.
pub(crate) fn any_quoted_string<'src>(
) -> impl ChumskyParser<'src, &'src str, String, extra::Err<Rich<'src, char>>> {
    just('"')
        .ignore_then(none_of('"').repeated().collect::<String>())
        .then_ignore(just('"'))
}

/// Parses an exact string `input`, that is surrounded by quotes.
/// This is usefull when searching for strings, or whne looking up a key-value pair.
pub(crate) fn quoted_string(
    input: &str,
) -> impl ChumskyParser<&str, String, extra::Err<Rich<'_, char>>> {
    just('"')
        .ignore_then(just(input))
        .then_ignore(just('"'))
        .map(|value| value.to_string())
}

/// Takes a `key` string value, and tries to get a value.
/// The format of this is: "key" "string".
pub(crate) fn key_value(key: &str) -> impl ChumskyParser<&str, String, extra::Err<Rich<'_, char>>> {
    quoted_string(key).padded().ignore_then(any_quoted_string())
}

/// Takes a `key` string value, and tries to get a number value.
/// The format of this is: "key" "10"
pub(crate) fn key_value_numeric<T>(
    key: &str,
) -> impl ChumskyParser<&str, T, extra::Err<Rich<'_, char>>>
where
    T: std::str::FromStr,
{
    quoted_string(key)
        .padded()
        .ignore_then(number::<T>().padded())
}

/// Starts a parser on VMF blocks. VMF block usually starts with a key, then new line and open
/// bracket.
///
/// example:
/// versioninfo
/// {
pub(crate) fn open_block(block: &str) -> impl ChumskyParser<&str, (), extra::Err<Rich<'_, char>>> {
    just(block)
        .ignore_then(whitespace())
        .ignore_then(just('{'))
        .ignored()
}

/// Closes a previously [`open_block`]. It just ignores the whitespace and the closing bracket.
pub(crate) fn close_block<'src>(
) -> impl ChumskyParser<'src, &'src str, (), extra::Err<Rich<'src, char>>> {
    whitespace().ignore_then(just('}')).ignored()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chumsky::Parser;

    #[test]
    fn test_number() {
        let r = number::<u32>().parse("0");
        assert!(!r.has_errors());
        assert_eq!(r.unwrap(), 0);

        let r = number::<u32>().parse("12345");
        assert!(!r.has_errors());
        assert_eq!(r.unwrap(), 12345);

        // out of range
        let r = number::<u8>().parse("300");
        assert!(r.has_errors());

        // non‐digit
        let r = number::<u32>().parse("abc");
        assert!(r.has_errors());
    }

    #[test]
    fn test_boolean() {
        let r = boolean().parse("true");
        assert!(!r.has_errors());
        assert!(r.unwrap());

        let r = boolean().parse("false");
        assert!(!r.has_errors());
        assert!(!r.unwrap());

        let r = boolean().parse("yes");
        assert!(r.has_errors());
    }

    #[test]
    fn test_whitespace() {
        assert!(!whitespace().parse("    ").has_errors());
        assert!(!whitespace().parse("\t\n\r").has_errors());
        assert!(!whitespace().parse("").has_errors());

        assert!(whitespace().parse("x").has_errors());
    }

    #[test]
    fn test_any_quoted_string() {
        let r = any_quoted_string().parse("\"hello\"");
        assert!(!r.has_errors());
        assert_eq!(r.unwrap(), "hello".to_string());

        let r = any_quoted_string().parse("\"\"");
        assert!(!r.has_errors());
        assert_eq!(r.unwrap(), "".to_string());

        // missing closing quote
        assert!(any_quoted_string().parse("\"abc").has_errors());
    }

    #[test]
    fn test_quoted_string() {
        let r = quoted_string("foo").parse("\"foo\"");
        assert!(!r.has_errors());
        assert_eq!(r.unwrap(), "foo".to_string());

        // wrong literal
        assert!(quoted_string("foo").parse("\"bar\"").has_errors());
    }

    #[test]
    fn test_key_value() {
        let r = key_value("key").parse("\"key\" \"value\"");
        assert!(!r.has_errors());
        assert_eq!(r.unwrap(), "value".to_string());

        // no space between
        let r = key_value("key").parse("\"key\"\"v\"");
        assert!(!r.has_errors());
        assert_eq!(r.unwrap(), "v".to_string());

        // wrong key
        assert!(key_value("key").parse("\"other\" \"value\"").has_errors());
    }

    #[test]
    fn test_key_value_numeric() {
        let r = key_value_numeric::<u32>("num").parse("\"num\" \"10\"");
        assert!(!r.has_errors());
        assert_eq!(r.unwrap(), 10);

        let r = key_value_numeric::<u32>("num").parse("\"num\" \"20\"");
        assert!(!r.has_errors());
        assert_eq!(r.unwrap(), 20);

        // wrong key
        assert!(key_value_numeric::<u32>("num")
            .parse("\"x\" \"5\"")
            .has_errors());

        // non‐numeric
        assert!(key_value_numeric::<u32>("num")
            .parse("\"num\" \"abc\"")
            .has_errors());
    }

    #[test]
    fn test_open_close_block() {
        // open_block
        let r = open_block("blk").parse("blk{");
        assert!(!r.has_errors());

        let r = open_block("blk").parse("blk {");
        assert!(!r.has_errors());

        // close_block
        let r = close_block().parse("}");
        assert!(!r.has_errors());

        let r = close_block().parse(" }");
        assert!(!r.has_errors());

        // errors
        assert!(open_block("x").parse("y{").has_errors());
        assert!(close_block().parse("]").has_errors());
    }
}
