use chumsky::{error::Rich, IterParser, Parser as ChumskyParser};

use crate::{
    impl_block_properties_parser,
    parser::{
        any_quoted_string, close_block, key_value_boolean, open_block, quoted_string,
        InternalParser, TokenError, TokenSource,
    },
    types::point::{parse_point_from_numbers_str, Point3D},
    Parser,
};

/// Represents a cordon entity (tool used to block off parts of the map)
#[derive(Debug, Default)]
pub struct Cordon {
    pub mins: Point3D, // Minimum bounds of the cordon box
    pub maxs: Point3D, // Maximum bounds of the cordon box
    pub active: bool,  // Whether the cordon is active
}

/// Internal [`Cordon`] Properties to be used in a parser impl
#[derive(Debug, Clone)]
enum CordonProperty {
    Mins(Point3D),
    Maxs(Point3D),
    Active(bool),
}

/// Parse a Point3D with parentheses format "(x y z)"
fn parse_point_with_parens(value_str: &str) -> Result<Point3D, String> {
    let trimmed = value_str.trim();

    // Remove parentheses
    if !trimmed.starts_with('(') || !trimmed.ends_with(')') {
        return Err(format!(
            "point must be in format (x y z), got: {}",
            value_str
        ));
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    parse_point_from_numbers_str(inner)
}

/// Parses a key-value pair where the value is a Point3D with parentheses
fn key_value_point_with_parens<'src, I>(
    key: &'src str,
) -> impl ChumskyParser<'src, I, Point3D, TokenError<'src>>
where
    I: TokenSource<'src>,
{
    quoted_string(key)
        .ignore_then(any_quoted_string())
        .try_map(move |value_str, span| {
            parse_point_with_parens(value_str)
                .map_err(|err_msg| Rich::custom(span, format!("Invalid point: {}", err_msg)))
        })
}

/// Public parser trait implementation that allows [`Cordon`] to use ::parse(input) call.
impl Parser<'_> for Cordon {}

/// A [`InternalParser`] implementation for [`Cordon`].
///
/// usage: `let cordon = Cordon::parser().parse(input);`.
///
/// The format that is being parsed here is:
/// ```ignore
/// cordon
/// {
///     "mins" "(-1024 -1024 -1024)"
///     "maxs" "(1024 1024 1024)"
///     "active" "0"
/// }
/// ```
impl<'src> InternalParser<'src> for Cordon {
    fn parser<I>() -> impl ChumskyParser<'src, I, Self, TokenError<'src>>
    where
        I: TokenSource<'src>,
    {
        impl_block_properties_parser! {
            property_list: CordonProperty = {
                p_mins   = key_value_point_with_parens("mins")   => CordonProperty::Mins,
                p_maxs   = key_value_point_with_parens("maxs")   => CordonProperty::Maxs,
                p_active = key_value_boolean("active")           => CordonProperty::Active,
            }
        }

        open_block("cordon")
            .ignore_then(property_list.repeated().collect::<Vec<CordonProperty>>())
            .then_ignore(close_block())
            .map(|properties: Vec<CordonProperty>| {
                let mut cordon = Cordon::default();
                for prop in properties {
                    match prop {
                        CordonProperty::Mins(val) => cordon.mins = val,
                        CordonProperty::Maxs(val) => cordon.maxs = val,
                        CordonProperty::Active(val) => cordon.active = val,
                    }
                }
                cordon
            })
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::lex;

    #[test]
    fn test_cordon_complete_valid() {
        let input = r#"
        cordon
        {
            "mins" "(-1024 -1024 -1024)"
            "maxs" "(1024 1024 1024)"
            "active" "0"
        }
        "#;

        let stream = lex(input);
        let result = Cordon::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let cordon = result.unwrap();
        assert_eq!(cordon.mins.x, -1024.0);
        assert_eq!(cordon.mins.y, -1024.0);
        assert_eq!(cordon.mins.z, -1024.0);
        assert_eq!(cordon.maxs.x, 1024.0);
        assert_eq!(cordon.maxs.y, 1024.0);
        assert_eq!(cordon.maxs.z, 1024.0);
        assert_eq!(cordon.active, false);
    }

    #[test]
    fn test_cordon_active() {
        let input = r#"
        cordon
        {
            "mins" "(-500 -500 -100)"
            "maxs" "(500 500 100)"
            "active" "1"
        }
        "#;

        let stream = lex(input);
        let result = Cordon::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let cordon = result.unwrap();
        assert_eq!(cordon.mins.x, -500.0);
        assert_eq!(cordon.maxs.x, 500.0);
        assert_eq!(cordon.active, true);
    }

    #[test]
    fn test_cordon_properties_out_of_order() {
        let input = r#"
        cordon
        {
            "active" "1"
            "maxs" "(100 200 300)"
            "mins" "(-100 -200 -300)"
        }
        "#;

        let stream = lex(input);
        let result = Cordon::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let cordon = result.unwrap();
        assert_eq!(cordon.mins.x, -100.0);
        assert_eq!(cordon.mins.y, -200.0);
        assert_eq!(cordon.mins.z, -300.0);
        assert_eq!(cordon.maxs.x, 100.0);
        assert_eq!(cordon.maxs.y, 200.0);
        assert_eq!(cordon.maxs.z, 300.0);
        assert_eq!(cordon.active, true);
    }

    #[test]
    fn test_cordon_with_decimals() {
        let input = r#"
        cordon
        {
            "mins" "(-123.5 -456.75 -789.25)"
            "maxs" "(123.5 456.75 789.25)"
            "active" "0"
        }
        "#;

        let stream = lex(input);
        let result = Cordon::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let cordon = result.unwrap();
        assert_eq!(cordon.mins.x, -123.5);
        assert_eq!(cordon.mins.y, -456.75);
        assert_eq!(cordon.mins.z, -789.25);
        assert_eq!(cordon.maxs.x, 123.5);
        assert_eq!(cordon.maxs.y, 456.75);
        assert_eq!(cordon.maxs.z, 789.25);
    }

    #[test]
    fn test_cordon_empty_block() {
        let input = r#"
        cordon
        {
        }
        "#;

        let stream = lex(input);
        let result = Cordon::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let cordon = result.unwrap();
        let default = Cordon::default();
        assert_eq!(cordon.mins.x, default.mins.x);
        assert_eq!(cordon.active, default.active);
    }

    #[test]
    fn test_cordon_missing_parentheses_mins() {
        let input = r#"
        cordon
        {
            "mins" "-1024 -1024 -1024"
            "maxs" "(1024 1024 1024)"
            "active" "0"
        }
        "#;

        let stream = lex(input);
        let result = Cordon::parse(stream);
        assert!(
            result.is_err(),
            "Parser should fail on mins without parentheses"
        );
    }

    #[test]
    fn test_cordon_missing_parentheses_maxs() {
        let input = r#"
        cordon
        {
            "mins" "(-1024 -1024 -1024)"
            "maxs" "1024 1024 1024"
            "active" "0"
        }
        "#;

        let stream = lex(input);
        let result = Cordon::parse(stream);
        assert!(
            result.is_err(),
            "Parser should fail on maxs without parentheses"
        );
    }

    #[test]
    fn test_cordon_invalid_mins_format() {
        let input = r#"
        cordon
        {
            "mins" "(invalid data)"
            "maxs" "(1024 1024 1024)"
            "active" "0"
        }
        "#;

        let stream = lex(input);
        let result = Cordon::parse(stream);
        assert!(result.is_err(), "Parser should fail on invalid mins data");
    }

    #[test]
    fn test_cordon_invalid_maxs_format() {
        let input = r#"
        cordon
        {
            "mins" "(-1024 -1024 -1024)"
            "maxs" "(1024 1024)"
            "active" "0"
        }
        "#;

        let stream = lex(input);
        let result = Cordon::parse(stream);
        assert!(
            result.is_err(),
            "Parser should fail on maxs with too few values"
        );
    }

    #[test]
    fn test_cordon_invalid_active_value() {
        let input = r#"
        cordon
        {
            "mins" "(-1024 -1024 -1024)"
            "maxs" "(1024 1024 1024)"
            "active" "not_a_boolean"
        }
        "#;

        let stream = lex(input);
        let result = Cordon::parse(stream);
        assert!(
            result.is_err(),
            "Parser should fail on invalid active value"
        );
    }

    #[test]
    fn test_cordon_invalid_block_name() {
        let input = r#"
        wrongname
        {
            "mins" "(-1024 -1024 -1024)"
            "maxs" "(1024 1024 1024)"
            "active" "0"
        }
        "#;

        let stream = lex(input);
        let result = Cordon::parse(stream);
        assert!(result.is_err(), "Parser should fail on invalid block name");
    }

    #[test]
    fn test_cordon_missing_closing_brace() {
        let input = r#"
        cordon
        {
            "mins" "(-1024 -1024 -1024)"
            "maxs" "(1024 1024 1024)"
            "active" "0"
        "#;

        let stream = lex(input);
        let result = Cordon::parse(stream);
        assert!(
            result.is_err(),
            "Parser should fail on missing closing brace"
        );
    }

    #[test]
    fn test_cordon_minimal_properties() {
        let input = r#"
        cordon
        {
            "mins" "(0 0 0)"
            "maxs" "(0 0 0)"
        }
        "#;

        let stream = lex(input);
        let result = Cordon::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let cordon = result.unwrap();
        assert_eq!(cordon.mins.x, 0.0);
        assert_eq!(cordon.maxs.x, 0.0);
        assert_eq!(cordon.active, false); // Default value
    }
}
