use chumsky::{IterParser, Parser as ChumskyParser};

use crate::{
    parser::{
        close_block, key_value, key_value_boolean, key_value_numeric, open_block, InternalParser,
        TokenError, TokenSource,
    },
    Parser,
};

/// Macro to define individual property parsers and combine them with .or().
///
/// Usage:
/// ```ignore
/// impl_block_properties_parser! {
///     // Output variable name for the combined parser: Its type will be Box<dyn Parser<..., Output = $PropEnumType, ...>>
///     any_property_variable_name: YourPropertyEnumType = {
///         // var_name = parser_call_expr => enum_variant_constructor_or_mapper_fn
///         p_some_bool = some_parser_that_outputs_bool("some_key") => YourPropertyEnumType::SomeBool,
///         p_some_num  = some_parser_that_outputs_u32("num_key")  => YourPropertyEnumType::SomeNum,
///         // ...
///     }
/// }
/// ```
macro_rules! impl_block_properties_parser {
    (@build_or_chain $first_parser_var:ident) => {
        $first_parser_var
    };
    (@build_or_chain $first_parser_var:ident, $($rest_parser_vars:ident),+) => {
        $first_parser_var.or(impl_block_properties_parser!(@build_or_chain $($rest_parser_vars),+))
    };

    (
        $any_property_let_name:ident: $PropEnumType:ty = {
            $(
                $var_name:ident = $parser_call_expr:expr => $value_mapper_fn:expr
            ),+ $(,)? // Allow trailing comma
        }
    ) => {
        $(
            let $var_name = $parser_call_expr.map($value_mapper_fn);
        )+
        let $any_property_let_name =
            impl_block_properties_parser!(@build_or_chain $($var_name),+).boxed();
    };
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct ViewSettings {
    snap_to_grid: bool,
    show_grid: bool,
    show_logical_grid: bool,
    grid_spacing: u32,
    show_3d_grid: bool,
    hide_objects: bool,
    hide_walls: bool,
    hide_stripes: bool,
    hide_neighbors: bool,
    hide_detail: bool,
    show_brushes: bool,
    show_entities: bool,
    show_light_radius: bool,
    show_lighting_preview: bool,
    show_wireframe: bool,
}

/// Internal ViewSettings Properties to be used in a parser impl
#[derive(Debug, Clone)]
enum ViewSettingsProperty {
    SnapToGrid(bool),
    ShowGrid(bool),
    ShowLogicalGrid(bool),
    GridSpacing(u32),
    Show3DGrid(bool),
    HideObjects(bool),
    HideWalls(bool),
    HideStripes(bool),
    HideNeighbors(bool),
    HideDetail(bool),
    ShowBrushes(bool),
    ShowEntities(bool),
    ShowLightRadius(bool),
    ShowLightingPreview(bool),
    ShowWireframe(bool),
}

/// Public parser trait implementation that allows [`ViewSettings`] to use ::parse(input) call.
impl Parser<'_> for ViewSettings {}

/// A [`ViewSettings`] implementation for [`ViewSettings`].
/// Every key-value pair needs to be in order, like in the example bellow.
///
/// usage: `let view_settings = ViewSettings::parser().parse();`.
///
/// The format that is being parsed here is:
/// viewsettings
///{
/// "bSnapToGrid" "1"
/// "bShowGrid" "1"
/// "bShowLogicalGrid" "0"
/// "nGridSpacing" "64"
/// "bShow3DGrid" "0"
///}
impl<'src> InternalParser<'src> for ViewSettings {
    fn parser<I>() -> impl ChumskyParser<'src, I, Self, TokenError<'src>>
    where
        I: TokenSource<'src>,
    {
        impl_block_properties_parser! {
            property_list: ViewSettingsProperty = {
                p_snap_to_grid        = key_value_boolean("bSnapToGrid")          => ViewSettingsProperty::SnapToGrid,
                p_show_grid           = key_value_boolean("bShowGrid")            => ViewSettingsProperty::ShowGrid,
                p_show_logical_grid   = key_value_boolean("bShowLogicalGrid")     => ViewSettingsProperty::ShowLogicalGrid,
                p_grid_spacing        = key_value_numeric("nGridSpacing")         => ViewSettingsProperty::GridSpacing,
                p_show_3d_grid        = key_value_boolean("bShow3DGrid")          => ViewSettingsProperty::Show3DGrid,
                p_hide_objects        = key_value_boolean("bHideObjects")         => ViewSettingsProperty::HideObjects,
                p_hide_walls          = key_value_boolean("bHideWalls")           => ViewSettingsProperty::HideWalls,
                p_hide_stripes        = key_value_boolean("bHideStripes")         => ViewSettingsProperty::HideStripes,
                p_hide_neighbors      = key_value_boolean("bHideNeighbors")       => ViewSettingsProperty::HideNeighbors,
                p_hide_detail         = key_value_boolean("bHideDetail")          => ViewSettingsProperty::HideDetail,
                p_show_brushes        = key_value_boolean("bShowBrushes")         => ViewSettingsProperty::ShowBrushes,
                p_show_entities       = key_value_boolean("bShowEntities")        => ViewSettingsProperty::ShowEntities,
                p_show_light_radius   = key_value_boolean("bShowLightRadius")     => ViewSettingsProperty::ShowLightRadius,
                p_show_lighting_preview = key_value_boolean("bShowLightingPreview") => ViewSettingsProperty::ShowLightingPreview,
                p_show_wireframe      = key_value_boolean("bShowWireframe")       => ViewSettingsProperty::ShowWireframe,
            }
        }
        open_block("viewsettings")
            .ignore_then(
                property_list
                    .repeated()
                    .collect::<Vec<ViewSettingsProperty>>(),
            )
            .then_ignore(close_block())
            .map(|properties: Vec<ViewSettingsProperty>| {
                let mut settings = ViewSettings::default(); // Start with default values
                for prop in properties {
                    match prop {
                        ViewSettingsProperty::SnapToGrid(val) => settings.snap_to_grid = val,
                        ViewSettingsProperty::ShowGrid(val) => settings.show_grid = val,
                        ViewSettingsProperty::ShowLogicalGrid(val) => {
                            settings.show_logical_grid = val
                        }
                        ViewSettingsProperty::GridSpacing(val) => settings.grid_spacing = val,
                        ViewSettingsProperty::Show3DGrid(val) => settings.show_3d_grid = val,
                        ViewSettingsProperty::HideObjects(val) => settings.hide_objects = val,
                        ViewSettingsProperty::HideWalls(val) => settings.hide_walls = val,
                        ViewSettingsProperty::HideStripes(val) => settings.hide_stripes = val,
                        ViewSettingsProperty::HideNeighbors(val) => settings.hide_neighbors = val,
                        ViewSettingsProperty::HideDetail(val) => settings.hide_detail = val,
                        ViewSettingsProperty::ShowBrushes(val) => settings.show_brushes = val,
                        ViewSettingsProperty::ShowEntities(val) => settings.show_entities = val,
                        ViewSettingsProperty::ShowLightRadius(val) => {
                            settings.show_light_radius = val
                        }
                        ViewSettingsProperty::ShowLightingPreview(val) => {
                            settings.show_lighting_preview = val
                        }
                        ViewSettingsProperty::ShowWireframe(val) => settings.show_wireframe = val,
                    }
                }
                settings
            })
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use crate::{parser::lexer::Token, util::lex, Parser};

    use super::*;
    use chumsky::{error::RichReason, input::Stream, Parser as ChumskyParser};
    use logos::Logos as _;

    fn parse_viewsettings_str(input_str: &str) -> Result<ViewSettings, Vec<RichReason<Token<'_>>>> {
        ViewSettings::parse(lex(input_str))
    }

    #[test]
    fn test_viewsettings_full_valid_input() {
        let input = r#"
        viewsettings
        {
            "bSnapToGrid" "1"
            "bShowGrid" "1"
            "bShowLogicalGrid" "0"
            "nGridSpacing" "64"
            "bShow3DGrid" "1"
            "bHideObjects" "0"
            "bHideWalls" "1"
            "bHideStripes" "0"
            "bHideNeighbors" "1"
            "bHideDetail" "0"
            "bShowBrushes" "1"
            "bShowEntities" "0"
            "bShowLightRadius" "1"
            "bShowLightingPreview" "0"
            "bShowWireframe" "1"
        }"#;

        let result = parse_viewsettings_str(input);
        assert!(
            result.is_ok(),
            "Parser failed with errors: {:?}",
            result.err()
        );

        let settings = result.unwrap();
        assert_eq!(
            settings,
            ViewSettings {
                snap_to_grid: true,
                show_grid: true,
                show_logical_grid: false,
                grid_spacing: 64,
                show_3d_grid: true,
                hide_objects: false,
                hide_walls: true,
                hide_stripes: false,
                hide_neighbors: true,
                hide_detail: false,
                show_brushes: true,
                show_entities: false,
                show_light_radius: true,
                show_lighting_preview: false,
                show_wireframe: true,
            }
        );
    }

    #[test]
    fn test_viewsettings_partial_input_uses_defaults() {
        let input = r#"
        viewsettings
        {
            "bSnapToGrid" "1"
            "nGridSpacing" "32"
            "bShowWireframe" "0"
        }"#;

        let result = parse_viewsettings_str(input);
        assert!(
            result.is_ok(),
            "Parser failed with errors: {:?}",
            result.err()
        );

        let settings = result.unwrap();
        let mut expected = ViewSettings {
            snap_to_grid: true,
            grid_spacing: 32,
            show_wireframe: false,
            ..Default::default()
        };

        assert_eq!(settings, expected);
    }

    #[test]
    fn test_viewsettings_out_of_order() {
        let input = r#"
        viewsettings
        {
            "nGridSpacing" "16"
            "bShowGrid" "0"
            "bSnapToGrid" "1"
        }"#;

        let result = parse_viewsettings_str(input);
        assert!(
            result.is_ok(),
            "Parser failed with errors: {:?}",
            result.err()
        );

        let settings = result.unwrap();
        let mut expected = ViewSettings {
            grid_spacing: 16,
            show_grid: false,
            snap_to_grid: true,
            ..Default::default()
        };

        assert_eq!(settings, expected);
    }

    #[test]
    fn test_viewsettings_empty_block() {
        let input = r#"
        viewsettings
        {
        }"#;

        let result = parse_viewsettings_str(input);
        assert!(
            result.is_ok(),
            "Parser failed with errors: {:?}",
            result.err()
        );

        let settings = result.unwrap();
        assert_eq!(settings, ViewSettings::default());
    }

    #[test]
    fn test_viewsettings_unknown_key_causes_error() {
        // An unknown key, if not consumed by a more general rule,
        // will prevent subsequent tokens (like the closing '}') from being parsed correctly.
        let input = r#"
        viewsettings
        {
            "bSnapToGrid" "1"
            "bUnknownKey" "some_value" 
            "nGridSpacing" "64"
        }"#;

        let result = parse_viewsettings_str(input);
        assert!(
            result.is_err(),
            "Parser should fail on unknown key mid-list"
        );
        // You could inspect the error types/reasons if needed, e.g., expecting '}' but found "bUnknownKey".
    }

    #[test]
    fn test_viewsettings_unknown_key_at_end_still_errors_if_not_last() {
        // Similar to above, if "unknown" is not the very last property before "}"
        let input = r#"
        viewsettings
        {
            "bSnapToGrid" "1"
            "bShowGrid" "1"            
            "bUnknownKey" "some_value" 
        }"#; // Missing closing brace technically

        let result = parse_viewsettings_str(input);
        assert!(
            result.is_err(),
            "Parser should fail if block doesn't close properly after unknown key"
        );
    }

    #[test]
    fn test_viewsettings_invalid_value_type_for_numeric() {
        let input = r#"
        viewsettings
        {
            "nGridSpacing" "not_a_number"
        }"#;

        let result = parse_viewsettings_str(input);
        assert!(
            result.is_err(),
            "Parser should fail on invalid numeric value"
        );
        // Error would likely be "integer out of range" or similar from your `number` parser.
    }

    #[test]
    fn test_viewsettings_invalid_value_type_for_boolean() {
        let input = r#"
        viewsettings
        {
            "bSnapToGrid" "not_a_boolean" 
        }"#;

        let result = parse_viewsettings_str(input);
        assert!(
            result.is_err(),
            "Parser should fail on invalid boolean value"
        );
        // Error would be from `quoted_string("true").or(quoted_string("false"))` failing.
    }

    #[test]
    fn test_viewsettings_duplicate_keys_last_one_wins() {
        let input = r#"
        viewsettings
        {
            "bSnapToGrid" "0"
            "nGridSpacing" "32"
            "bSnapToGrid" "1" 
            "nGridSpacing" "64"
        }"#;

        let result = parse_viewsettings_str(input);
        assert!(
            result.is_ok(),
            "Parser failed with errors: {:?}",
            result.err()
        );

        let settings = result.unwrap();
        let mut expected = ViewSettings {
            snap_to_grid: true,
            grid_spacing: 64,
            ..Default::default()
        };

        assert_eq!(settings, expected);
    }

    #[test]
    fn test_viewsettings_malformed_block_missing_closing_brace() {
        let input = r#"
        viewsettings
        {
            "bSnapToGrid" "1"
        "#;

        let result = parse_viewsettings_str(input);
        assert!(
            result.is_err(),
            "Parser should fail on missing closing brace"
        );
    }

    #[test]
    fn test_viewsettings_malformed_block_missing_opening_brace() {
        let input = r#"
        viewsettings
            "bSnapToGrid" "1"
        }"#;

        let result = parse_viewsettings_str(input);
        assert!(
            result.is_err(),
            "Parser should fail on missing opening brace"
        );
    }

    #[test]
    fn test_viewsettings_invalid_block_name() {
        let input = r#"
        wrongblockname
        {
            "bSnapToGrid" "1"
        }"#;

        let result = parse_viewsettings_str(input);
        assert!(result.is_err(), "Parser should fail on invalid block name");
    }

    #[test]
    fn test_viewsettings_extra_content_after_closing_brace() {
        let input = r#"
        viewsettings
        {
            "bSnapToGrid" "1"
        }
        "extrastuff" "not good" 
        "#;
        let result = parse_viewsettings_str(input);
        assert!(
            result.is_err(),
            "Parser should fail. Errors: {:?}",
            result.err()
        );
    }
}
