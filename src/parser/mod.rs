pub(crate) mod error;
pub(crate) mod lexer;
pub mod util;

use chumsky::{
    Parser as ChumskyParser,
    error::{Rich, RichReason},
    extra,
    input::ValueInput,
    number::{format::STANDARD, number as chumsky_number},
    prelude::*,
    span::SimpleSpan,
};
use lexical_core::FromLexical;

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

/// Parse a numeric literal `T` from a `Token::Number(&str)`.
pub fn number<'src, I, T>() -> impl ChumskyParser<'src, I, T, TokenError<'src>>
where
    I: TokenSource<'src>,
    T: FromLexical + 'src,
{
    select! { lexer::Token::Number(s) => s }.try_map(|s, span| {
        let parsed = chumsky_number::<STANDARD, &str, T, extra::Default>().parse(s);

        if parsed.has_errors() {
            Err(Rich::custom(
                span,
                format!("invalid numeric literal: {}", s),
            ))
        } else {
            Ok(parsed.into_result().unwrap())
        }
    })
}

/// Parse a boolean literal: `true` or `false`.
pub(crate) fn boolean<'a, I>() -> impl ChumskyParser<'a, I, bool, TokenError<'a>>
where
    I: TokenSource<'a>,
{
    quoted(number::<_, u8>())
        .or(quoted(number::<_, u8>()))
        .map(|v| match v {
            1 => true,
            0 => false,
            _ => unreachable!(),
        })
}

/// Takes a parser and returns a new parser that matches the input surrounded by quotes.
pub(crate) fn quoted<'src, I, O>(
    inner: impl ChumskyParser<'src, I, O, TokenError<'src>>,
) -> impl ChumskyParser<'src, I, O, TokenError<'src>>
where
    I: TokenSource<'src>,
{
    just(lexer::Token::Quote)
        .ignore_then(inner)
        .then_ignore(just(lexer::Token::Quote))
}

fn word<'src, I>() -> impl ChumskyParser<'src, I, &'src str, TokenError<'src>>
where
    I: TokenSource<'src>,
{
    select! { lexer::Token::Text(s) => s }
}

/// Parses any string, that is surrounded by quotes.
pub(crate) fn any_quoted_string<'src, I>()
-> impl ChumskyParser<'src, I, &'src str, TokenError<'src>>
where
    I: TokenSource<'src>,
{
    quoted(word())
}

/// Parses an exact string `input`, that is surrounded by quotes.
/// This is usefull when searching for strings, or whne looking up a key-value pair.
pub(crate) fn quoted_string<'src, I>(
    input: &'src str,
) -> impl ChumskyParser<'src, I, &'src str, TokenError<'src>>
where
    I: TokenSource<'src>,
{
    quoted(select! { lexer::Token::Text(s) if s == input => s })
}

/// Takes a `key` string value, and tries to get a value.
/// The format of this is: "key" "string".
pub(crate) fn key_value<'src, I>(
    key: &'src str,
) -> impl ChumskyParser<'src, I, &'src str, TokenError<'src>>
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
    T: std::str::FromStr + FromLexical,
    T::Err: std::fmt::Debug,
    I: TokenSource<'src>,
{
    quoted_string(key).ignore_then(quoted(number::<I, T>()))
}

/// Takes a `key` string value, and tries to get a boolean value.
/// The format of this is: "key" "false"
pub(crate) fn key_value_boolean<'src, I>(
    key: &'src str,
) -> impl ChumskyParser<'src, I, bool, TokenError<'src>>
where
    I: TokenSource<'src>,
{
    quoted_string(key).ignore_then(quoted(boolean()))
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
    just(lexer::Token::Text(block))
        .ignore_then(just(lexer::Token::LBrace))
        .ignored()
}

/// Closes a previously [`open_block`]. It just ignores the whitespace and the closing bracket.
pub(crate) fn close_block<'src, I>() -> impl ChumskyParser<'src, I, (), TokenError<'src>>
where
    I: TokenSource<'src>,
{
    just(lexer::Token::RBrace).ignored()
}

/// Parses and skips any unknown/unrecognized block.
/// It matches any identifier followed by a block, and recursively skips nested blocks.
pub(crate) fn skip_unknown_block<'src, I>() -> impl ChumskyParser<'src, I, (), TokenError<'src>>
where
    I: TokenSource<'src>,
{
    recursive(|skip_block| {
        any()
            .filter(|tok| matches!(tok, lexer::Token::Text(_)))
            .ignore_then(just(lexer::Token::LBracket))
            .ignore_then(
                none_of([lexer::Token::LBracket, lexer::Token::RBracket])
                    .ignored()
                    .or(skip_block)
                    .repeated(),
            )
            .then_ignore(just(lexer::Token::RBracket))
            .ignored()
    })
}

#[cfg(test)]
mod tests {
    use crate::util::lex;

    use super::*;
    use chumsky::Parser;

    #[test]
    fn test_number() {
        let stream = lex("\"12345\"");

        let result = quoted(number::<_, u32>()).parse(stream);
        for e in result.errors() {
            println!("error: {:?}", e.reason());
        }
        assert!(!result.has_errors());
        assert_eq!(result.unwrap(), 12345);
    }

    #[test]
    fn test_boolean() {
        let stream = lex(r#""1""#);

        let result = boolean::<_>().parse(stream);
        assert!(!result.has_errors());
        assert!(result.unwrap());
    }

    #[test]
    fn test_key_value_numeric() {
        let stream = lex(r#""num" "42""#);
        let result = key_value_numeric::<u32, _>("num").parse(stream);
        for e in result.errors() {
            println!("error: {:?}", e.reason());
        }
        assert!(!result.has_errors());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_open_close_block() {
        let stream = lex("blk {");
        let r1 = open_block("blk").parse(stream);
        for e in r1.errors() {
            println!("error: {:?}", e.reason());
        }
        assert!(!r1.has_errors());

        let stream = lex("}");
        let r2 = close_block().parse(stream);
        for e in r1.errors() {
            println!("error: {:?}", e.reason());
        }
        assert!(!r2.has_errors());
    }
}
