use chumsky::input::Stream;
use memmap2::{Mmap, MmapOptions};
use std::path::Path;

use crate::error::VMFError;
use crate::parser::lexer::TokenIter;
use crate::parser::util::{stream, tokenize};
use crate::parser::{skip_unknown_block, InternalParser};
use crate::types::entity::*;
use crate::types::*;

use chumsky::primitive::choice;
use chumsky::IterParser;
use chumsky::Parser as ChumskyParser;

/// `VMFValue` holds types of all items from a VMF.
#[derive(Debug)]
pub enum VMFValue<'src> {
    VersionInfo(VersionInfo),
    VisGroups(Box<VisGroups<'src>>),
    ViewSettings(Box<ViewSettings>),
    World(Box<World<'src>>),
    Entity(Box<Entity<'src>>),
    Cameras(Box<Cameras<'src>>),
    Cordon(Box<Cordon>),
}

/// Memory-mapped VMF file.
/// Use `parse()` to get parsed data that borrows from this instance.
#[allow(clippy::upper_case_acronyms)]
pub struct VMF {
    mmap: Mmap,
}

impl VMF {
    /// Opens and memory-maps a VMF file.
    ///
    /// # Example
    /// ```ignore
    /// let vmf = VMF::open("test.vmf")?;
    /// let data = vmf.parse()?;
    /// // Use data..
    /// ```
    pub fn open(path: impl AsRef<Path>) -> Result<Self, VMFError> {
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        Ok(VMF { mmap })
    }

    /// Parse the VMF file and return the parsed data.
    /// The returned data borrows from this VMF instance.
    pub fn parse(&self) -> Result<Vec<VMFValue>, VMFError> {
        let src = self.as_str()?;
        parse_vmf_from_str(src)
    }

    /// Get the raw file content as a string slice.
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.mmap)
    }
}

/// Parse VMF data from a string slice.
/// Uses a sequential parser that handles all top-level blocks in order.
fn parse_vmf_from_str<'src>(src: &'src str) -> Result<Vec<VMFValue<'src>>, VMFError> {
    let token_iter = TokenIter::new(src).map(|tok| tok.expect("valid token"));
    let token_stream = Stream::from_iter(token_iter);

    let any_block = choice((
        VersionInfo::parser().map(VMFValue::VersionInfo),
        VisGroups::parser().map(|v| VMFValue::VisGroups(Box::new(v))),
        ViewSettings::parser().map(|v| VMFValue::ViewSettings(Box::new(v))),
        World::parser().map(|v| VMFValue::World(Box::new(v))),
        Entity::parser().map(|v| VMFValue::Entity(Box::new(v))),
        Cameras::parser().map(|v| VMFValue::Cameras(Box::new(v))),
        Cordon::parser().map(|v| VMFValue::Cordon(Box::new(v))),
    ));

    let any_block = any_block
        .map(|v| Some(v))
        .or(skip_unknown_block().map(|_| None));

    let all_blocks_parser = any_block.repeated().collect::<Vec<_>>();

    all_blocks_parser
        .parse(token_stream)
        .into_result()
        .map(|blocks| blocks.into_iter().flatten().collect())
        .map_err(|errors| {
            let error_msg = errors
                .into_iter()
                .map(|e| format!("{:?}", e.reason()))
                .collect::<Vec<_>>()
                .join("; ");
            VMFError::ParseError(format!("Failed to parse VMF: {}", error_msg))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_parser_test() {
        let vmf = VMF::open("test.vmf").expect("Failed to open VMF");
        let data = vmf.parse().expect("Failed to parse VMF");

        verify_parsed_data(&data);
    }

    fn verify_parsed_data(data: &[VMFValue]) {
        assert!(!data.is_empty(), "VMF data should not be empty");
        println!("Successfully parsed {} top-level blocks", data.len());

        for value in data {
            match value {
                VMFValue::VersionInfo(v) => {
                    println!(
                        "VersionInfo: editor v{}, build {}",
                        v.editor_version, v.editor_build
                    );
                    assert_eq!(v.editor_version, 400);
                    assert_eq!(v.editor_build, 6157);
                }
                VMFValue::VisGroups(_) => println!("VisGroups parsed"),
                VMFValue::ViewSettings(_) => println!("ViewSettings parsed"),
                VMFValue::World(w) => {
                    println!("World parsed with {} solids", w.solids.len());
                    assert!(w.id == 1);
                    assert!(w.classname == "worldspawn");
                }
                VMFValue::Entity(e) => println!("Entity: {:?}", e.classname),
                VMFValue::Cameras(c) => {
                    println!("Cameras: activecamera={}", c.activecamera);
                    assert_eq!(c.activecamera, -1);
                }
                VMFValue::Cordon(_) => println!("Cordon parsed"),
            }
        }
    }

    #[test]
    fn test_large_real_map() {
        let path = Path::new("Gm_RunDownTown.vmf");

        if !path.exists() {
            eprintln!("Skipping large map test - file not found");
            return;
        }

        println!("Parsing Gm_RunDownTown.vmf...");

        let start = std::time::Instant::now();
        let vmf = VMF::open(path).expect("Failed to open large VMF");
        let open_time = start.elapsed();
        println!("Open time: {:?}", open_time);

        let start = std::time::Instant::now();
        let data = vmf.parse().expect("Failed to parse large VMF");
        let parse_time = start.elapsed();
        println!("Parse time: {:?}", parse_time);

        println!("Total blocks parsed: {}", data.len());

        // Count different types
        let mut world_count = 0;
        let mut entity_count = 0;
        let mut solid_count = 0;

        for value in &data {
            match value {
                VMFValue::World(w) => {
                    world_count += 1;
                    solid_count += w.solids.len();
                }
                VMFValue::Entity(e) => {
                    entity_count += 1;
                    solid_count += e.solids.len();
                }
                _ => {}
            }
        }

        println!("Worlds: {}", world_count);
        println!("Entities: {}", entity_count);
        println!("Total solids: {}", solid_count);
        println!("Total time: {:?}", open_time + parse_time);
    }
}
