use chumsky::{Parser as ChumskyParser, error::Rich};

use crate::parser::{TokenError, TokenSource, any_quoted_string, quoted_string};

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Parses a key-value pair where the value is a Point3D
pub(crate) fn key_value_point3d<'src, I>(
    key: &'src str,
) -> impl ChumskyParser<'src, I, Point3D, TokenError<'src>>
where
    I: TokenSource<'src>,
{
    quoted_string(key)
        .ignore_then(any_quoted_string())
        .try_map(move |value_str, span| {
            parse_point_from_numbers_str(value_str)
                .map_err(|err_msg| Rich::custom(span, format!("Invalid point: {}", err_msg)))
        })
}

/// Helper to parse a string segment like "1.0 2.5 -3.0" into a [`Point3D`]
pub(crate) fn parse_point_from_numbers_str(numbers_str: &str) -> Result<Point3D, String> {
    let mut parts = numbers_str.split_whitespace();

    if let (Some(x), Some(y), Some(z)) = (parts.next(), parts.next(), parts.next()) {
        let x = x
            .parse::<f32>()
            .map_err(|e| format!("invalid x '{}': {}", x, e))?;
        let y = y
            .parse::<f32>()
            .map_err(|e| format!("invalid y '{}': {}", y, e))?;
        let z = z
            .parse::<f32>()
            .map_err(|e| format!("invalid z '{}': {}", z, e))?;
        Ok(Point3D { x, y, z })
    } else {
        Err("invalid number of parts".to_string())
    }
}

/// Parses a "plane" to get tuple of three [`Point3D`]
/// Format for this is: "key" "(p1x p1y p1z) (p2x p2y p2z) (p3x p3y p3z)"
pub(crate) fn key_value_plane<'src, I>(
    key: &'static str,
) -> impl ChumskyParser<'src, I, (Point3D, Point3D, Point3D), TokenError<'src>>
where
    I: TokenSource<'src>,
{
    quoted_string(key)
        .ignore_then(any_quoted_string())
        .try_map(move |plane_value_str, span| {
            let mut points = [Point3D::default(); 3];
            let mut remainder = plane_value_str.trim();

            for i in 0..3 {
                // Find the opening parenthesis
                if let Some(open_idx) = remainder.find('(') {
                    remainder = &remainder[open_idx + 1..];
                } else {
                    return Err(Rich::custom(
                        span,
                        format!("Point {}: missing opening parenthesis", i + 1),
                    ));
                }

                // Find the closing parenthesis
                if let Some(close_idx) = remainder.find(')') {
                    let numbers_part = &remainder[..close_idx];
                    remainder = &remainder[close_idx + 1..];

                    match parse_point_from_numbers_str(numbers_part) {
                        Ok(point) => points[i] = point,
                        Err(err_msg) => {
                            return Err(Rich::custom(
                                span,
                                format!("Point {}: {} (in '{}')", i + 1, err_msg, numbers_part),
                            ));
                        }
                    }
                } else {
                    return Err(Rich::custom(
                        span,
                        format!("Point {}: missing closing parenthesis", i + 1),
                    ));
                }
            }

            if points.len() == 3 {
                Ok((points[0], points[1], points[2]))
            } else {
                Err(Rich::custom(
                    span,
                    "Internal error: Failed to collect 3 points".to_string(),
                ))
            }
        })
}

#[cfg(test)]
mod tests {
    use chumsky::Parser as _;

    use crate::{
        types::point::{Point3D, key_value_plane},
        util::lex,
    };

    #[test]
    fn test_parse_valid_plane() {
        let stream = lex(r#""test_plane" "(1.0 2.0 3.0) (4.0 5.0 6.0) (7.0 8.0 9.0)""#);

        let parser = key_value_plane("test_plane");
        let result = parser.parse(stream).into_result();

        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());
        let (p1, p2, p3) = result.unwrap();
        assert_eq!(
            p1,
            Point3D {
                x: 1.0,
                y: 2.0,
                z: 3.0
            }
        );
        assert_eq!(
            p2,
            Point3D {
                x: 4.0,
                y: 5.0,
                z: 6.0
            }
        );
        assert_eq!(
            p3,
            Point3D {
                x: 7.0,
                y: 8.0,
                z: 9.0
            }
        );
    }

    #[test]
    fn test_parse_plane_malformed_numbers() {
        let stream = lex(r#""test_plane" "(1.0 2.0 oops) (4.0 5.0 6.0) (7.0 8.0 9.0)""#);
        let parser = key_value_plane("test_plane");
        let result = parser.parse(stream).into_result();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_plane_missing_paren() {
        let stream = lex(r#""test_plane" "(1.0 2.0 3.0 (4.0 5.0 6.0) (7.0 8.0 9.0)""#);
        let parser = key_value_plane("test_plane");
        let result = parser.parse(stream).into_result();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_plane_too_few_points() {
        let stream = lex(r#""test_plane" "(1.0 2.0 3.0) (4.0 5.0 6.0)""#);
        let parser = key_value_plane("test_plane");
        let result = parser.parse(stream).into_result();
        assert!(result.is_err());
    }
}
