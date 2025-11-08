use chumsky::{prelude::recursive, IterParser, Parser as ChumskyParser};

use crate::{
    impl_block_properties_parser,
    parser::{close_block, key_value_numeric, open_block, InternalParser, TokenError, TokenSource},
    types::EditorData,
    Parser,
};

#[derive(Debug, Default, Clone)]
pub struct Group<'src> {
    pub id: u32,
    pub editor: Option<EditorData<'src>>,
    pub groups: Vec<Group<'src>>,
}

#[derive(Debug, Clone)]
enum GroupProperty<'src> {
    Id(u32),
    Editor(EditorData<'src>),
    Child(Group<'src>),
}

/// Public parser trait implementation that allows [`Group`] to use ::parse(input) call.
impl<'src> Parser<'src> for Group<'src> {}

/// A [`InternalParser`] implementation for [`Group`].
///
/// usage: `let editor = Group::parser().parse(input);`.
///
/// The format that is being parsed here is:
/// ```ignore
/// group
/// 	{
/// 		"id" "772983"
/// 		editor
/// 		{
/// 			"color" "254 255 0"
/// 			"groupid" "772977"
/// 			"visgroupshown" "1"
/// 			"visgroupautoshown" "1"
/// 		}
/// 	}
///```
impl<'src> InternalParser<'src> for Group<'src> {
    fn parser<I>() -> impl ChumskyParser<'src, I, Self, TokenError<'src>>
    where
        I: TokenSource<'src>,
    {
        recursive(|group_parser| {
            impl_block_properties_parser! {
                property_list: GroupProperty = {
                    p_id     = key_value_numeric("id") => GroupProperty::Id,
                    p_editor = EditorData::parser()    => GroupProperty::Editor,
                    p_child  = group_parser.clone()    => GroupProperty::Child,
                }
            }

            open_block("group")
                .boxed()
                .ignore_then(property_list.repeated().collect::<Vec<GroupProperty>>())
                .then_ignore(close_block())
                .map(|properties: Vec<GroupProperty>| {
                    let mut group = Group::default();
                    for prop in properties {
                        match prop {
                            GroupProperty::Id(val) => group.id = val,
                            GroupProperty::Editor(val) => group.editor = Some(val),
                            GroupProperty::Child(val) => group.groups.push(val),
                        }
                    }
                    group
                })
                .boxed()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::lex;

    #[test]
    fn parse_simple_group() {
        let input = lex(r#"
            group
            {
                "id" "10"
                editor
                {
                    "color" "255 0 0"
                    "visgroupshown" "1"
                    "visgroupautoshown" "1"
                }
            }
        "#);

        let result = Group::parse(input);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());
        let group = result.unwrap();
        assert_eq!(group.id, 10);
        assert!(group.editor.is_some());
    }

    #[test]
    fn parse_nested_groups() {
        let input = lex(r#"
            group
            {
                "id" "100"
                group
                {
                    "id" "101"
                }
            }
        "#);

        let result = Group::parse(input);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());
        let group = result.unwrap();
        assert_eq!(group.id, 100);
        assert_eq!(group.groups.len(), 1);
        assert_eq!(group.groups[0].id, 101);
    }
}
