pub(crate) mod error;
pub(crate) mod lexer;

use chumsky::{
    combinator::Repeated,
    error::{Rich, RichReason},
    extra,
    input::{Stream, ValueInput},
    prelude::*,
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

/// A shorthand alias for any input source that produces our `lexer::Token` values
/// along with `SimpleSpan` offsets, and supports value-based parsing (cloning tokens).
///
/// This trait is automatically implemented for any `I` that satisfies:
/// ```ignore
/// I: ValueInput<'src, Token = lexer::Token<'src>, Span = SimpleSpan>
/// ```
///
/// This is a helper trait for a Chumsky parser over tokens, so we dont have
/// to spell out the bound everywhere.
pub(crate) trait TokenSource<'src>:
    ValueInput<'src, Token = lexer::Token<'src>, Span = SimpleSpan>
{
}

/// Seals the `TokenSource` automatically for any `I` that is a ValueInput of the right types
impl<'src, I> TokenSource<'src> for I where
    I: ValueInput<'src, Token = lexer::Token<'src>, Span = SimpleSpan>
{
}

pub(crate) type TokenError<'src> = extra::Err<Rich<'src, lexer::Token<'src>>>;

/// A private trait that every VMF‐block parser must implement.
///
/// Each implementer provides a `parser()` method that builds a Chumsky parser
/// from any `TokenSource`.  This parser:
/// - Consumes tokens of type `lexer::Token<'src>` from the input `I`.
/// - Produces an instance of `Self` on success.
/// - Yields errors of type `TokenError<'src>` on failure.
///
/// By making it generic over `I: TokenSource<'src>`, we can drive the parser
/// off either a pre-collected slice of tokens (`&[Token<'_, _>]`) or a streaming
/// iterator wrapped with `Stream::from_iter(...)`.
pub(crate) trait InternalParser<'src>: Sized {
    fn parser<I>() -> impl ChumskyParser<'src, I, Self, TokenError<'src>>
    where
        I: TokenSource<'src>;
}

/// A trait that should be implemented on all VMF block types.
///
/// example: `let version_info = VersionInfo::parse(input);`
///
// We don't expect anyone to implement VMF parsing outside of this crate,
// so we have the Parser require InternalParser.
#[allow(private_bounds)]
pub trait Parser<'src>: InternalParser<'src> {
    fn parse(
        src: impl TokenSource<'src>,
    ) -> Result<Self, Vec<RichReason<'src, lexer::Token<'src>>>> {
        let result = <Self as InternalParser<'src>>::parser::<_>().parse(src);
        if result.has_errors() {
            Err(result.errors().map(|e| e.reason().clone()).collect())
        } else {
            Ok(result.unwrap())
        }
    }
}

/// Parse a number from `T`.
pub(crate) fn number<'a, T, I>() -> impl ChumskyParser<'a, I, T, TokenError<'a>>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
    I: TokenSource<'a>,
{
    select! { lexer::Token::Number(s) => s }.try_map(|s: &str, span| {
        s.parse::<T>()
            .map_err(|_| Rich::custom(span, "integer out of range"))
    })
}

/// Parse a boolean literal: `true` or `false`.
pub(crate) fn boolean<'a, I>() -> impl ChumskyParser<'a, I, bool, TokenError<'a>>
where
    I: TokenSource<'a>,
{
    quoted_string("true")
        .or(quoted_string("false"))
        .map(|v| match v.as_ref() {
            "true" => true,
            "false" => false,
            _ => unreachable!(),
        })
}

/// Parses a white space (or many).
pub(crate) fn whitespace<'src, I>() -> impl ChumskyParser<'src, I, (), TokenError<'src>>
where
    I: TokenSource<'src>,
{
    select! {
        lexer::Token::Whitespace => ()
    }
}

/// Parses any string, that is surrounded by quotes.
pub(crate) fn any_quoted_string<'src, I>() -> impl ChumskyParser<'src, I, String, TokenError<'src>>
where
    I: TokenSource<'src>,
{
    just(lexer::Token::Quote)
        .ignore_then(select! { lexer::Token::Ident(s) => s.to_string() })
        .then_ignore(just(lexer::Token::Quote))
}

/// Parses an exact string `input`, that is surrounded by quotes.
/// This is usefull when searching for strings, or whne looking up a key-value pair.
pub(crate) fn quoted_string<'src, I>(
    input: &'src str,
) -> impl ChumskyParser<'src, I, String, TokenError<'src>>
where
    I: TokenSource<'src>,
{
    just(lexer::Token::Quote)
        .ignore_then(just(lexer::Token::Ident(input)))
        .then_ignore(just(lexer::Token::Quote))
        .map(|tok| match tok {
            lexer::Token::Ident(s) => s.to_string(),
            _ => unreachable!("only Token::Ident can reach this point"),
        })
}

/// Takes a `key` string value, and tries to get a value.
/// The format of this is: "key" "string".
pub(crate) fn key_value<'src, I>(
    key: &'src str,
) -> impl ChumskyParser<'src, I, String, TokenError<'src>>
where
    I: TokenSource<'src>,
{
    quoted_string(key).ignore_then(any_quoted_string())
}

/// Takes a `key` string value, and tries to get a number value.
/// The format of this is: "key" "10"
pub(crate) fn key_value_numeric<'src, T, I>(
    key: &'src str,
) -> impl ChumskyParser<'src, I, T, TokenError<'src>>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
    I: TokenSource<'src>,
{
    quoted_string(key).ignore_then(number::<T, I>())
}

/// Starts a parser on VMF blocks. VMF block usually starts with a key, then new line and open
/// bracket.
///
/// example:
/// versioninfo
/// {
pub(crate) fn open_block<'src, I>(
    block: &'src str,
) -> impl ChumskyParser<'src, I, (), TokenError<'src>>
where
    I: TokenSource<'src>,
{
    just(lexer::Token::Ident(block))
        .ignore_then(whitespace())
        .ignore_then(just(lexer::Token::LBracket))
        .ignored()
}

/// Closes a previously [`open_block`]. It just ignores the whitespace and the closing bracket.
pub(crate) fn close_block<'src, I>() -> impl ChumskyParser<'src, I, (), TokenError<'src>>
where
    I: TokenSource<'src>,
{
    whitespace()
        .ignore_then(just(lexer::Token::RBracket))
        .ignored()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chumsky::prelude::SimpleSpan;
    use chumsky::{input::Stream, ParseResult, Parser};
    use logos::Logos as _;

    fn lex(input: &str) -> Vec<lexer::Token> {
        lexer::Token::lexer(input).map(|tok| tok.unwrap()).collect()
    }

    #[test]
    fn test_number() {
        let input = lex("12345");
        let stream = Stream::from_iter(input);

        let result = number::<u32, _>().parse(stream);
        assert!(!result.has_errors());
        assert_eq!(result.unwrap(), 12345);
    }

    #[test]
    fn test_boolean() {
        let input = lex(r#""true""#);
        let stream = Stream::from_iter(input);

        let result = boolean::<_>().parse(stream);
        assert!(!result.has_errors());
        assert!(result.unwrap());
    }

    #[test]
    fn test_key_value_numeric() {
        let tokens = lex(r#""num" "42""#);
        let stream = Stream::from_iter(tokens);
        let mut result = key_value_numeric::<u32, _>("num").parse(stream);
        assert!(!result.has_errors());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_open_close_block() {
        let tokens = lex("blk{");
        let stream = Stream::from_iter(tokens);
        let mut r1 = open_block("blk").parse(stream);
        assert!(!r1.has_errors());

        let tokens = lex("}");
        let stream = Stream::from_iter(tokens);
        let mut r2 = close_block().parse(stream);
        assert!(!r2.has_errors());
    }
}
