use crate::{
    parser::{close_block, number, open_block, quoted_string, InternalParser},
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
impl Parser<'_, Color> for Color {}

/// A [`InternalParser`] implementation for [`Color`].
/// Every key-value pair needs to be in order, like in the example bellow.
///
/// usage: `let color = Color::parser().parse();`.
///
/// The format that is being parsed here is:
/// "color" "10 100 250"
impl<'src> InternalParser<'src, Color> for Color {
    fn parser() -> impl ChumskyParser<'src, &'src str, Self, extra::Err<Rich<'src, char>>> {
        quoted_string("color")
            .padded()
            .ignore_then(just("\""))
            .ignore_then(
                number::<u8>()
                    .padded()
                    .then(number::<u8>().padded())
                    .then(number::<u8>().padded()),
            )
            .then_ignore(just("\"").padded())
            .map(|((r, g), b)| Color { r, g, b })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_valid() {
        let input = r#""color" "10 100 250""#;
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
            let res = Color::parse(input).expect("Color should parse");
        }
    }

    #[test]
    fn test_color_invalid_key() {
        assert!(Color::parse(r#""colour" "1 2 3""#).is_err());
    }

    #[test]
    fn test_color_invalid_value_formats() {
        assert!(Color::parse(r#""color" "10 20""#).is_err());
        assert!(Color::parse(r#""color" "10 20 30 40""#).is_err());
        assert!(Color::parse(r#""color" "10 twenty 30""#).is_err());
    }

    #[test]
    fn test_color_out_of_range() {
        assert!(Color::parse(r#""color" "256 0 0""#).is_err());
        assert!(Color::parse(r#""color" "0 300 0""#).is_err());
        assert!(Color::parse(r#""color" "0 0 999""#).is_err());
    }
}
