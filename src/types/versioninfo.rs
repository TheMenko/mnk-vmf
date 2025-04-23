use chumsky::{prelude::*, span::Span, Parser};

use crate::parser::{
    close_block, error::VMFParserError, key_value, open_block, whitespace, VMFParser,
};

/// `VersionInfo` holds the VMF Header information.
#[derive(Clone, Debug)]
pub struct VersionInfo {
    editor_version: u32,
    editor_build: u32,
    map_version: u16,
    format_version: u16,
    prefab: u32,
}

impl VersionInfo {
    pub fn new(
        version: u32,
        build: u32,
        map_version: u16,
        format_version: u16,
        prefab: u32,
    ) -> VersionInfo {
        Self {
            editor_version: version,
            editor_build: build,
            map_version,
            format_version,
            prefab,
        }
    }
}

/// A `VMFParser` implementation for VersionInfo.
/// Every key-value pair needs to be in order, like in the example bellow.
///
/// usage: `let version_info = VersionInfo::parser().parse();`.
///
/// The format that is being parsed here is:
/// versioninfo
/// {
/// "editorversion" "400"
/// "editorbuild" "6157"
/// "mapversion" "16"
/// "formatversion" "100"
/// "prefab" "0"
/// }
impl VMFParser<VersionInfo> for VersionInfo {
    fn parser<'src>() -> impl Parser<'src, &'src str, Self, extra::Err<Rich<'src, char>>> {
        open_block("versioninfo")
            .ignore_then(
                key_value("editorversion")
                    .try_map(|ev, span| {
                        Ok(VersionInfo {
                            editor_version: ev.parse::<u32>().map_err(|e| {
                                Rich::custom(span, "Could not parse a u32 value for editor_version")
                            })?,
                            editor_build: 0,
                            map_version: 0,
                            format_version: 0,
                            prefab: 0,
                        })
                    })
                    .then_ignore(whitespace())
                    .then(key_value("editorbuild"))
                    .try_map(|(info, eb), span| {
                        Ok(VersionInfo {
                            editor_build: eb.parse::<u32>().map_err(|e| {
                                Rich::custom(span, "Could not parse a u32 value for editor_build")
                            })?,
                            ..info
                        })
                    })
                    .then_ignore(whitespace())
                    .then(key_value("mapversion"))
                    .try_map(|(info, mv), span| {
                        Ok(VersionInfo {
                            map_version: mv.parse::<u16>().map_err(|e| {
                                Rich::custom(span, "Could not parse a u8 value for map_version")
                            })?,
                            ..info
                        })
                    })
                    .then_ignore(whitespace())
                    .then(key_value("formatversion"))
                    .try_map(|(info, fv), span| {
                        Ok(VersionInfo {
                            format_version: fv.parse::<u16>().map_err(|e| {
                                Rich::custom(span, "Could not parse a u8 value for format_version")
                            })?,
                            ..info
                        })
                    })
                    .then_ignore(whitespace())
                    .then(key_value("prefab"))
                    .try_map(|(info, prefab), span| {
                        Ok(VersionInfo {
                            prefab: prefab.parse::<u32>().map_err(|e| {
                                Rich::custom(span, "Could not parse a u32 value for prefab")
                            })?,
                            ..info
                        })
                    })
                    .then_ignore(whitespace())
                    .then_ignore(close_block()),
            )
            .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chumsky::Parser;

    #[test]
    fn test_version_info_parser() {
        // Valid input
        let input = r#"versioninfo
                    {
                        "editorversion" "400"
                        "editorbuild" "6157"
                        "mapversion" "16"
                        "formatversion" "100"
                        "prefab" "0"
                    }"#;

        let result = VersionInfo::parser().parse(input);
        assert!(
            !result.has_errors(),
            "Parser failed with error: {:?}",
            result.errors().collect::<Vec<_>>()
        );

        let version_info = result.unwrap();
        assert_eq!(version_info.editor_version, 400);
        assert_eq!(version_info.editor_build, 6157);
        assert_eq!(version_info.map_version, 16);
        assert_eq!(version_info.format_version, 100);
        assert_eq!(version_info.prefab, 0);

        // Test with different whitespace patterns
        let compact_input = r#"versioninfo{"editorversion""500""editorbuild""7000""mapversion""20""formatversion""110""prefab""1"}"#;
        let compact_result = VersionInfo::parser().parse(compact_input);
        assert!(
            !compact_result.has_errors(),
            "Compact parser failed with error: {:?}",
            compact_result.errors().collect::<Vec<_>>()
        );

        // Test with invalid input - missing field
        let missing_field = r#"versioninfo
                                    {
                                        "editorversion" "400"
                                        "editorbuild" "6157"
                                        "mapversion" "16"
                                        "prefab" "0"
                                    }"#; // Missing formatversion

        let missing_result = VersionInfo::parser().parse(missing_field);
        assert!(
            missing_result.has_errors(),
            "Parser should fail on missing field"
        );

        // Test with invalid input - invalid number format
        let invalid_format = r#"versioninfo
                                    {
                                        "editorversion" "400"
                                        "editorbuild" "invalid"
                                        "mapversion" "16"
                                        "formatversion" "100"
                                        "prefab" "0"
                                    }"#;

        let invalid_result = VersionInfo::parser().parse(invalid_format);
        assert!(
            invalid_result.has_errors(),
            "Parser should fail on invalid number format"
        );
    }
}
