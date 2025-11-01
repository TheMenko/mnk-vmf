use chumsky::{IterParser, Parser as ChumskyParser};

use crate::{
    impl_block_properties_parser,
    parser::{
        any_quoted_string, close_block, key_value, key_value_boolean, open_block, InternalParser,
        TokenError, TokenSource,
    },
    types::Color,
    Parser,
};

/// Represents editor-specific data for entities and brushes
#[derive(Default, Debug, Clone)]
pub struct EditorData<'src> {
    pub color: Color,
    pub visgroupshown: bool,
    pub visgroupautoshown: bool,
    pub comments: Option<&'src str>,
    pub logicalpos: Option<&'src str>,
}

/// Internal [`EditorData`] Properties to be used in a parser impl
#[derive(Debug, Clone)]
enum EditorDataProperty<'src> {
    Color(Color),
    VisGroupShown(bool),
    VisGroupAutoShown(bool),
    Comments(&'src str),
    LogicalPos(&'src str),
}

/// Public parser trait implementation that allows [`EditorData`] to use ::parse(input) call.
impl<'src> Parser<'src> for EditorData<'src> {}

/// A [`InternalParser`] implementation for [`EditorData`].
///
/// usage: `let editor = EditorData::parser().parse(input);`.
///
/// The format that is being parsed here is:
/// ```ignore
/// editor
/// {
///     "color" "0 111 152"
///     "visgroupshown" "1"
///     "visgroupautoshown" "1"
///     "logicalpos" "[0 10000]"
///     "comments" "This is a comment"
/// }
/// ```
impl<'src> InternalParser<'src> for EditorData<'src> {
    fn parser<I>() -> impl ChumskyParser<'src, I, Self, TokenError<'src>>
    where
        I: TokenSource<'src>,
    {
        impl_block_properties_parser! {
            property_list: EditorDataProperty = {
                p_color                = Color::parser()                       => EditorDataProperty::Color,
                p_visgroupshown        = key_value_boolean("visgroupshown")    => EditorDataProperty::VisGroupShown,
                p_visgroupautoshown    = key_value_boolean("visgroupautoshown") => EditorDataProperty::VisGroupAutoShown,
                p_comments             = key_value("comments")                 => |s: &str| EditorDataProperty::Comments(s),
                p_logicalpos           = key_value("logicalpos")               => |s: &str| EditorDataProperty::LogicalPos(s),
            }
        }

        open_block("editor")
            .ignore_then(
                property_list
                    .repeated()
                    .collect::<Vec<EditorDataProperty>>(),
            )
            .then_ignore(close_block())
            .map(|properties: Vec<EditorDataProperty>| {
                let mut editor = EditorData::default();
                for prop in properties {
                    match prop {
                        EditorDataProperty::Color(val) => editor.color = val,
                        EditorDataProperty::VisGroupShown(val) => editor.visgroupshown = val,
                        EditorDataProperty::VisGroupAutoShown(val) => {
                            editor.visgroupautoshown = val
                        }
                        EditorDataProperty::Comments(val) => editor.comments = Some(val),
                        EditorDataProperty::LogicalPos(val) => editor.logicalpos = Some(val),
                    }
                }
                editor
            })
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::lex;

    #[test]
    fn test_editor_complete_valid() {
        let input = r#"
        editor
        {
            "color" "0 111 152"
            "visgroupshown" "1"
            "visgroupautoshown" "1"
            "logicalpos" "[0 10000]"
            "comments" "Test comment"
        }
        "#;

        let stream = lex(input);
        let result = EditorData::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let editor = result.unwrap();
        assert_eq!(editor.color.r, 0);
        assert_eq!(editor.color.g, 111);
        assert_eq!(editor.color.b, 152);
        assert_eq!(editor.visgroupshown, true);
        assert_eq!(editor.visgroupautoshown, true);
        assert_eq!(editor.logicalpos, Some("[0 10000]"));
        assert_eq!(editor.comments, Some("Test comment"));
    }

    #[test]
    fn test_editor_minimal() {
        let input = r#"
        editor
        {
            "color" "255 0 0"
            "visgroupshown" "1"
            "visgroupautoshown" "1"
        }
        "#;

        let stream = lex(input);
        let result = EditorData::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let editor = result.unwrap();
        assert_eq!(editor.color.r, 255);
        assert_eq!(editor.color.g, 0);
        assert_eq!(editor.color.b, 0);
        assert_eq!(editor.visgroupshown, true);
        assert_eq!(editor.visgroupautoshown, true);
        assert_eq!(editor.logicalpos, None);
        assert_eq!(editor.comments, None);
    }

    #[test]
    fn test_editor_properties_out_of_order() {
        let input = r#"
        editor
        {
            "logicalpos" "[0 5000]"
            "visgroupautoshown" "0"
            "color" "100 200 50"
            "comments" "Out of order test"
            "visgroupshown" "0"
        }
        "#;

        let stream = lex(input);
        let result = EditorData::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let editor = result.unwrap();
        assert_eq!(editor.color.r, 100);
        assert_eq!(editor.color.g, 200);
        assert_eq!(editor.color.b, 50);
        assert_eq!(editor.visgroupshown, false);
        assert_eq!(editor.visgroupautoshown, false);
        assert_eq!(editor.logicalpos, Some("[0 5000]"));
        assert_eq!(editor.comments, Some("Out of order test"));
    }

    #[test]
    fn test_editor_with_logicalpos_only() {
        let input = r#"
        editor
        {
            "color" "128 128 128"
            "visgroupshown" "1"
            "visgroupautoshown" "1"
            "logicalpos" "[0 0]"
        }
        "#;

        let stream = lex(input);
        let result = EditorData::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let editor = result.unwrap();
        assert_eq!(editor.logicalpos, Some("[0 0]"));
        assert_eq!(editor.comments, None);
    }

    #[test]
    fn test_editor_with_comments_only() {
        let input = r#"
        editor
        {
            "color" "64 64 64"
            "visgroupshown" "1"
            "visgroupautoshown" "1"
            "comments" "This brush needs work"
        }
        "#;

        let stream = lex(input);
        let result = EditorData::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let editor = result.unwrap();
        assert_eq!(editor.comments, Some("This brush needs work"));
        assert_eq!(editor.logicalpos, None);
    }

    #[test]
    fn test_editor_empty_block() {
        let input = r#"
        editor
        {
        }
        "#;

        let stream = lex(input);
        let result = EditorData::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let editor = result.unwrap();
        let default = EditorData::default();
        assert_eq!(editor.color.r, default.color.r);
        assert_eq!(editor.visgroupshown, default.visgroupshown);
        assert_eq!(editor.comments, None);
    }

    #[test]
    fn test_editor_invalid_color() {
        let input = r#"
        editor
        {
            "color" "invalid color"
            "visgroupshown" "1"
            "visgroupautoshown" "1"
        }
        "#;

        let stream = lex(input);
        let result = EditorData::parse(stream);
        assert!(result.is_err(), "Parser should fail on invalid color");
    }

    #[test]
    fn test_editor_invalid_visgroupshown() {
        let input = r#"
        editor
        {
            "color" "255 255 255"
            "visgroupshown" "not_a_bool"
            "visgroupautoshown" "1"
        }
        "#;

        let stream = lex(input);
        let result = EditorData::parse(stream);
        assert!(
            result.is_err(),
            "Parser should fail on invalid visgroupshown"
        );
    }

    #[test]
    fn test_editor_invalid_block_name() {
        let input = r#"
        wrongname
        {
            "color" "255 255 255"
            "visgroupshown" "1"
            "visgroupautoshown" "1"
        }
        "#;

        let stream = lex(input);
        let result = EditorData::parse(stream);
        assert!(result.is_err(), "Parser should fail on invalid block name");
    }

    #[test]
    fn test_editor_missing_closing_brace() {
        let input = r#"
        editor
        {
            "color" "255 255 255"
            "visgroupshown" "1"
            "visgroupautoshown" "1"
        "#;

        let stream = lex(input);
        let result = EditorData::parse(stream);
        assert!(
            result.is_err(),
            "Parser should fail on missing closing brace"
        );
    }

    #[test]
    fn test_editor_multiline_comments() {
        let input = r#"
        editor
        {
            "color" "50 100 150"
            "visgroupshown" "1"
            "visgroupautoshown" "1"
            "comments" "This is a very long comment that describes the purpose of this brush in detail"
        }
        "#;

        let stream = lex(input);
        let result = EditorData::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let editor = result.unwrap();
        assert_eq!(
            editor.comments,
            Some("This is a very long comment that describes the purpose of this brush in detail")
        );
    }

    #[test]
    fn test_editor_duplicate_properties_last_wins() {
        let input = r#"
        editor
        {
            "color" "100 100 100"
            "visgroupshown" "0"
            "visgroupshown" "1"
            "color" "200 200 200"
        }
        "#;

        let stream = lex(input);
        let result = EditorData::parse(stream);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        let editor = result.unwrap();
        assert_eq!(editor.color.r, 200);
        assert_eq!(editor.color.g, 200);
        assert_eq!(editor.color.b, 200);
        assert_eq!(editor.visgroupshown, true);
    }
}
