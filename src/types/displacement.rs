use chumsky::{IterParser, Parser as ChumskyParser};

use crate::{
    impl_block_properties_parser,
    parser::{
        any_quoted_string, close_block, key_value, key_value_boolean, key_value_numeric,
        open_block, quoted_string, InternalParser, TokenError, TokenSource,
    },
    types::point::{key_value_point3d, parse_point_from_numbers_str, Point3D},
    Parser,
};

/// Represents a displacement vertex
#[derive(Debug, Clone, Default)]
pub struct DispVertex {
    pub position: Point3D,
    pub normal: Point3D,
    pub distance: f32,
    pub alpha: f32,
}

/// Represents a displacement triangle
#[derive(Debug, Clone, Default)]
pub struct DispTri {
    pub indices: [u32; 3],
}

/// Represents displacement information for terrain
#[derive(Debug, Default)]
pub struct DispInfo {
    pub power: u32,              // Power of 2 determining grid size (2^power + 1)
    pub start_position: Point3D, // Starting position of the displacement
    pub elevation: f32,          // Base height offset
    pub subdiv: bool,            // Whether to use subdivision

    // Normals and distances
    pub normals: Vec<Point3D>,
    pub distances: Vec<f32>,

    // Offsets (x,y,z) for each vertex
    pub offsets: Vec<Point3D>,

    // Offset normals
    pub offset_normals: Vec<Point3D>,

    // Alpha values (transparency/blending)
    pub alphas: Vec<f32>,

    // Triangle tags for collision
    pub triangle_tags: Vec<u32>,

    // Allowed vertex positions
    pub allowed_verts: Vec<u32>,
    pub flags: u32,
}

/// Internal [`DispInfo`] Properties to be used in a parser impl
#[derive(Debug, Clone)]
enum DispInfoProperty {
    Power(u32),
    StartPosition(Point3D),
    Elevation(f32),
    Subdiv(bool),
    Flags(u32),
    NormalsBlock(Vec<Point3D>),
    DistancesBlock(Vec<f32>),
    OffsetsBlock(Vec<Point3D>),
    OffsetNormalsBlock(Vec<Point3D>),
    AlphasBlock(Vec<f32>),
    TriangleTagsBlock(Vec<u32>),
    AllowedVertsBlock(Vec<u32>),
}

/// Helper to parse a row of displacement data (key-value pair where key is "rowN")
fn parse_row_data<'src, I, T, F>(
    block_name: &'static str,
    parser_fn: F,
) -> impl ChumskyParser<'src, I, Vec<T>, TokenError<'src>>
where
    I: TokenSource<'src>,
    F: Fn(&'src str) -> Result<Vec<T>, String> + Clone + 'src,
    T: 'src,
{
    use chumsky::error::Rich;

    let row_parser =
        any_quoted_string()
            .then(any_quoted_string())
            .try_map(move |(key, value_str), span| {
                // Key should be like "row0", "row1", etc.
                parser_fn(value_str).map_err(|err_msg| {
                    Rich::custom(span, format!("Invalid {} data: {}", block_name, err_msg))
                })
            });

    open_block(block_name)
        .ignore_then(row_parser.repeated().collect::<Vec<Vec<T>>>())
        .then_ignore(close_block())
        .map(|rows: Vec<Vec<T>>| rows.into_iter().flatten().collect())
}

/// Parse a row of Point3D normals from a string like "x1 y1 z1 x2 y2 z2 ..."
fn parse_normals_row(value_str: &str) -> Result<Vec<Point3D>, String> {
    let parts: Vec<&str> = value_str.split_whitespace().collect();
    if parts.len() % 3 != 0 {
        return Err(format!(
            "expected multiple of 3 numbers, found {}",
            parts.len()
        ));
    }

    let mut normals = Vec::new();
    for chunk in parts.chunks(3) {
        let x = chunk[0]
            .parse::<f32>()
            .map_err(|e| format!("invalid x '{}': {}", chunk[0], e))?;
        let y = chunk[1]
            .parse::<f32>()
            .map_err(|e| format!("invalid y '{}': {}", chunk[1], e))?;
        let z = chunk[2]
            .parse::<f32>()
            .map_err(|e| format!("invalid z '{}': {}", chunk[2], e))?;
        normals.push(Point3D { x, y, z });
    }
    Ok(normals)
}

/// Parse a row of f32 distances from a string like "1.0 2.5 3.0 ..."
fn parse_distances_row(value_str: &str) -> Result<Vec<f32>, String> {
    let parts: Vec<&str> = value_str.split_whitespace().collect();
    parts
        .iter()
        .map(|s| {
            s.parse::<f32>()
                .map_err(|e| format!("invalid float '{}': {}", s, e))
        })
        .collect()
}

/// Parse a row of u32 values from a string like "0 1 2 3 ..."
fn parse_u32_row(value_str: &str) -> Result<Vec<u32>, String> {
    let parts: Vec<&str> = value_str.split_whitespace().collect();
    parts
        .iter()
        .map(|s| {
            s.parse::<u32>()
                .map_err(|e| format!("invalid integer '{}': {}", s, e))
        })
        .collect()
}

/// Parse startposition which has format "[x y z]"
fn parse_startposition(value_str: &str) -> Result<Point3D, String> {
    let trimmed = value_str.trim();

    // Remove brackets
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Err(format!(
            "startposition must be in format [x y z], got: {}",
            value_str
        ));
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    parse_point_from_numbers_str(inner)
}

/// Parses a key-value pair where the value is a Point3D with square brackets
fn key_value_startposition<'src, I>() -> impl ChumskyParser<'src, I, Point3D, TokenError<'src>>
where
    I: TokenSource<'src>,
{
    use chumsky::error::Rich;
    quoted_string("startposition")
        .ignore_then(any_quoted_string())
        .try_map(move |value_str, span| {
            parse_startposition(value_str).map_err(|err_msg| {
                Rich::custom(span, format!("Invalid startposition: {}", err_msg))
            })
        })
}

/// Public parser trait implementation that allows [`DispInfo`] to use ::parse(input) call.
impl Parser<'_> for DispInfo {}

/// A [`InternalParser`] implementation for [`DispInfo`].
///
/// usage: `let dispinfo = DispInfo::parser().parse(input);`.
///
/// The format that is being parsed here is:
/// ```ignore
/// dispinfo
/// {
///     "power" "3"
///     "startposition" "[0 0 0]"
///     "elevation" "0"
///     "subdiv" "0"
///     normals
///     {
///         "row0" "0 0 1 0 0 1 0 0 1"
///         "row1" "0 0 1 0 0 1 0 0 1"
///     }
///     distances
///     {
///         "row0" "0 0 0"
///         "row1" "0 0 0"
///     }
///     offsets
///     {
///         "row0" "0 0 0 0 0 0 0 0 0"
///     }
///     offset_normals
///     {
///         "row0" "0 0 0 0 0 0 0 0 0"
///     }
///     alphas
///     {
///         "row0" "0 0 0"
///     }
///     triangle_tags
///     {
///         "row0" "0 0 0"
///     }
///     allowed_verts
///     {
///         "10" "0 1 2 3 4 5 6 7 8 9"
///     }
/// }
/// ```
impl<'src> InternalParser<'src> for DispInfo {
    fn parser<I>() -> impl ChumskyParser<'src, I, Self, TokenError<'src>>
    where
        I: TokenSource<'src>,
    {
        let normals_parser =
            parse_row_data("normals", parse_normals_row).map(DispInfoProperty::NormalsBlock);
        let distances_parser =
            parse_row_data("distances", parse_distances_row).map(DispInfoProperty::DistancesBlock);
        let offsets_parser =
            parse_row_data("offsets", parse_normals_row).map(DispInfoProperty::OffsetsBlock);
        let offset_normals_parser = parse_row_data("offset_normals", parse_normals_row)
            .map(DispInfoProperty::OffsetNormalsBlock);
        let alphas_parser =
            parse_row_data("alphas", parse_distances_row).map(DispInfoProperty::AlphasBlock);
        let triangle_tags_parser =
            parse_row_data("triangle_tags", parse_u32_row).map(DispInfoProperty::TriangleTagsBlock);
        let allowed_verts_parser =
            parse_row_data("allowed_verts", parse_u32_row).map(DispInfoProperty::AllowedVertsBlock);

        impl_block_properties_parser! {
            property_list: DispInfoProperty = {
                p_power            = key_value_numeric("power")          => DispInfoProperty::Power,
                p_startposition    = key_value_startposition()           => DispInfoProperty::StartPosition,
                p_elevation        = key_value_numeric("elevation")      => DispInfoProperty::Elevation,
                p_subdiv           = key_value_boolean("subdiv")         => DispInfoProperty::Subdiv,
                p_flags            = key_value_numeric("flags")          => DispInfoProperty::Flags,
            }
        }

        let any_property = property_list
            .or(normals_parser)
            .or(distances_parser)
            .or(offsets_parser)
            .or(offset_normals_parser)
            .or(alphas_parser)
            .or(triangle_tags_parser)
            .or(allowed_verts_parser);

        open_block("dispinfo")
            .ignore_then(any_property.repeated().collect::<Vec<DispInfoProperty>>())
            .then_ignore(close_block())
            .map(|properties: Vec<DispInfoProperty>| {
                let mut dispinfo = DispInfo::default();
                for prop in properties {
                    match prop {
                        DispInfoProperty::Power(val) => dispinfo.power = val,
                        DispInfoProperty::StartPosition(val) => dispinfo.start_position = val,
                        DispInfoProperty::Elevation(val) => dispinfo.elevation = val,
                        DispInfoProperty::Subdiv(val) => dispinfo.subdiv = val,
                        DispInfoProperty::Flags(val) => dispinfo.flags = val,
                        DispInfoProperty::NormalsBlock(val) => dispinfo.normals = val,
                        DispInfoProperty::DistancesBlock(val) => dispinfo.distances = val,
                        DispInfoProperty::OffsetsBlock(val) => dispinfo.offsets = val,
                        DispInfoProperty::OffsetNormalsBlock(val) => dispinfo.offset_normals = val,
                        DispInfoProperty::AlphasBlock(val) => dispinfo.alphas = val,
                        DispInfoProperty::TriangleTagsBlock(val) => dispinfo.triangle_tags = val,
                        DispInfoProperty::AllowedVertsBlock(val) => dispinfo.allowed_verts = val,
                    }
                }
                dispinfo
            })
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::lex;

    #[test]
    fn test_dispinfo_minimal() {
        let input = r#"
        dispinfo
        {
            "power" "2"
            "startposition" "[0 0 0]"
            "elevation" "0"
            "subdiv" "0"
        }
        "#;

        let stream = lex(input);
        let result = DispInfo::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let dispinfo = result.unwrap();
        assert_eq!(dispinfo.power, 2);
        assert_eq!(dispinfo.elevation, 0.0);
        assert_eq!(dispinfo.subdiv, false);
        assert_eq!(dispinfo.start_position.x, 0.0);
        assert_eq!(dispinfo.start_position.y, 0.0);
        assert_eq!(dispinfo.start_position.z, 0.0);
    }

    #[test]
    fn test_dispinfo_with_normals() {
        let input = r#"
        dispinfo
        {
            "power" "3"
            "startposition" "[100 200 0]"
            "elevation" "5"
            "subdiv" "1"
            normals
            {
                "row0" "0 0 1 0 0 1 0 0 1"
                "row1" "0 0 1 0 0 1 0 0 1"
            }
        }
        "#;

        let stream = lex(input);
        let result = DispInfo::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let dispinfo = result.unwrap();
        assert_eq!(dispinfo.power, 3);
        assert_eq!(dispinfo.elevation, 5.0);
        assert_eq!(dispinfo.subdiv, true);
        assert_eq!(dispinfo.start_position.x, 100.0);
        assert_eq!(dispinfo.start_position.y, 200.0);
        assert_eq!(dispinfo.start_position.z, 0.0);
        assert_eq!(dispinfo.normals.len(), 6); // 2 rows * 3 normals each
        assert_eq!(dispinfo.normals[0].x, 0.0);
        assert_eq!(dispinfo.normals[0].y, 0.0);
        assert_eq!(dispinfo.normals[0].z, 1.0);
    }

    #[test]
    fn test_dispinfo_with_distances() {
        let input = r#"
        dispinfo
        {
            "power" "2"
            "startposition" "[0 0 0]"
            "elevation" "0"
            "subdiv" "0"
            distances
            {
                "row0" "1.0 2.5 3.0"
                "row1" "4.0 5.5 6.0"
            }
        }
        "#;

        let stream = lex(input);
        let result = DispInfo::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let dispinfo = result.unwrap();
        assert_eq!(dispinfo.distances.len(), 6);
        assert_eq!(dispinfo.distances[0], 1.0);
        assert_eq!(dispinfo.distances[1], 2.5);
        assert_eq!(dispinfo.distances[2], 3.0);
        assert_eq!(dispinfo.distances[3], 4.0);
    }

    #[test]
    fn test_dispinfo_with_offsets() {
        let input = r#"
        dispinfo
        {
            "power" "2"
            "startposition" "[0 0 0]"
            "elevation" "0"
            "subdiv" "0"
            offsets
            {
                "row0" "0 0 5 0 0 10 0 0 15"
            }
        }
        "#;

        let stream = lex(input);
        let result = DispInfo::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let dispinfo = result.unwrap();
        assert_eq!(dispinfo.offsets.len(), 3);
        assert_eq!(dispinfo.offsets[0].z, 5.0);
        assert_eq!(dispinfo.offsets[1].z, 10.0);
        assert_eq!(dispinfo.offsets[2].z, 15.0);
    }

    #[test]
    fn test_dispinfo_with_alphas() {
        let input = r#"
        dispinfo
        {
            "power" "2"
            "startposition" "[0 0 0]"
            "elevation" "0"
            "subdiv" "0"
            alphas
            {
                "row0" "0 128 255"
            }
        }
        "#;

        let stream = lex(input);
        let result = DispInfo::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let dispinfo = result.unwrap();
        assert_eq!(dispinfo.alphas.len(), 3);
        assert_eq!(dispinfo.alphas[0], 0.0);
        assert_eq!(dispinfo.alphas[1], 128.0);
        assert_eq!(dispinfo.alphas[2], 255.0);
    }

    #[test]
    fn test_dispinfo_with_triangle_tags() {
        let input = r#"
        dispinfo
        {
            "power" "2"
            "startposition" "[0 0 0]"
            "elevation" "0"
            "subdiv" "0"
            triangle_tags
            {
                "row0" "0 1 2 3 4"
            }
        }
        "#;

        let stream = lex(input);
        let result = DispInfo::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let dispinfo = result.unwrap();
        assert_eq!(dispinfo.triangle_tags.len(), 5);
        assert_eq!(dispinfo.triangle_tags[0], 0);
        assert_eq!(dispinfo.triangle_tags[4], 4);
    }

    #[test]
    fn test_dispinfo_with_allowed_verts() {
        let input = r#"
        dispinfo
        {
            "power" "2"
            "startposition" "[0 0 0]"
            "elevation" "0"
            "subdiv" "0"
            allowed_verts
            {
                "10" "0 1 2 3 4 5 6 7 8 9"
            }
        }
        "#;

        let stream = lex(input);
        let result = DispInfo::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let dispinfo = result.unwrap();
        assert_eq!(dispinfo.allowed_verts.len(), 10);
        assert_eq!(dispinfo.allowed_verts[0], 0);
        assert_eq!(dispinfo.allowed_verts[9], 9);
    }

    #[test]
    fn test_dispinfo_complete() {
        let input = r#"
        dispinfo
        {
            "power" "3"
            "startposition" "[128 256 64]"
            "elevation" "10"
            "subdiv" "1"
            "flags" "0"
            normals
            {
                "row0" "0 0 1 0 0 1"
            }
            distances
            {
                "row0" "0 0"
            }
            offsets
            {
                "row0" "0 0 0 0 0 0"
            }
            offset_normals
            {
                "row0" "0 0 0 0 0 0"
            }
            alphas
            {
                "row0" "0 0"
            }
            triangle_tags
            {
                "row0" "0 0"
            }
            allowed_verts
            {
                "10" "0 1 2 3 4 5 6 7 8 9"
            }
        }
        "#;

        let stream = lex(input);
        let result = DispInfo::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let dispinfo = result.unwrap();
        assert_eq!(dispinfo.power, 3);
        assert_eq!(dispinfo.start_position.x, 128.0);
        assert_eq!(dispinfo.elevation, 10.0);
        assert_eq!(dispinfo.subdiv, true);
        assert_eq!(dispinfo.flags, 0);
        assert_eq!(dispinfo.normals.len(), 2);
        assert_eq!(dispinfo.distances.len(), 2);
        assert_eq!(dispinfo.offsets.len(), 2);
        assert_eq!(dispinfo.offset_normals.len(), 2);
        assert_eq!(dispinfo.alphas.len(), 2);
        assert_eq!(dispinfo.triangle_tags.len(), 2);
        assert_eq!(dispinfo.allowed_verts.len(), 10);
    }

    #[test]
    fn test_dispinfo_properties_out_of_order() {
        let input = r#"
        dispinfo
        {
            "subdiv" "1"
            "elevation" "5"
            normals
            {
                "row0" "0 0 1"
            }
            "power" "2"
            "startposition" "[0 0 0]"
            distances
            {
                "row0" "0"
            }
        }
        "#;

        let stream = lex(input);
        let result = DispInfo::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let dispinfo = result.unwrap();
        assert_eq!(dispinfo.power, 2);
        assert_eq!(dispinfo.subdiv, true);
        assert_eq!(dispinfo.elevation, 5.0);
        assert_eq!(dispinfo.normals.len(), 1);
        assert_eq!(dispinfo.distances.len(), 1);
    }

    #[test]
    fn test_dispinfo_empty() {
        let input = r#"
        dispinfo
        {
        }
        "#;

        let stream = lex(input);
        let result = DispInfo::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let dispinfo = result.unwrap();
        assert_eq!(dispinfo.power, 0);
        assert_eq!(dispinfo.normals.len(), 0);
    }

    #[test]
    fn test_dispinfo_invalid_normals() {
        let input = r#"
        dispinfo
        {
            "power" "2"
            normals
            {
                "row0" "0 0 invalid"
            }
        }
        "#;

        let stream = lex(input);
        let result = DispInfo::parse(stream);
        assert!(result.is_err(), "Parser should fail on invalid normals");
    }

    #[test]
    fn test_dispinfo_invalid_distances() {
        let input = r#"
        dispinfo
        {
            "power" "2"
            distances
            {
                "row0" "1.0 not_a_number"
            }
        }
        "#;

        let stream = lex(input);
        let result = DispInfo::parse(stream);
        assert!(result.is_err(), "Parser should fail on invalid distances");
    }

    #[test]
    fn test_dispinfo_invalid_block_name() {
        let input = r#"
        wrongname
        {
            "power" "2"
        }
        "#;

        let stream = lex(input);
        let result = DispInfo::parse(stream);
        assert!(result.is_err(), "Parser should fail on invalid block name");
    }

    #[test]
    fn test_dispinfo_missing_closing_brace() {
        let input = r#"
        dispinfo
        {
            "power" "2"
        "#;

        let stream = lex(input);
        let result = DispInfo::parse(stream);
        assert!(
            result.is_err(),
            "Parser should fail on missing closing brace"
        );
    }

    #[test]
    fn test_dispinfo_startposition_without_brackets() {
        let input = r#"
        dispinfo
        {
            "power" "2"
            "startposition" "0 0 0"
            "elevation" "0"
            "subdiv" "0"
        }
        "#;

        let stream = lex(input);
        let result = DispInfo::parse(stream);
        assert!(
            result.is_err(),
            "Parser should fail on startposition without brackets"
        );
    }

    #[test]
    fn test_dispinfo_startposition_with_brackets() {
        let input = r#"
        dispinfo
        {
            "power" "2"
            "startposition" "[100 200 300]"
            "elevation" "5"
            "subdiv" "1"
        }
        "#;

        let stream = lex(input);
        let result = DispInfo::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let dispinfo = result.unwrap();
        assert_eq!(dispinfo.start_position.x, 100.0);
        assert_eq!(dispinfo.start_position.y, 200.0);
        assert_eq!(dispinfo.start_position.z, 300.0);
    }
}
