use chumsky::{prelude::*, span::Span, Parser};

use crate::parser::{error::VMFParserError, key_value, whitespace, VMFParser};

/// `VersionInfo` holds the VMF Header information.
#[derive(Clone, Debug)]
pub struct VersionInfo {
    editor_version: u32,
    editor_build: u32,
    map_version: u8,
    format_version: u8,
    prefab: u32,
}

impl VersionInfo {
    pub fn new(
        version: u32,
        build: u32,
        map_version: u8,
        format_version: u8,
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

/*versioninfo*/
/*{*/
/*"editorversion" "400"*/
/*"editorbuild" "6157"*/
/*"mapversion" "16"*/
/*"formatversion" "100"*/
/*"prefab" "0"*/
/*}*/

impl VMFParser<VersionInfo> for VersionInfo {
    fn parser<'src>() -> impl Parser<'src, &'src str, Self> {
        let open_block = just("versioninfo")
            .ignore_then(whitespace())
            .ignore_then(just('{'))
            .ignore_then(whitespace());

        let close_block = whitespace().ignore_then(just('}'));

        open_block
            .ignore_then(
                key_value("editorversion")
                    .map(|ev| VersionInfo {
                        editor_version: ev.parse::<u32>().unwrap(),
                        editor_build: 0,
                        map_version: 0,
                        format_version: 0,
                        prefab: 0,
                    })
                    .then_ignore(whitespace())
                    .then(key_value("editorbuild"))
                    .map(|(info, eb)| VersionInfo {
                        editor_build: eb.parse::<u32>().unwrap(),
                        ..info
                    })
                    .then_ignore(whitespace())
                    .then(key_value("mapversion"))
                    .map(|(info, mv)| VersionInfo {
                        map_version: mv.parse::<u8>().unwrap(),
                        ..info
                    })
                    .then_ignore(whitespace())
                    .then(key_value("formatversion"))
                    .map(|(info, fv)| VersionInfo {
                        format_version: fv.parse::<u8>().unwrap(),
                        ..info
                    })
                    .then_ignore(whitespace())
                    .then(key_value("prefab"))
                    .map(|(info, prefab)| VersionInfo {
                        prefab: prefab.parse::<u32>().unwrap(),
                        ..info
                    }),
            )
            .then_ignore(whitespace())
            .then_ignore(close_block)
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
        println!("Errors: {:?}", missing_result.has_errors());
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
