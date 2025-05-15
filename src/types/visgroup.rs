use chumsky::{
    prelude::{just, recursive},
    IterParser, Parser as ChumskyParser,
};

use crate::{
    parser::{
        any_quoted_string, close_block, lexer, number, open_block, quoted_string, InternalParser,
        TokenError, TokenSource,
    },
    types::Color,
    Parser,
};

/// Represents a visgroup in the VMF file
/// Visgroups can be nested and contain properties like name, id, and color
#[derive(Debug, Clone, PartialEq)]
pub struct VisGroup<'a> {
    /// The name of the visgroup
    name: &'a str,

    /// The unique identifier for the visgroup
    visgroupid: u32,

    /// The color of the visgroup in RGB format
    color: Color,

    /// Child visgroups contained within this visgroup
    children: Vec<VisGroup<'a>>,
}

impl<'a> VisGroup<'a> {
    /// Crates a new [`VisGroup`] instance.
    pub fn new(
        name: &'a str,
        visgroupid: u32,
        color: Color,
        children: Vec<VisGroup<'a>>,
    ) -> VisGroup<'a> {
        VisGroup {
            name,
            visgroupid,
            color,
            children,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VisGroups<'a>(Vec<VisGroup<'a>>);

impl<'a> VisGroups<'a> {
    pub fn new(visgroups: Vec<VisGroup<'a>>) -> VisGroups<'a> {
        Self(visgroups)
    }
}

/// Public parser trait implementation that allows [`VisGroups`] to use ::parse(input) call.
impl<'src> Parser<'src> for VisGroups<'src> {}

/// A [`InternalParser`] implementation for [`VisGroups`].
/// Every key-value pair needs to be in order, like in the example bellow.
///
/// usage:
/// ```ignore
///     let visgroups = VisGroups::parser().parse();
/// ```
///
/// The format that is being parsed here is:
/// ```ignore
/// visgroups
/// {
///  
///     visgroup
///     {
///        ...
///     }
///
///     visgroup
///     {
///         ...
///         visgroup
///         {
///             ...
///         }
///     }
/// }
/// ```
impl<'src> InternalParser<'src> for VisGroups<'src> {
    fn parser<I>() -> impl ChumskyParser<'src, I, Self, TokenError<'src>>
    where
        I: TokenSource<'src>,
    {
        open_block("visgroups")
            .ignore_then(VisGroup::parser::<I>().repeated().collect())
            .then_ignore(close_block())
            .map(VisGroups::new)
    }
}

/// Public parser trait implementation that allows [`VisGroup`] to use ::parse(input) call.
impl<'src> Parser<'src> for VisGroup<'src> {}

/// A [`InternalParser`] implementation for [`VisGroup`].
/// Every key-value pair needs to be in order, like in the example bellow.
///
/// usage: `let visgroup = VisGroup::parser().parse();`.
///
/// The format that is being parsed here is:
/// ```ignore
/// visgroup
/// {
///     "name" "Tree_1"
///     "visgroupid" "5"
///     "color" "65 45 0"
/// }
/// ```
impl<'src> InternalParser<'src> for VisGroup<'src> {
    fn parser<I>() -> impl ChumskyParser<'src, I, Self, TokenError<'src>>
    where
        I: TokenSource<'src>,
    {
        recursive(|vis_group| {
            open_block("visgroup")
                .boxed()
                .ignore_then(
                    quoted_string("name")
                        .boxed()
                        .ignore_then(any_quoted_string().boxed())
                        .then_ignore(quoted_string("visgroupid").boxed())
                        .then(number::<u32, I>().boxed())
                        .then(Color::parser::<I>().boxed())
                        .then(vis_group.repeated().collect().boxed()),
                )
                .then_ignore(close_block().boxed())
                .map(|(((name, id), color), children)| VisGroup::new(name, id, color, children))
        })
    }Viewsettings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        parser::{util::lex, TokenSource},
        Parser,
    };

    #[test]
    fn test_single_visgroup() {
        let input = lex(r#"
            visgroup {
                "name" "Tree_1"
                "visgroupid" "5"
                "color" "65 45 0"
            }
        "#);

        let parsed = VisGroup::parse(input).unwrap();
        assert_eq!(parsed.name, "Tree_1");
        assert_eq!(parsed.visgroupid, 5);
        assert_eq!(parsed.color.r, 65);
        assert_eq!(parsed.color.g, 45);
        assert_eq!(parsed.color.b, 0);
        assert!(parsed.children.is_empty());
    }

    #[test]
    fn test_nested_visgroup() {
        let input = lex(r#"
            visgroup {
                "name" "Parent"
                "visgroupid" "1"
                "color" "10 20 30"
                visgroup {
                    "name" "Child"
                    "visgroupid" "2"
                    "color" "100 100 100"
                }
            }
        "#);

        let parsed = VisGroup::parse(input).unwrap();
        assert_eq!(parsed.name, "Parent");
        assert_eq!(parsed.children.len(), 1);
        assert_eq!(parsed.children[0].name, "Child");
        assert_eq!(parsed.children[0].color.r, 100);
    }

    #[test]
    fn test_visgroups_block() {
        let input = lex(r#"
            visgroups {
                visgroup {
                    "name" "One"
                    "visgroupid" "11"
                    "color" "11 22 33"
                }
                visgroup {
                    "name" "Two"
                    "visgroupid" "12"
                    "color" "44 55 66"
                }
            }
        "#);

        let parsed = VisGroups::parse(input).unwrap();
        assert_eq!(parsed.0.len(), 2);
        assert_eq!(parsed.0[0].name, "One");
        assert_eq!(parsed.0[1].name, "Two");
    }

    #[test]
    fn test_empty_visgroups() {
        let input = lex(r#"
            visgroups {
            }
        "#);

        let parsed = VisGroups::parse(input).unwrap();
        assert_eq!(parsed.0.len(), 0);
    }
}
