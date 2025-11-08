use chumsky::IterParser;
use chumsky::Parser as ChumskyParser;

use crate::impl_block_properties_parser;
use crate::parser::{
    close_block, key_value, key_value_numeric, open_block, skip_unknown_block, InternalParser,
    TokenError, TokenSource,
};
use crate::types::point::key_value_plane;
use crate::types::textureaxis::key_value_texture_axis;
use crate::Parser;

use super::point::Point3D;
use super::textureaxis::TextureAxis;
use super::DispInfo;

/// Represents a side (face) of a solid brush
#[derive(Debug, Default, Clone)]
pub struct Side<'src> {
    pub id: u32,
    pub plane: (Point3D, Point3D, Point3D),
    pub material: &'src str,
    pub uaxis: TextureAxis,
    pub vaxis: TextureAxis,
    pub rotation: f32,
    pub lightmapscale: u32,
    pub smoothing_groups: u32,
    pub dispinfo: Option<DispInfo>, // Displacement information for terrain
}

/// Side properties used for parser impl
enum SideProperty<'src> {
    Id(u32),
    Plane((Point3D, Point3D, Point3D)),
    Material(&'src str),
    UAxis(TextureAxis),
    VAxis(TextureAxis),
    Rotation(f32),
    LightmapScale(u32),
    SmoothingGroups(u32),
    DispInfo(DispInfo),
}

/// Public parser trait implementation that allows [`Side`] to use ::parse(input) call.
impl<'src> Parser<'src> for Side<'src> {}

/// A [`Side`] implementation for [`Side`].
/// Every key-value pair needs to be in order, like in the example bellow.
///
/// usage: `let side = Side::parser().parse(input);`.
///
/// The format that is being parsed here is:
/// side
/// {
///     "id" "1"
///     "plane" "(-320 -320 0) (-320 320 0) (320 320 0)"
///     "material" "DEV/DEV_MEASUREGENERIC01B"
///     "uaxis" "[1 0 0 0] 0.25"
///     "vaxis" "[0 -1 0 0] 0.25"
///     "rotation" "0"
///     "lightmapscale" "16"
///     "smoothing_groups" "0"
/// }
impl<'src> InternalParser<'src> for Side<'src> {
    fn parser<I>() -> impl ChumskyParser<'src, I, Self, TokenError<'src>>
    where
        I: TokenSource<'src>,
    {
        impl_block_properties_parser! {
            property_list: SideProperty = {
                p_id                  = key_value_numeric("id")                 => SideProperty::Id,
                p_plane               = key_value_plane("plane")                => SideProperty::Plane,
                p_material            = key_value("material")                   => SideProperty::Material,
                p_uaxis               = key_value_texture_axis("uaxis")         => SideProperty::UAxis,
                p_vaxis               = key_value_texture_axis("vaxis")         => SideProperty::VAxis,
                p_rotation            = key_value_numeric("rotation")           => SideProperty::Rotation,
                p_lightmap_scale      = key_value_numeric("lightmapscale")      => SideProperty::LightmapScale,
                p_smoothing_groups    = key_value_numeric("smoothing_groups")   => SideProperty::SmoothingGroups,
            }
        }

        let dispinfo_parser = DispInfo::parser().map(SideProperty::DispInfo);
        let any_property_or_block = property_list
            .or(dispinfo_parser)
            .map(Some)
            .or(skip_unknown_block().map(|_| None));

        open_block("side")
            .ignore_then(
                any_property_or_block
                    .repeated()
                    .collect::<Vec<Option<SideProperty>>>(),
            )
            .then_ignore(close_block())
            .map(|properties: Vec<Option<SideProperty>>| {
                let mut side = Side::default();
                for prop_opt in properties {
                    if let Some(prop) = prop_opt {
                        match prop {
                            SideProperty::Id(val) => side.id = val,
                            SideProperty::Plane(val) => side.plane = val,
                            SideProperty::Material(val) => side.material = val,
                            SideProperty::UAxis(val) => side.uaxis = val,
                            SideProperty::VAxis(val) => side.vaxis = val,
                            SideProperty::Rotation(val) => side.rotation = val,
                            SideProperty::LightmapScale(val) => side.lightmapscale = val,
                            SideProperty::SmoothingGroups(val) => side.smoothing_groups = val,
                            SideProperty::DispInfo(val) => side.dispinfo = Some(val),
                        }
                    }
                }
                side
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::util::lex;

    use super::*;
    use chumsky::Parser as ChumskyParser;

    #[test]
    fn test_parse_side_complete_valid_order() {
        let input = r#"
        side
        {
            "id" "1"
            "plane" "(-320 -320 0) (-320 320 0) (320 320 0)"
            "material" "DEV/DEV_MEASUREGENERIC01B"
            "uaxis" "[1 0 0 0] 0.25"
            "vaxis" "[0 -1 0 0] 0.25"
            "rotation" "0"
            "lightmapscale" "16"
            "smoothing_groups" "0"
        }
        "#;
        let stream = lex(input);
        let result = Side::parser().parse(stream).into_result();

        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());
        let side = result.unwrap();

        let expected_plane = (
            Point3D {
                x: -320.0,
                y: -320.0,
                z: 0.0,
            },
            Point3D {
                x: -320.0,
                y: 320.0,
                z: 0.0,
            },
            Point3D {
                x: 320.0,
                y: 320.0,
                z: 0.0,
            },
        );
        let expected_uaxis = TextureAxis {
            x: 1.0,
            y: 0.0,
            z: 0.0,
            shift: 0.0,
            scale: 0.25,
        };
        let expected_vaxis = TextureAxis {
            x: 0.0,
            y: -1.0,
            z: 0.0,
            shift: 0.0,
            scale: 0.25,
        };

        assert_eq!(side.id, 1);
        assert_eq!(side.plane, expected_plane);
        assert_eq!(side.material, "DEV/DEV_MEASUREGENERIC01B");
        assert_eq!(side.uaxis, expected_uaxis);
        assert_eq!(side.vaxis, expected_vaxis);
        assert_eq!(side.rotation, 0.0);
        assert_eq!(side.lightmapscale, 16);
        assert_eq!(side.smoothing_groups, 0);
    }

    #[test]
    fn test_parse_side_properties_out_of_order() {
        let input = r#"
        side
        {
            "material" "BRICK/BRICKWALL001A"
            "id" "42"
            "uaxis" "[0 1 0 10] 0.125"
            "smoothing_groups" "1"
            "plane" "(0 0 0) (100 0 0) (100 100 0)"
            "lightmapscale" "32"
            "vaxis" "[1 0 0 20] 0.125"
            "rotation" "90"
        }
        "#;
        let stream = lex(input);
        let result = Side::parser().parse(stream).into_result();

        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());
        let side = result.unwrap();

        let expected_plane = (
            Point3D {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Point3D {
                x: 100.0,
                y: 0.0,
                z: 0.0,
            },
            Point3D {
                x: 100.0,
                y: 100.0,
                z: 0.0,
            },
        );
        let expected_uaxis = TextureAxis {
            x: 0.0,
            y: 1.0,
            z: 0.0,
            shift: 10.0,
            scale: 0.125,
        };
        let expected_vaxis = TextureAxis {
            x: 1.0,
            y: 0.0,
            z: 0.0,
            shift: 20.0,
            scale: 0.125,
        };

        assert_eq!(side.id, 42);
        assert_eq!(side.plane, expected_plane);
        assert_eq!(side.material, "BRICK/BRICKWALL001A");
        assert_eq!(side.uaxis, expected_uaxis);
        assert_eq!(side.vaxis, expected_vaxis);
        assert_eq!(side.rotation, 90.0);
        assert_eq!(side.lightmapscale, 32);
        assert_eq!(side.smoothing_groups, 1);
    }

    #[test]
    fn test_parse_side_missing_optional_properties() {
        let input = r#"
        side
        {
            "id" "3"
            "plane" "(1 1 1) (2 2 2) (3 3 3)"
            "material" "CONCRETE/CONCRETEFLOOR001"
            "uaxis" "[1 0 0 0] 1"
            "vaxis" "[0 1 0 0] 1"
        }
        "#;
        let stream = lex(input);
        let result = Side::parser().parse(stream).into_result();

        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());
        let side = result.unwrap();
        let default_side_for_missing_fields = Side::default(); // To get default values

        let expected_plane = (
            Point3D {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Point3D {
                x: 2.0,
                y: 2.0,
                z: 2.0,
            },
            Point3D {
                x: 3.0,
                y: 3.0,
                z: 3.0,
            },
        );
        let expected_uaxis = TextureAxis {
            x: 1.0,
            y: 0.0,
            z: 0.0,
            shift: 0.0,
            scale: 1.0,
        };
        let expected_vaxis = TextureAxis {
            x: 0.0,
            y: 1.0,
            z: 0.0,
            shift: 0.0,
            scale: 1.0,
        };

        assert_eq!(side.id, 3);
        assert_eq!(side.plane, expected_plane);
        assert_eq!(side.material, "CONCRETE/CONCRETEFLOOR001");
        assert_eq!(side.uaxis, expected_uaxis);
        assert_eq!(side.vaxis, expected_vaxis);
        assert_eq!(side.rotation, default_side_for_missing_fields.rotation);
        assert_eq!(
            side.lightmapscale,
            default_side_for_missing_fields.lightmapscale
        );
        assert_eq!(
            side.smoothing_groups,
            default_side_for_missing_fields.smoothing_groups
        );
    }

    #[test]
    fn test_parse_side_empty_block() {
        let input = r#"
        side
        {
        }
        "#;
        let stream = lex(input);
        let result = Side::parser().parse(stream).into_result();

        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());
        let side = result.unwrap();
        let expected_side = Side::default(); // Assumes all fields get their default values

        assert_eq!(side.id, expected_side.id);
        assert_eq!(side.plane, expected_side.plane);
        assert_eq!(side.material, expected_side.material);
        assert_eq!(side.uaxis, expected_side.uaxis);
        assert_eq!(side.vaxis, expected_side.vaxis);
        assert_eq!(side.rotation, expected_side.rotation);
        assert_eq!(side.lightmapscale, expected_side.lightmapscale);
        assert_eq!(side.smoothing_groups, expected_side.smoothing_groups);
    }

    #[test]
    fn test_parse_side_malformed_id() {
        let input = r#"
        side
        {
            "id" "not_a_number"
            "plane" "(-320 -320 0) (-320 320 0) (320 320 0)"
            "material" "DEV/DEV_MEASUREGENERIC01B"
            "uaxis" "[1 0 0 0] 0.25"
            "vaxis" "[0 -1 0 0] 0.25"
            "rotation" "0"
            "lightmapscale" "16"
            "smoothing_groups" "0"
        }
        "#;
        let stream = lex(input);
        let result = Side::parser().parse(stream).into_result();
        assert!(
            result.is_err(),
            "Parsing should have failed for malformed id"
        );
    }

    #[test]
    fn test_parse_side_malformed_plane() {
        let input = r#"
        side
        {
            "id" "1"
            "plane" "this_is_not_a_plane"
            "material" "DEV/DEV_MEASUREGENERIC01B"
            "uaxis" "[1 0 0 0] 0.25"
            "vaxis" "[0 -1 0 0] 0.25"
            "rotation" "0"
            "lightmapscale" "16"
            "smoothing_groups" "0"
        }
        "#;
        let stream = lex(input);
        let result = Side::parser().parse(stream).into_result();
        assert!(
            result.is_err(),
            "Parsing should have failed for malformed plane"
        );
    }

    #[test]
    fn test_parse_side_malformed_uaxis() {
        let input = r#"
        side
        {
            "id" "1"
            "plane" "(-320 -320 0) (-320 320 0) (320 320 0)"
            "material" "DEV/DEV_MEASUREGENERIC01B"
            "uaxis" "not_a_uaxis"
            "vaxis" "[0 -1 0 0] 0.25"
            "rotation" "0"
            "lightmapscale" "16"
            "smoothing_groups" "0"
        }
        "#;
        let stream = lex(input);
        let result = Side::parser().parse(stream).into_result();
        assert!(
            result.is_err(),
            "Parsing should have failed for malformed uaxis"
        );
    }

    #[test]
    fn test_parse_side_missing_closing_brace_for_block() {
        let input = r#"
        side
        {
            "id" "1"
            "plane" "(-320 -320 0) (-320 320 0) (320 320 0)"
            "material" "DEV/DEV_MEASUREGENERIC01B"
            "uaxis" "[1 0 0 0] 0.25"
            "vaxis" "[0 -1 0 0] 0.25"
            "rotation" "0"
            "lightmapscale" "16"
            "smoothing_groups" "0"
        "#;
        let stream = lex(input);
        let result = Side::parser().parse(stream).into_result();
        assert!(
            result.is_err(),
            "Parsing should have failed for missing closing brace"
        );
    }

    #[test]
    fn test_parse_side_unknown_property() {
        let input = r#"
        side
        {
            "id" "1"
            "unknown_property" "some_value"
            "material" "DEV/DEV_MEASUREGENERIC01B"
        }
        "#;
        let stream = lex(input);
        let result = Side::parser().parse(stream).into_result();

        assert!(
            result.is_err(),
            "Parsing should fail on unknown property if not explicitly skipped"
        );
    }
}
