use chumsky::{IterParser, Parser as ChumskyParser};

use crate::{
    impl_block_properties_parser,
    parser::{close_block, key_value_numeric, open_block, InternalParser, TokenError, TokenSource},
    types::{EditorData, Side},
    Parser,
};

/// Represents a solid brush in the VMF file
#[derive(Debug, Default, Clone)]
pub struct Solid<'src> {
    pub id: u32,
    pub sides: Vec<Side<'src>>,
    pub editor: Option<EditorData>,
}

/// Internal [`Solid`] Properties to be used in a parser impl
#[derive(Debug, Clone)]
enum SolidProperty<'src> {
    Id(u32),
    Side(Side<'src>),
    Editor(EditorData),
}

/// Public parser trait implementation that allows [`Solid`] to use ::parse(input) call.
impl<'src> Parser<'src> for Solid<'src> {}

/// A [`InternalParser`] implementation for [`Solid`].
///
/// usage: `let solid = Solid::parser().parse(input);`.
///
/// The format that is being parsed here is:
/// ```ignore
/// solid
/// {
///     "id" "9"
///     side
///     {
///         "id" "1"
///         "plane" "(-320 -320 0) (-320 320 0) (320 320 0)"
///         "material" "DEV/DEV_MEASUREGENERIC01B"
///         "uaxis" "[1 0 0 0] 0.25"
///         "vaxis" "[0 -1 0 0] 0.25"
///         "rotation" "0"
///         "lightmapscale" "16"
///         "smoothing_groups" "0"
///     }
///     side
///     {
///         ...
///     }
///     editor
///     {
///         "color" "0 111 152"
///         "visgroupshown" "1"
///         "visgroupautoshown" "1"
///     }
/// }
/// ```
impl<'src> InternalParser<'src> for Solid<'src> {
    fn parser<I>() -> impl ChumskyParser<'src, I, Self, TokenError<'src>>
    where
        I: TokenSource<'src>,
    {
        impl_block_properties_parser! {
            property_list: SolidProperty = {
                p_id = key_value_numeric("id") => SolidProperty::Id,
            }
        }

        // Nested block parsers
        let side_parser = Side::parser().map(SolidProperty::Side);
        let editor_parser = EditorData::parser().map(SolidProperty::Editor);

        // Combine all parsers
        let any_property = property_list.or(side_parser).or(editor_parser);

        open_block("solid")
            .ignore_then(any_property.repeated().collect::<Vec<SolidProperty>>())
            .then_ignore(close_block())
            .map(|properties: Vec<SolidProperty>| {
                let mut solid = Solid::default();
                for prop in properties {
                    match prop {
                        SolidProperty::Id(val) => solid.id = val,
                        SolidProperty::Side(val) => solid.sides.push(val),
                        SolidProperty::Editor(val) => solid.editor = Some(val),
                    }
                }
                solid
            })
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::lex;

    #[test]
    fn test_solid_complete() {
        let input = r#"
        solid
        {
            "id" "9"
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
            side
            {
                "id" "2"
                "plane" "(-320 320 -64) (-320 -320 -64) (320 -320 -64)"
                "material" "DEV/DEV_MEASUREGENERIC01B"
                "uaxis" "[1 0 0 0] 0.25"
                "vaxis" "[0 -1 0 0] 0.25"
                "rotation" "0"
                "lightmapscale" "16"
                "smoothing_groups" "0"
            }
            editor
            {
                "color" "0 111 152"
                "visgroupshown" "1"
                "visgroupautoshown" "1"
            }
        }
        "#;

        let stream = lex(input);
        let result = Solid::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let solid = result.unwrap();
        assert_eq!(solid.id, 9);
        assert_eq!(solid.sides.len(), 2);
        assert!(solid.editor.is_some());
        assert_eq!(solid.sides[0].id, 1);
        assert_eq!(solid.sides[1].id, 2);
    }

    #[test]
    fn test_solid_minimal() {
        let input = r#"
        solid
        {
            "id" "100"
            side
            {
                "id" "1"
                "plane" "(0 0 0) (1 0 0) (1 1 0)"
                "material" "DEV/DEV_MEASUREGENERIC01B"
                "uaxis" "[1 0 0 0] 0.25"
                "vaxis" "[0 -1 0 0] 0.25"
            }
        }
        "#;

        let stream = lex(input);
        let result = Solid::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let solid = result.unwrap();
        assert_eq!(solid.id, 100);
        assert_eq!(solid.sides.len(), 1);
        assert!(solid.editor.is_none());
    }

    #[test]
    fn test_solid_multiple_sides() {
        let input = r#"
        solid
        {
            "id" "50"
            side
            {
                "id" "1"
                "plane" "(0 0 0) (1 0 0) (1 1 0)"
                "material" "BRICK/BRICKWALL001A"
                "uaxis" "[1 0 0 0] 0.25"
                "vaxis" "[0 -1 0 0] 0.25"
            }
            side
            {
                "id" "2"
                "plane" "(0 0 0) (1 0 0) (1 1 0)"
                "material" "CONCRETE/CONCRETEWALL001"
                "uaxis" "[1 0 0 0] 0.25"
                "vaxis" "[0 -1 0 0] 0.25"
            }
            side
            {
                "id" "3"
                "plane" "(0 0 0) (1 0 0) (1 1 0)"
                "material" "METAL/METALWALL001"
                "uaxis" "[1 0 0 0] 0.25"
                "vaxis" "[0 -1 0 0] 0.25"
            }
            editor
            {
                "color" "255 128 64"
                "visgroupshown" "1"
                "visgroupautoshown" "1"
            }
        }
        "#;

        let stream = lex(input);
        let result = Solid::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let solid = result.unwrap();
        assert_eq!(solid.id, 50);
        assert_eq!(solid.sides.len(), 3);
        assert_eq!(solid.sides[0].material, "BRICK/BRICKWALL001A");
        assert_eq!(solid.sides[1].material, "CONCRETE/CONCRETEWALL001");
        assert_eq!(solid.sides[2].material, "METAL/METALWALL001");
    }

    #[test]
    fn test_solid_out_of_order() {
        let input = r#"
        solid
        {
            side
            {
                "id" "1"
                "plane" "(0 0 0) (1 0 0) (1 1 0)"
                "material" "DEV/DEV_MEASUREGENERIC01B"
                "uaxis" "[1 0 0 0] 0.25"
                "vaxis" "[0 -1 0 0] 0.25"
            }
            "id" "75"
            editor
            {
                "color" "0 255 0"
                "visgroupshown" "1"
                "visgroupautoshown" "1"
            }
        }
        "#;

        let stream = lex(input);
        let result = Solid::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let solid = result.unwrap();
        assert_eq!(solid.id, 75);
        assert_eq!(solid.sides.len(), 1);
        assert!(solid.editor.is_some());
    }

    #[test]
    fn test_solid_empty_block() {
        let input = r#"
        solid
        {
        }
        "#;

        let stream = lex(input);
        let result = Solid::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let solid = result.unwrap();
        assert_eq!(solid.id, 0);
        assert_eq!(solid.sides.len(), 0);
    }

    #[test]
    fn test_solid_no_editor() {
        let input = r#"
        solid
        {
            "id" "200"
            side
            {
                "id" "1"
                "plane" "(0 0 0) (1 0 0) (1 1 0)"
                "material" "DEV/DEV_MEASUREGENERIC01B"
                "uaxis" "[1 0 0 0] 0.25"
                "vaxis" "[0 -1 0 0] 0.25"
            }
        }
        "#;

        let stream = lex(input);
        let result = Solid::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let solid = result.unwrap();
        assert_eq!(solid.id, 200);
        assert_eq!(solid.sides.len(), 1);
        assert!(solid.editor.is_none());
    }

    #[test]
    fn test_solid_invalid_block_name() {
        let input = r#"
        wrongname
        {
            "id" "1"
        }
        "#;

        let stream = lex(input);
        let result = Solid::parse(stream);
        assert!(result.is_err(), "Parser should fail on invalid block name");
    }

    #[test]
    fn test_solid_missing_closing_brace() {
        let input = r#"
        solid
        {
            "id" "1"
            side
            {
                "id" "1"
                "plane" "(0 0 0) (1 0 0) (1 1 0)"
                "material" "DEV/DEV_MEASUREGENERIC01B"
                "uaxis" "[1 0 0 0] 0.25"
                "vaxis" "[0 -1 0 0] 0.25"
            }
        "#;

        let stream = lex(input);
        let result = Solid::parse(stream);
        assert!(
            result.is_err(),
            "Parser should fail on missing closing brace"
        );
    }
}
