use chumsky::Parser as ChumskyParser;

use crate::parser::{TokenError, TokenSource, any_quoted_string};

/// Represents an output connection between entities
#[derive(Debug, Default, Clone)]
pub struct EntityOutput<'src> {
    pub output_name: &'src str,
    pub target: &'src str,
    pub input: &'src str,
    pub parameter: &'src str,
    pub delay: f32,
    pub times_to_fire: i32,
}

impl<'src> EntityOutput<'src> {
    /// Parse an output string in the format: "target,input,parameter,delay,times_to_fire"
    /// Example: "motor*,TurnOn,,0,-1"
    pub fn parse_output_string(output_name: &'src str, value: &'src str) -> Result<Self, String> {
        let mut parts = value.split(',').map(|split| split.trim());

        let (target, input, parameter, delay, times_to_fire) = match (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) {
            (Some(a), Some(b), Some(c), Some(d), Some(e)) => {
                let delay = d
                    .parse::<f32>()
                    .map_err(|e| format!("invalid delay '{}': {}", d, e))?;
                let times_to_fire = e
                    .parse::<i32>()
                    .map_err(|e| format!("invalid times_to_fire '{}': {}", e, e))?;
                (a, b, c, delay, times_to_fire)
            }
            _ => return Err("expected at least 5 comma-separated values".into()),
        };

        Ok(EntityOutput::<'src> {
            output_name,
            target,
            input,
            parameter,
            delay,
            times_to_fire,
        })
    }
}

/// Parser for a single output key-value pair
/// Format: "OutputName" "target,input,parameter,delay,times"
pub(crate) fn parse_output_entry<'src, I>()
-> impl ChumskyParser<'src, I, EntityOutput<'src>, TokenError<'src>>
where
    I: TokenSource<'src>,
{
    use chumsky::error::Rich;

    any_quoted_string()
        .then(any_quoted_string())
        .try_map(|(output_name, value_str), span| {
            EntityOutput::parse_output_string(output_name, value_str)
                .map_err(|err_msg| Rich::custom(span, format!("Invalid output: {}", err_msg)))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::lex;

    #[test]
    fn test_parse_output_string_complete() {
        let result = EntityOutput::parse_output_string("OnIn", "motor*,TurnOn,,0,-1");
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.output_name, "OnIn");
        assert_eq!(output.target, "motor*");
        assert_eq!(output.input, "TurnOn");
        assert_eq!(output.parameter, "");
        assert_eq!(output.delay, 0.0);
        assert_eq!(output.times_to_fire, -1);
    }

    #[test]
    fn test_parse_output_string_with_parameter() {
        let result = EntityOutput::parse_output_string("OnStartTouch", "door1,Open,fast,0.5,1");
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.output_name, "OnStartTouch");
        assert_eq!(output.target, "door1");
        assert_eq!(output.input, "Open");
        assert_eq!(output.parameter, "fast");
        assert_eq!(output.delay, 0.5);
        assert_eq!(output.times_to_fire, 1);
    }

    #[test]
    fn test_parse_output_string_invalid_format() {
        let result = EntityOutput::parse_output_string("OnUse", "target,input,param");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_output_string_invalid_delay() {
        let result = EntityOutput::parse_output_string("OnUse", "target,input,,not_a_number,-1");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_output_string_invalid_times() {
        let result = EntityOutput::parse_output_string("OnUse", "target,input,,0,not_a_number");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_output_entry() {
        let input = r#""OnIn" "motor*,TurnOn,,0,-1""#;
        let stream = lex(input);

        let parser = parse_output_entry();
        let result = parser.parse(stream).into_result();

        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());
        let output = result.unwrap();
        assert_eq!(output.output_name, "OnIn");
        assert_eq!(output.target, "motor*");
        assert_eq!(output.input, "TurnOn");
        assert_eq!(output.delay, 0.0);
        assert_eq!(output.times_to_fire, -1);
    }

    #[test]
    fn test_parse_output_entry_with_spaces() {
        let input = r#""OnOut" "motor*, TurnOff, , 0.5, 1""#;
        let stream = lex(input);

        let parser = parse_output_entry();
        let result = parser.parse(stream).into_result();

        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());
        let output = result.unwrap();
        assert_eq!(output.output_name, "OnOut");
        assert_eq!(output.target, "motor*");
        assert_eq!(output.input, "TurnOff");
        assert_eq!(output.parameter, "");
        assert_eq!(output.delay, 0.5);
        assert_eq!(output.times_to_fire, 1);
    }
}
