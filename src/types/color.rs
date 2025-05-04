use crate::{
    parser::{
        any_quoted_string, close_block, lexer, number, open_block, quoted_string, InternalParser,
        TokenError, TokenSource,
    },
    Parser,
};

use chumsky::{error::Rich, extra, prelude::just, Parser as ChumskyParser};

/// Represents an RGB color with three components
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Public parser trait implementation that allows [`Color`] to use ::parse(input) call.
impl Parser<'_> for Color {}

/// A [`InternalParser`] implementation for [`Color`].
/// Every key-value pair needs to be in order, like in the example bellow.
///
/// usage: `let color = Color::parser().parse();`.
///
/// The format that is being parsed here is:
/// "color" "10 100 250"
impl<'src> InternalParser<'src> for Color {
    fn parser<I>() -> impl ChumskyParser<'src, I, Self, TokenError<'src>>
    where
        I: TokenSource<'src>,
    {
        quoted_string("color")
            .ignore_then(any_quoted_string())
            .try_map(|s: String, span| {
                let mut parts = s.split_whitespace().map(str::parse::<u8>);
                let (r, g, b) = match (parts.next(), parts.next(), parts.next()) {
                    (Some(Ok(r)), Some(Ok(g)), Some(Ok(b))) => (r, g, b),
                    _ => return Err(Rich::custom(span, "invalid color components")),
                };

                if parts.next().is_some() {
                    return Err(Rich::custom(span, "too many color components"));
                }

                Ok(Color { r, g, b })
            })
    }
}

#[cfg(test)]
mod tests {
    use chumsky::input::Stream;
    use logos::Logos as _;

    use crate::util::lex;

    use super::*;

    #[test]
    fn test_color_valid() {
        let input = lex(r#""color" "10 100 250""#);
        let result = Color::parse(input).expect("Color should parse");
        assert_eq!(
            result,
            Color {
                r: 10,
                g: 100,
                b: 250
            }
        );
    }

    #[test]
    fn test_color_whitespace_variations() {
        let cases = [
            r#""color" "0 0 0""#,
            r#"   "color"   "255 255 255"  "#,
            "\"color\"\t\"1\t2\t3\"",
        ];
        for &input in &cases {
            let stream = lex(input);
            let res = Color::parse(stream).expect("Color should parse");
        }
    }

    #[test]
    fn test_color_invalid_key() {
        assert!(Color::parse(lex(r#""colour" "1 2 3""#)).is_err());
    }

    #[test]
    fn test_color_invalid_value_formats() {
        assert!(Color::parse(lex(r#""color" "10 20""#)).is_err());
        assert!(Color::parse(lex(r#""color" "10 20 30 40""#)).is_err());
        assert!(Color::parse(lex(r#""color" "10 twenty 30""#)).is_err());
    }

    #[test]
    fn test_color_out_of_range() {
        assert!(Color::parse(lex(r#""color" "256 0 0""#)).is_err());
        assert!(Color::parse(lex(r#""color" "0 300 0""#)).is_err());
        assert!(Color::parse(lex(r#""color" "0 0 999""#)).is_err());
    }
}
