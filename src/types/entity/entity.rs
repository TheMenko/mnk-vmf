use chumsky::{IterParser, Parser as ChumskyParser};
use std::collections::HashMap;

use crate::{
    Parser, impl_block_properties_parser,
    parser::{
        InternalParser, TokenError, TokenSource, any_quoted_string, close_block, key_value,
        key_value_boolean, key_value_numeric, open_block, quoted_string,
    },
    types::{
        Color, EditorData, Solid,
        entity::{EntityOutput, parse_output_entry},
        point::{Point3D, key_value_point3d},
    },
};

/// Represents a generic entity in a VMF file
#[derive(Debug, Default)]
pub struct Entity<'src> {
    pub id: u32,
    pub classname: &'src str,
    pub origin: Option<Point3D>,
    pub angles: Option<Point3D>,

    // Common entity properties
    pub targetname: Option<&'src str>,
    pub parentname: Option<&'src str>,
    pub target: Option<&'src str>,
    pub model: Option<&'src str>,
    pub skin: Option<u32>,
    pub spawnflags: Option<u32>,
    pub rendermode: Option<u32>,
    pub renderamt: Option<u32>,
    pub rendercolor: Option<Color>,
    pub disableshadows: Option<bool>,
    pub disablereceiveshadows: Option<bool>,
    pub startdisabled: Option<bool>,

    // Entity connections (outputs)
    pub outputs: Vec<EntityOutput<'src>>,

    // Custom key-value pairs for entity-specific properties
    pub properties: HashMap<&'src str, &'src str>,

    // Solids (for brush entities)
    pub solids: Vec<Solid<'src>>,

    // Editor data
    pub editor: Option<EditorData<'src>>,
}

/// Internal [`Entity`] Properties to be used in a parser impl
#[derive(Debug, Clone)]
enum EntityProperty<'src> {
    Id(u32),
    Classname(&'src str),
    Origin(Point3D),
    Angles(Point3D),
    Targetname(&'src str),
    Parentname(&'src str),
    Target(&'src str),
    Model(&'src str),
    Skin(u32),
    SpawnFlags(u32),
    RenderMode(u32),
    RenderAmt(u32),
    RenderColor(Color),
    DisableShadows(bool),
    DisableReceiveShadows(bool),
    StartDisabled(bool),
    Editor(EditorData<'src>),
    Connections(Vec<EntityOutput<'src>>),
    Solid(Solid<'src>),
    Custom(&'src str, &'src str),
}

/// Parser for the connections block containing entity outputs
fn parse_connections_block<'src, I>()
-> impl ChumskyParser<'src, I, Vec<EntityOutput<'src>>, TokenError<'src>>
where
    I: TokenSource<'src>,
{
    open_block("connections")
        .ignore_then(parse_output_entry().repeated().collect())
        .then_ignore(close_block())
}

/// Parse a color from rendercolor format "R G B"
fn parse_rendercolor<'src, I>() -> impl ChumskyParser<'src, I, Color, TokenError<'src>>
where
    I: TokenSource<'src>,
{
    use chumsky::error::Rich;

    quoted_string("rendercolor")
        .ignore_then(any_quoted_string())
        .try_map(|s: &str, span| {
            let mut parts = s.split_whitespace().map(str::parse::<u8>);
            let (r, g, b) = match (parts.next(), parts.next(), parts.next()) {
                (Some(Ok(r)), Some(Ok(g)), Some(Ok(b))) => (r, g, b),
                _ => return Err(Rich::custom(span, "invalid rendercolor components")),
            };

            if parts.next().is_some() {
                return Err(Rich::custom(span, "too many rendercolor components"));
            }

            Ok(Color { r, g, b })
        })
}

/// Public parser trait implementation that allows [`Entity`] to use ::parse(input) call.
impl<'src> Parser<'src> for Entity<'src> {}

/// A [`InternalParser`] implementation for [`Entity`].
///
/// usage: `let entity = Entity::parser().parse(input);`.
///
/// The format that is being parsed here is:
/// ```ignore
/// entity
/// {
///     "id" "7"
///     "classname" "info_player_start"
///     "angles" "0 90 0"
///     "origin" "0 -256 0"
///     editor
///     {
///         "color" "0 255 0"
///         "visgroupshown" "1"
///         "visgroupautoshown" "1"
///     }
/// }
/// ```
impl<'src> InternalParser<'src> for Entity<'src> {
    fn parser<I>() -> impl ChumskyParser<'src, I, Self, TokenError<'src>>
    where
        I: TokenSource<'src>,
    {
        // Known property parsers
        impl_block_properties_parser! {
            known_properties: EntityProperty = {
                p_id                        = key_value_numeric("id")                          => EntityProperty::Id,
                p_classname                 = key_value("classname")                           => |s: &str| EntityProperty::Classname(s),
                p_origin                    = key_value_point3d("origin")                      => EntityProperty::Origin,
                p_angles                    = key_value_point3d("angles")                      => EntityProperty::Angles,
                p_targetname                = key_value("targetname")                          => |s: &str| EntityProperty::Targetname(s),
                p_parentname                = key_value("parentname")                          => |s: &str| EntityProperty::Parentname(s),
                p_target                    = key_value("target")                              => |s: &str| EntityProperty::Target(s),
                p_model                     = key_value("model")                               => |s: &str| EntityProperty::Model(s),
                p_skin                      = key_value_numeric("skin")                        => EntityProperty::Skin,
                p_spawnflags                = key_value_numeric("spawnflags")                  => EntityProperty::SpawnFlags,
                p_rendermode                = key_value_numeric("rendermode")                  => EntityProperty::RenderMode,
                p_renderamt                 = key_value_numeric("renderamt")                   => EntityProperty::RenderAmt,
                p_rendercolor               = parse_rendercolor()                              => EntityProperty::RenderColor,
                p_disableshadows            = key_value_boolean("disableshadows")              => EntityProperty::DisableShadows,
                p_disablereceiveshadows     = key_value_boolean("disablereceiveshadows")       => EntityProperty::DisableReceiveShadows,
                p_startdisabled             = key_value_boolean("startdisabled")               => EntityProperty::StartDisabled,
            }
        }

        // Nested block parsers
        let editor_parser = EditorData::parser().map(EntityProperty::Editor);
        let connections_parser = parse_connections_block().map(EntityProperty::Connections);
        let solid_parser = Solid::parser().map(EntityProperty::Solid);

        // Custom property parser (catch-all for unknown properties)
        let custom_property = any_quoted_string()
            .then(any_quoted_string())
            .map(|(key, value): (&str, &str)| EntityProperty::Custom(key, value));

        // Combine all parsers
        let any_property = known_properties
            .or(editor_parser)
            .or(connections_parser)
            .or(solid_parser)
            .or(custom_property);

        open_block("entity")
            .ignore_then(any_property.repeated().collect::<Vec<EntityProperty>>())
            .then_ignore(close_block())
            .map(|properties: Vec<EntityProperty>| {
                let mut entity = Entity::default();
                for prop in properties {
                    match prop {
                        EntityProperty::Id(val) => entity.id = val,
                        EntityProperty::Classname(val) => entity.classname = val,
                        EntityProperty::Origin(val) => entity.origin = Some(val),
                        EntityProperty::Angles(val) => entity.angles = Some(val),
                        EntityProperty::Targetname(val) => entity.targetname = Some(val),
                        EntityProperty::Parentname(val) => entity.parentname = Some(val),
                        EntityProperty::Target(val) => entity.target = Some(val),
                        EntityProperty::Model(val) => entity.model = Some(val),
                        EntityProperty::Skin(val) => entity.skin = Some(val),
                        EntityProperty::SpawnFlags(val) => entity.spawnflags = Some(val),
                        EntityProperty::RenderMode(val) => entity.rendermode = Some(val),
                        EntityProperty::RenderAmt(val) => entity.renderamt = Some(val),
                        EntityProperty::RenderColor(val) => entity.rendercolor = Some(val),
                        EntityProperty::DisableShadows(val) => entity.disableshadows = Some(val),
                        EntityProperty::DisableReceiveShadows(val) => {
                            entity.disablereceiveshadows = Some(val)
                        }
                        EntityProperty::StartDisabled(val) => entity.startdisabled = Some(val),
                        EntityProperty::Editor(val) => entity.editor = Some(val),
                        EntityProperty::Connections(val) => entity.outputs = val,
                        EntityProperty::Solid(val) => entity.solids.push(val),
                        EntityProperty::Custom(key, value) => {
                            entity.properties.insert(key, value);
                        }
                    }
                }
                entity
            })
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::lex;

    #[test]
    fn test_entity_simple_point_entity() {
        let input = r#"
        entity
        {
            "id" "7"
            "classname" "info_player_start"
            "angles" "0 90 0"
            "origin" "0 -256 0"
            editor
            {
                "color" "0 255 0"
                "visgroupshown" "1"
                "visgroupautoshown" "1"
            }
        }
        "#;

        let stream = lex(input);
        let result = Entity::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let entity = result.unwrap();
        assert_eq!(entity.id, 7);
        assert_eq!(entity.classname, "info_player_start");
        assert!(entity.angles.is_some());
        assert_eq!(entity.angles.unwrap().y, 90.0);
        assert!(entity.origin.is_some());
        assert_eq!(entity.origin.unwrap().y, -256.0);
        assert!(entity.editor.is_some());
    }

    #[test]
    fn test_entity_with_custom_properties() {
        let input = r#"
        entity
        {
            "id" "85"
            "classname" "light"
            "_light" "255 255 255 400"
            "_lightHDR" "-1 -1 -1 1"
            "_lightscaleHDR" "1"
            "_quadratic_attn" "1"
            "origin" "-192 192 128"
            editor
            {
                "color" "220 30 220"
                "visgroupshown" "1"
                "visgroupautoshown" "1"
            }
        }
        "#;

        let stream = lex(input);
        let result = Entity::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let entity = result.unwrap();
        assert_eq!(entity.id, 85);
        assert_eq!(entity.classname, "light");
        assert_eq!(entity.properties.get("_light"), Some(&"255 255 255 400"));
        assert_eq!(entity.properties.get("_lightHDR"), Some(&"-1 -1 -1 1"));
        assert_eq!(entity.properties.get("_lightscaleHDR"), Some(&"1"));
        assert_eq!(entity.properties.get("_quadratic_attn"), Some(&"1"));
    }

    #[test]
    fn test_entity_with_connections() {
        let input = r#"
        entity
        {
            "id" "243"
            "classname" "func_button"
            "origin" "32 -217 48"
            connections
            {
                "OnIn" "motor*,TurnOn,,0,-1"
                "OnOut" "motor*,TurnOff,,0,-1"
            }
        }
        "#;

        let stream = lex(input);
        let result = Entity::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let entity = result.unwrap();
        assert_eq!(entity.id, 243);
        assert_eq!(entity.classname, "func_button");
        assert_eq!(entity.outputs.len(), 2);
        assert_eq!(entity.outputs[0].output_name, "OnIn");
        assert_eq!(entity.outputs[0].target, "motor*");
        assert_eq!(entity.outputs[0].input, "TurnOn");
        assert_eq!(entity.outputs[1].output_name, "OnOut");
        assert_eq!(entity.outputs[1].input, "TurnOff");
    }

    #[test]
    fn test_entity_with_render_properties() {
        let input = r#"
        entity
        {
            "id" "100"
            "classname" "prop_static"
            "origin" "0 0 0"
            "renderamt" "255"
            "rendercolor" "255 128 64"
            "rendermode" "0"
            "disableshadows" "1"
            "disablereceiveshadows" "0"
        }
        "#;

        let stream = lex(input);
        let result = Entity::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let entity = result.unwrap();
        assert_eq!(entity.renderamt, Some(255));
        assert!(entity.rendercolor.is_some());
        let color = entity.rendercolor.unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
        assert_eq!(entity.rendermode, Some(0));
        assert_eq!(entity.disableshadows, Some(true));
        assert_eq!(entity.disablereceiveshadows, Some(false));
    }

    #[test]
    fn test_entity_with_targetname() {
        let input = r#"
        entity
        {
            "id" "50"
            "classname" "func_door"
            "targetname" "main_door"
            "target" "door_trigger"
            "parentname" "door_parent"
            "origin" "0 0 0"
        }
        "#;

        let stream = lex(input);
        let result = Entity::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let entity = result.unwrap();
        assert_eq!(entity.targetname, Some("main_door"));
        assert_eq!(entity.target, Some("door_trigger"));
        assert_eq!(entity.parentname, Some("door_parent"));
    }

    #[test]
    fn test_entity_minimal() {
        let input = r#"
        entity
        {
            "id" "1"
            "classname" "worldspawn"
        }
        "#;

        let stream = lex(input);
        let result = Entity::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let entity = result.unwrap();
        assert_eq!(entity.id, 1);
        assert_eq!(entity.classname, "worldspawn");
        assert!(entity.origin.is_none());
        assert!(entity.angles.is_none());
    }

    #[test]
    fn test_entity_empty_block() {
        let input = r#"
        entity
        {
        }
        "#;

        let stream = lex(input);
        let result = Entity::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let entity = result.unwrap();
        assert_eq!(entity.id, 0);
        assert_eq!(entity.classname, "");
    }

    #[test]
    fn test_entity_properties_out_of_order() {
        let input = r#"
        entity
        {
            "origin" "100 200 300"
            "classname" "test_entity"
            "angles" "45 90 0"
            "id" "999"
        }
        "#;

        let stream = lex(input);
        let result = Entity::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let entity = result.unwrap();
        assert_eq!(entity.id, 999);
        assert_eq!(entity.classname, "test_entity");
    }

    #[test]
    fn test_entity_invalid_block_name() {
        let input = r#"
        wrongname
        {
            "id" "1"
            "classname" "test"
        }
        "#;

        let stream = lex(input);
        let result = Entity::parse(stream);
        assert!(result.is_err(), "Parser should fail on invalid block name");
    }

    #[test]
    fn test_entity_missing_closing_brace() {
        let input = r#"
        entity
        {
            "id" "1"
            "classname" "test"
        "#;

        let stream = lex(input);
        let result = Entity::parse(stream);
        assert!(
            result.is_err(),
            "Parser should fail on missing closing brace"
        );
    }

    #[test]
    fn test_entity_with_solid_brush_entity() {
        let input = r#"
        entity
        {
            "id" "243"
            "classname" "func_button"
            "origin" "32 -217 48"
            solid
            {
                "id" "187"
                side
                {
                    "id" "102"
                    "plane" "(26 -216 54) (38 -216 54) (38 -218 54)"
                    "material" "DEV/DEV_MEASUREGENERIC01B"
                    "uaxis" "[1 0 0 0] 0.25"
                    "vaxis" "[0 -1 0 0] 0.25"
                    "rotation" "0"
                    "lightmapscale" "16"
                    "smoothing_groups" "0"
                }
                editor
                {
                    "color" "0 255 0"
                    "visgroupshown" "1"
                    "visgroupautoshown" "1"
                }
            }
            editor
            {
                "color" "220 30 220"
                "visgroupshown" "1"
                "visgroupautoshown" "1"
            }
        }
        "#;

        let stream = lex(input);
        let result = Entity::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let entity = result.unwrap();
        assert_eq!(entity.id, 243);
        assert_eq!(entity.classname, "func_button");
        assert_eq!(entity.solids.len(), 1);
        assert_eq!(entity.solids[0].id, 187);
        assert_eq!(entity.solids[0].sides.len(), 1);
        assert!(entity.editor.is_some());
    }

    #[test]
    fn test_entity_with_multiple_solids() {
        let input = r#"
        entity
        {
            "id" "100"
            "classname" "func_door"
            "origin" "0 0 0"
            solid
            {
                "id" "1"
                side
                {
                    "id" "1"
                    "plane" "(0 0 0) (1 0 0) (1 1 0)"
                    "material" "METAL/METALDOOR001"
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
                    "material" "METAL/METALDOOR002"
                    "uaxis" "[1 0 0 0] 0.25"
                    "vaxis" "[0 -1 0 0] 0.25"
                }
            }
        }
        "#;

        let stream = lex(input);
        let result = Entity::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let entity = result.unwrap();
        assert_eq!(entity.id, 100);
        assert_eq!(entity.classname, "func_door");
        assert_eq!(entity.solids.len(), 2);
        assert_eq!(entity.solids[0].id, 1);
        assert_eq!(entity.solids[1].id, 2);
    }
}
