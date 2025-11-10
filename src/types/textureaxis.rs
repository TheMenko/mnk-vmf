use chumsky::{Parser as ChumskyParser, error::Rich};

use crate::parser::{TokenError, TokenSource, any_quoted_string, quoted_string};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct TextureAxis {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub shift: f32,
    pub scale: f32,
}

/// Helper to parse a string segment like "1.0 0.0 0.0 16.0" into (x, y, z, shift)
fn parse_texture_vector_str(numbers_str: &str) -> Result<(f32, f32, f32, f32), String> {
    let mut parts = numbers_str.split_whitespace();

    if let (Some(x_str), Some(y_str), Some(z_str), Some(shift_str)) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    {
        let x = x_str
            .parse::<f32>()
            .map_err(|e| format!("invalid x '{}': {}", x_str, e))?;
        let y = y_str
            .parse::<f32>()
            .map_err(|e| format!("invalid y '{}': {}", y_str, e))?;
        let z = z_str
            .parse::<f32>()
            .map_err(|e| format!("invalid z '{}': {}", z_str, e))?;
        let shift = shift_str
            .parse::<f32>()
            .map_err(|e| format!("invalid shift '{}': {}", shift_str, e))?;
        Ok((x, y, z, shift))
    } else {
        Err("expected multiple of 3 numbers".into())
    }
}

/// Parses a "uaxis" or "vaxis" to get a [`TextureAxis`]
/// Format for this is: "key" "[x y z shift] scale"
pub(crate) fn key_value_texture_axis<'src, I>(
    key: &'static str,
) -> impl ChumskyParser<'src, I, TextureAxis, TokenError<'src>>
where
    I: TokenSource<'src>,
{
    quoted_string(key) // Parses "uaxis" or "vaxis"
        .ignore_then(any_quoted_string()) // Gets the content string "[1 0 0 0] 0.25"
        .try_map(move |value_str, span| {
            let mut remainder = value_str.trim();

            let vector_part_str: &str;
            if let Some(open_idx) = remainder.find('[') {
                remainder = &remainder[open_idx + 1..];
            } else {
                return Err(Rich::custom(
                    span,
                    "Missing opening bracket '[' for texture axis vector".to_string(),
                ));
            }

            if let Some(close_idx) = remainder.find(']') {
                vector_part_str = &remainder[..close_idx];
                remainder = &remainder[close_idx + 1..];
            } else {
                return Err(Rich::custom(
                    span,
                    "Missing closing bracket ']' for texture axis vector".to_string(),
                ));
            }

            let (x, y, z, shift) = match parse_texture_vector_str(vector_part_str) {
                Ok(v) => v,
                Err(err_msg) => {
                    return Err(Rich::custom(
                        span,
                        format!("Invalid texture vector: {err_msg} (in '{vector_part_str}')",),
                    ));
                }
            };

            let scale_str = remainder.trim();
            if scale_str.is_empty() {
                return Err(Rich::custom(
                    span,
                    "Missing scale value after texture vector".to_string(),
                ));
            }

            let scale = match scale_str.parse::<f32>() {
                Ok(s) => s,
                Err(e) => {
                    return Err(Rich::custom(
                        span,
                        format!("Invalid scale value '{scale_str}': {e}"),
                    ));
                }
            };

            Ok(TextureAxis {
                x,
                y,
                z,
                shift,
                scale,
            })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::lex;

    #[test]
    fn test_parse_valid_uaxis() {
        let stream = lex(r#""uaxis" "[1 0 0 0] 0.25""#); // Use raw string for convenience
        let parser = key_value_texture_axis("uaxis");
        let result = parser.parse(stream).into_result();

        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());
        let axis = result.unwrap();
        assert_eq!(
            axis,
            TextureAxis {
                x: 1.0,
                y: 0.0,
                z: 0.0,
                shift: 0.0,
                scale: 0.25
            }
        );
    }

    #[test]
    fn test_parse_valid_vaxis_with_shift() {
        let stream = lex(r#""vaxis" "[0 -1 0 128] 0.5""#);
        let parser = key_value_texture_axis("vaxis");
        let result = parser.parse(stream).into_result();

        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());
        let axis = result.unwrap();
        assert_eq!(
            axis,
            TextureAxis {
                x: 0.0,
                y: -1.0,
                z: 0.0,
                shift: 128.0,
                scale: 0.5
            }
        );
    }

    #[test]
    fn test_parse_texture_axis_missing_bracket_open() {
        let stream = lex(r#""uaxis" "1 0 0 0] 0.25""#);
        let parser = key_value_texture_axis("uaxis");
        let result = parser.parse(stream).into_result();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_texture_axis_missing_bracket_close() {
        let stream = lex(r#""uaxis" "[1 0 0 0 0.25""#);
        let parser = key_value_texture_axis("uaxis");
        let result = parser.parse(stream).into_result();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_texture_axis_malformed_vector_numbers() {
        let stream = lex(r#""uaxis" "[1 0 oops 0] 0.25""#);
        let parser = key_value_texture_axis("uaxis");
        let result = parser.parse(stream).into_result();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_texture_axis_too_few_vector_numbers() {
        let stream = lex(r#""uaxis" "[1 0 0] 0.25""#);
        let parser = key_value_texture_axis("uaxis");
        let result = parser.parse(stream).into_result();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_texture_axis_missing_scale() {
        let stream = lex(r#""uaxis" "[1 0 0 0]""#); // Scale is missing
        let parser = key_value_texture_axis("uaxis");
        let result = parser.parse(stream).into_result();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_texture_axis_malformed_scale() {
        let stream = lex(r#""uaxis" "[1 0 0 0] scale_text""#);
        let parser = key_value_texture_axis("uaxis");
        let result = parser.parse(stream).into_result();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_texture_axis_extra_garbage_after_scale() {
        // The current manual parsing for scale will consume everything after the vector.
        // If "0.25 garbage" is parsed as f32, it might succeed or fail depending on Rust's f32::parse.
        // Standard f32::parse would fail.
        let stream = lex(r#""uaxis" "[1 0 0 0] 0.25 garbage""#);
        let parser = key_value_texture_axis("uaxis");
        let result = parser.parse(stream).into_result();
        assert!(result.is_err()); // because "0.25 garbage" is not a valid f32
    }
}
