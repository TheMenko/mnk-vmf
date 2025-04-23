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

#[allow(clippy::upper_case_acronyms)]
pub struct VMF {
    data: Vec<VMFValue>,
}

impl VMF {
    pub fn new(path: &std::path::Path) -> Self {
        VMF { data: Vec::new() }
    }
    pub fn get_data(&self) -> &Vec<VMFValue> {
        &self.data
    }
}

pub fn load_vmf(path: &std::path::Path) {
    let vmf = VMF::new(path);
    println!("{:?}", vmf.get_data());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load() {
        load_vmf(std::path::Path::new("test.vmf"));
    }
}
