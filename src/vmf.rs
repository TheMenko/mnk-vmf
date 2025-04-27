use memmap2::{Mmap, MmapOptions};

use crate::types::entity::*;
use crate::types::*;

/// `VMFValue` holds types of all items from a VMF.
#[derive(Debug)]
pub enum VMFValue {
    VersionInfo,
    VisGroup(Box<VisGroup>),
    ViewSettings(Box<ViewSettings>),
    World(Box<World>),
    Entity(Box<Entity>),
    Camera(Box<Camera>),
    Cordon(Box<Cordon>),
}

/// Memory map backed VMF file.
/// `mmap` is supposed to be passed `.as_str()` to parsers.
#[allow(clippy::upper_case_acronyms)]
pub struct VMF {
    mmap: Mmap,
    data: Vec<VMFValue>,
}

impl VMF {
    pub fn new(path: &std::path::Path) -> Result<Self, std::io::Error> {
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        Ok(VMF {
            mmap,
            data: Vec::new(),
        })
    }
    pub fn get_data(&self) -> &Vec<VMFValue> {
        &self.data
    }

    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.mmap)
    }
}

#[cfg(test)]
mod tests {
    use chumsky::Parser as _;

    use crate::{parser::InternalParser, Parser};

    use super::*;

    #[test]
    fn load() {
        VMF::new(std::path::Path::new("test.vmf")).expect("Failed to open VMF file.");
    }

    #[test]
    fn parse_versioninfo_and_color_together() {
        let src = r#"versioninfo
{
    "editorversion"  "400"
    "editorbuild"    "6157"
    "mapversion"     "16"
    "formatversion"  "100"
    "prefab"         "0"
}
    "color"          "255 0 0"
"#;

        let parser = VersionInfo::parser().then(Color::parser());

        let result = parser.parse(src).unwrap();

        println!("{:?} {:?}", result.0, result.1);
    }
}
