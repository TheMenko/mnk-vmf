use chumsky::{IterParser, Parser as ChumskyParser};
use std::collections::HashMap;

use crate::{
    impl_block_properties_parser,
    parser::{
        any_quoted_string, close_block, key_value, key_value_boolean, key_value_numeric,
        open_block, InternalParser, TokenError, TokenSource,
    },
    types::{EditorData, Solid},
    Parser,
};

/// Represents the worldspawn entity in a VMF file
#[derive(Debug, Default)]
pub struct World<'src> {
    pub id: u32,
    pub mapversion: u32,
    pub classname: &'src str,
    pub detailmaterial: Option<&'src str>,
    pub detailvbsp: Option<&'src str>,
    pub maxpropscreenwidth: Option<i32>,
    pub skyname: Option<&'src str>,
    pub sounds: Option<u32>,
    pub maxrange: Option<f32>,

    // Game-specific properties
    pub maxoccludeearea: Option<f32>,
    pub minoccluderarea: Option<f32>,
    pub maxoccludeearea_csgo: Option<f32>,
    pub minoccluderarea_csgo: Option<f32>,
    pub difficulty_level: Option<u32>,
    pub hdr_level: Option<u32>,

    // Geometry
    pub solids: Vec<Solid<'src>>,

    // Entity connections
    pub targetname: Option<&'src str>,
    pub target: Option<&'src str>,

    // Custom key-value pairs for world-specific properties
    pub properties: HashMap<&'src str, &'src str>,

    // Editor data
    pub hidden: Option<bool>,
    pub group: Option<u32>,
    pub editor: Option<EditorData>,
}

/// Internal [`World`] Properties to be used in a parser impl
#[derive(Debug, Clone)]
enum WorldProperty<'src> {
    Id(u32),
    MapVersion(u32),
    Classname(&'src str),
    DetailMaterial(&'src str),
    DetailVbsp(&'src str),
    MaxPropScreenWidth(i32),
    Skyname(&'src str),
    Sounds(u32),
    MaxRange(f32),
    MaxOccludeeArea(f32),
    MinOccluderArea(f32),
    MaxOccludeeAreaCsgo(f32),
    MinOccluderAreaCsgo(f32),
    DifficultyLevel(u32),
    HdrLevel(u32),
    Targetname(&'src str),
    Target(&'src str),
    Hidden(bool),
    Group(u32),
    Editor(EditorData),
    Solid(Solid<'src>),
    Custom(&'src str, &'src str),
}

/// Public parser trait implementation
impl<'src> Parser<'src> for World<'src> {}

/// InternalParser implementation for World
impl<'src> InternalParser<'src> for World<'src> {
    fn parser<I>() -> impl ChumskyParser<'src, I, Self, TokenError<'src>>
    where
        I: TokenSource<'src>,
    {
        impl_block_properties_parser! {
            known_properties: WorldProperty<'src> = {
                p_id                    = key_value_numeric("id")                     => WorldProperty::Id,
                p_mapversion            = key_value_numeric("mapversion")             => WorldProperty::MapVersion,
                p_classname             = key_value("classname")                      => |s: &str| WorldProperty::Classname(s),
                p_detailmaterial        = key_value("detailmaterial")                 => |s: &str| WorldProperty::DetailMaterial(s),
                p_detailvbsp            = key_value("detailvbsp")                     => |s: &str| WorldProperty::DetailVbsp(s),
                p_maxpropscreenwidth    = key_value_numeric("maxpropscreenwidth")     => WorldProperty::MaxPropScreenWidth,
                p_skyname               = key_value("skyname")                        => |s: &str| WorldProperty::Skyname(s),
                p_sounds                = key_value_numeric("sounds")                 => WorldProperty::Sounds,
                p_maxrange              = key_value_numeric("maxrange")               => WorldProperty::MaxRange,
                p_maxoccludeearea       = key_value_numeric("maxoccludeearea")        => WorldProperty::MaxOccludeeArea,
                p_minoccluderarea       = key_value_numeric("minoccluderarea")        => WorldProperty::MinOccluderArea,
                p_maxoccludeearea_csgo  = key_value_numeric("maxoccludeearea_csgo")   => WorldProperty::MaxOccludeeAreaCsgo,
                p_minoccluderarea_csgo  = key_value_numeric("minoccluderarea_csgo")   => WorldProperty::MinOccluderAreaCsgo,
                p_difficulty_level      = key_value_numeric("difficulty_level")       => WorldProperty::DifficultyLevel,
                p_hdr_level             = key_value_numeric("hdr_level")              => WorldProperty::HdrLevel,
                p_targetname            = key_value("targetname")                     => |s: &str| WorldProperty::Targetname(s),
                p_target                = key_value("target")                         => |s: &str| WorldProperty::Target(s),
                p_hidden                = key_value_boolean("hidden")                 => WorldProperty::Hidden,
                p_group                 = key_value_numeric("group")                  => WorldProperty::Group,
            }
        }

        let editor_parser = EditorData::parser().map(WorldProperty::Editor);
        let solid_parser = Solid::parser().map(WorldProperty::Solid);
        let custom_property = any_quoted_string()
            .then(any_quoted_string())
            .map(|(key, value): (&str, &str)| WorldProperty::Custom(key, value));

        let any_property = known_properties
            .or(editor_parser)
            .or(solid_parser)
            .or(custom_property);

        open_block("world")
            .ignore_then(
                any_property
                    .repeated()
                    .collect::<Vec<WorldProperty<'src>>>(),
            )
            .then_ignore(close_block())
            .map(|properties: Vec<WorldProperty<'src>>| {
                let mut world = World::default();
                for prop in properties {
                    match prop {
                        WorldProperty::Id(val) => world.id = val,
                        WorldProperty::MapVersion(val) => world.mapversion = val,
                        WorldProperty::Classname(val) => world.classname = val,
                        WorldProperty::DetailMaterial(val) => world.detailmaterial = Some(val),
                        WorldProperty::DetailVbsp(val) => world.detailvbsp = Some(val),
                        WorldProperty::MaxPropScreenWidth(val) => {
                            world.maxpropscreenwidth = Some(val)
                        }
                        WorldProperty::Skyname(val) => world.skyname = Some(val),
                        WorldProperty::Sounds(val) => world.sounds = Some(val),
                        WorldProperty::MaxRange(val) => world.maxrange = Some(val),
                        WorldProperty::MaxOccludeeArea(val) => world.maxoccludeearea = Some(val),
                        WorldProperty::MinOccluderArea(val) => world.minoccluderarea = Some(val),
                        WorldProperty::MaxOccludeeAreaCsgo(val) => {
                            world.maxoccludeearea_csgo = Some(val)
                        }
                        WorldProperty::MinOccluderAreaCsgo(val) => {
                            world.minoccluderarea_csgo = Some(val)
                        }
                        WorldProperty::DifficultyLevel(val) => world.difficulty_level = Some(val),
                        WorldProperty::HdrLevel(val) => world.hdr_level = Some(val),
                        WorldProperty::Targetname(val) => world.targetname = Some(val),
                        WorldProperty::Target(val) => world.target = Some(val),
                        WorldProperty::Hidden(val) => world.hidden = Some(val),
                        WorldProperty::Group(val) => world.group = Some(val),
                        WorldProperty::Editor(val) => world.editor = Some(val),
                        WorldProperty::Solid(val) => world.solids.push(val),
                        WorldProperty::Custom(key, value) => {
                            world.properties.insert(key, value);
                        }
                    }
                }
                world
            })
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::lex;

    #[test]
    fn test_world_minimal() {
        let input = r#"
        world
        {
            "id" "1"
            "mapversion" "16"
            "classname" "worldspawn"
        }
        "#;

        let stream = lex(input);
        let result = World::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let world = result.unwrap();
        assert_eq!(world.id, 1);
        assert_eq!(world.mapversion, 16);
        assert_eq!(world.classname, "worldspawn");
        assert_eq!(world.solids.len(), 0);
        assert!(world.editor.is_none());
    }

    #[test]
    fn test_world_with_properties() {
        let input = r#"
        world
        {
            "id" "1"
            "mapversion" "16"
            "classname" "worldspawn"
            "detailmaterial" "detail/detailsprites"
            "detailvbsp" "detail.vbsp"
            "maxpropscreenwidth" "-1"
            "skyname" "sky_day01_01"
        }
        "#;

        let stream = lex(input);
        let result = World::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let world = result.unwrap();
        assert_eq!(world.detailmaterial, Some("detail/detailsprites"));
        assert_eq!(world.detailvbsp, Some("detail.vbsp"));
        assert_eq!(world.maxpropscreenwidth, Some(-1));
        assert_eq!(world.skyname, Some("sky_day01_01"));
    }

    #[test]
    fn test_world_with_solids() {
        let input = r#"
        world
        {
            "id" "1"
            "mapversion" "16"
            "classname" "worldspawn"
            solid
            {
                "id" "9"
                side
                {
                    "id" "1"
                    "plane" "(0 0 0) (1 0 0) (1 1 0)"
                    "material" "DEV/DEV_MEASUREGENERIC01B"
                    "uaxis" "[1 0 0 0] 0.25"
                    "vaxis" "[0 -1 0 0] 0.25"
                }
            }
            solid
            {
                "id" "30"
                side
                {
                    "id" "2"
                    "plane" "(0 0 0) (1 0 0) (1 1 0)"
                    "material" "DEV/DEV_MEASUREGENERIC01"
                    "uaxis" "[1 0 0 0] 0.25"
                    "vaxis" "[0 -1 0 0] 0.25"
                }
            }
        }
        "#;

        let stream = lex(input);
        let result = World::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let world = result.unwrap();
        assert_eq!(world.solids.len(), 2);
        assert_eq!(world.solids[0].id, 9);
        assert_eq!(world.solids[1].id, 30);
    }

    #[test]
    fn test_world_with_multiple_solids() {
        let input = r#"
        world
        {
            "id" "1"
            "mapversion" "16"
            "classname" "worldspawn"
            "skyname" "sky_day01_01"
            solid
            {
                "id" "1"
                side
                {
                    "id" "1"
                    "plane" "(0 0 0) (1 0 0) (1 1 0)"
                    "material" "CONCRETE"
                    "uaxis" "[1 0 0 0] 0.25"
                    "vaxis" "[0 -1 0 0] 0.25"
                }
            }
            solid
            {
                "id" "2"
                side
                {
                    "id" "2"
                    "plane" "(0 0 0) (1 0 0) (1 1 0)"
                    "material" "METAL"
                    "uaxis" "[1 0 0 0] 0.25"
                    "vaxis" "[0 -1 0 0] 0.25"
                }
            }
            solid
            {
                "id" "3"
                side
                {
                    "id" "3"
                    "plane" "(0 0 0) (1 0 0) (1 1 0)"
                    "material" "WOOD"
                    "uaxis" "[1 0 0 0] 0.25"
                    "vaxis" "[0 -1 0 0] 0.25"
                }
            }
        }
        "#;

        let stream = lex(input);
        let result = World::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let world = result.unwrap();
        assert_eq!(world.solids.len(), 3);
        assert_eq!(world.solids[0].id, 1);
        assert_eq!(world.solids[1].id, 2);
        assert_eq!(world.solids[2].id, 3);
    }

    #[test]
    fn test_world_with_custom_properties() {
        let input = r#"
        world
        {
            "id" "1"
            "mapversion" "16"
            "classname" "worldspawn"
            "customkey1" "customvalue1"
            "customkey2" "customvalue2"
            "_light" "255 255 255 200"
        }
        "#;

        let stream = lex(input);
        let result = World::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let world = result.unwrap();
        assert_eq!(world.properties.get("customkey1"), Some(&"customvalue1"));
        assert_eq!(world.properties.get("customkey2"), Some(&"customvalue2"));
        assert_eq!(world.properties.get("_light"), Some(&"255 255 255 200"));
    }

    #[test]
    fn test_world_properties_out_of_order() {
        let input = r#"
        world
        {
            "skyname" "sky_day01_01"
            "classname" "worldspawn"
            "id" "1"
            "mapversion" "16"
            "detailmaterial" "detail/detailsprites"
        }
        "#;

        let stream = lex(input);
        let result = World::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let world = result.unwrap();
        assert_eq!(world.id, 1);
        assert_eq!(world.mapversion, 16);
        assert_eq!(world.classname, "worldspawn");
        assert_eq!(world.skyname, Some("sky_day01_01"));
        assert_eq!(world.detailmaterial, Some("detail/detailsprites"));
    }

    #[test]
    fn test_world_with_editor() {
        let input = r#"
        world
        {
            "id" "1"
            "mapversion" "16"
            "classname" "worldspawn"
            editor
            {
                "color" "255 0 0"
                "visgroupshown" "1"
                "visgroupautoshown" "1"
            }
        }
        "#;

        let stream = lex(input);
        let result = World::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let world = result.unwrap();
        assert!(world.editor.is_some());
        let editor = world.editor.unwrap();
        assert_eq!(editor.color.r, 255);
        assert_eq!(editor.color.g, 0);
        assert_eq!(editor.color.b, 0);
    }

    #[test]
    fn test_world_with_game_specific_properties() {
        let input = r#"
        world
        {
            "id" "1"
            "mapversion" "16"
            "classname" "worldspawn"
            "maxoccludeearea" "1000.5"
            "minoccluderarea" "500.25"
            "maxoccludeearea_csgo" "2000.75"
            "minoccluderarea_csgo" "750.5"
            "difficulty_level" "2"
            "hdr_level" "1"
        }
        "#;

        let stream = lex(input);
        let result = World::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let world = result.unwrap();
        assert_eq!(world.maxoccludeearea, Some(1000.5));
        assert_eq!(world.minoccluderarea, Some(500.25));
        assert_eq!(world.maxoccludeearea_csgo, Some(2000.75));
        assert_eq!(world.minoccluderarea_csgo, Some(750.5));
        assert_eq!(world.difficulty_level, Some(2));
        assert_eq!(world.hdr_level, Some(1));
    }

    #[test]
    fn test_world_with_entity_connections() {
        let input = r#"
        world
        {
            "id" "1"
            "mapversion" "16"
            "classname" "worldspawn"
            "targetname" "world_spawn"
            "target" "some_target"
        }
        "#;

        let stream = lex(input);
        let result = World::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let world = result.unwrap();
        assert_eq!(world.targetname, Some("world_spawn"));
        assert_eq!(world.target, Some("some_target"));
    }

    #[test]
    fn test_world_with_editor_properties() {
        let input = r#"
        world
        {
            "id" "1"
            "mapversion" "16"
            "classname" "worldspawn"
            "hidden" "1"
            "group" "5"
        }
        "#;

        let stream = lex(input);
        let result = World::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let world = result.unwrap();
        assert_eq!(world.hidden, Some(true));
        assert_eq!(world.group, Some(5));
    }

    #[test]
    fn test_world_empty_block() {
        let input = r#"
        world
        {
        }
        "#;

        let stream = lex(input);
        let result = World::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let world = result.unwrap();
        assert_eq!(world.id, 0);
        assert_eq!(world.mapversion, 0);
        assert_eq!(world.classname, "");
        assert_eq!(world.solids.len(), 0);
    }

    #[test]
    fn test_world_with_all_sound_properties() {
        let input = r#"
        world
        {
            "id" "1"
            "mapversion" "16"
            "classname" "worldspawn"
            "sounds" "1"
            "maxrange" "4096.5"
        }
        "#;

        let stream = lex(input);
        let result = World::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let world = result.unwrap();
        assert_eq!(world.sounds, Some(1));
        assert_eq!(world.maxrange, Some(4096.5));
    }

    #[test]
    fn test_world_complete_with_everything() {
        let input = r#"
        world
        {
            "id" "1"
            "mapversion" "16"
            "classname" "worldspawn"
            "detailmaterial" "detail/detailsprites"
            "detailvbsp" "detail.vbsp"
            "maxpropscreenwidth" "-1"
            "skyname" "sky_day01_01"
            solid
            {
                "id" "9"
                side
                {
                    "id" "1"
                    "plane" "(0 0 0) (1 0 0) (1 1 0)"
                    "material" "DEV/DEV_MEASUREGENERIC01B"
                    "uaxis" "[1 0 0 0] 0.25"
                    "vaxis" "[0 -1 0 0] 0.25"
                }
                editor
                {
                    "color" "0 111 152"
                    "visgroupshown" "1"
                    "visgroupautoshown" "1"
                }
            }
            editor
            {
                "color" "255 255 255"
                "visgroupshown" "1"
                "visgroupautoshown" "1"
            }
        }
        "#;

        let stream = lex(input);
        let result = World::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let world = result.unwrap();
        assert_eq!(world.id, 1);
        assert_eq!(world.classname, "worldspawn");
        assert_eq!(world.solids.len(), 1);
        assert!(world.editor.is_some());
        assert!(world.solids[0].editor.is_some());
    }

    #[test]
    fn test_world_invalid_block_name() {
        let input = r#"
        wrongname
        {
            "id" "1"
            "classname" "worldspawn"
        }
        "#;

        let stream = lex(input);
        let result = World::parse(stream);
        assert!(result.is_err(), "Parser should fail on invalid block name");
    }

    #[test]
    fn test_world_missing_closing_brace() {
        let input = r#"
        world
        {
            "id" "1"
            "classname" "worldspawn"
        "#;

        let stream = lex(input);
        let result = World::parse(stream);
        assert!(
            result.is_err(),
            "Parser should fail on missing closing brace"
        );
    }

    #[test]
    fn test_world_mixed_solids_and_properties() {
        let input = r#"
        world
        {
            "id" "1"
            solid
            {
                "id" "1"
                side
                {
                    "id" "1"
                    "plane" "(0 0 0) (1 0 0) (1 1 0)"
                    "material" "BRICK"
                    "uaxis" "[1 0 0 0] 0.25"
                    "vaxis" "[0 -1 0 0] 0.25"
                }
            }
            "classname" "worldspawn"
            "mapversion" "16"
            solid
            {
                "id" "2"
                side
                {
                    "id" "2"
                    "plane" "(0 0 0) (1 0 0) (1 1 0)"
                    "material" "METAL"
                    "uaxis" "[1 0 0 0] 0.25"
                    "vaxis" "[0 -1 0 0] 0.25"
                }
            }
            "skyname" "sky_day01_01"
        }
        "#;

        let stream = lex(input);
        let result = World::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let world = result.unwrap();
        assert_eq!(world.id, 1);
        assert_eq!(world.classname, "worldspawn");
        assert_eq!(world.solids.len(), 2);
        assert_eq!(world.skyname, Some("sky_day01_01"));
    }
}
