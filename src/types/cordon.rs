use super::point::Point3D;

/// Represents a cordon entity (tool used to block off parts of the map)
#[derive(Debug)]
pub struct Cordon {
    pub mins: Point3D, // Minimum bounds of the cordon box
    pub maxs: Point3D, // Maximum bounds of the cordon box
    pub active: bool,  // Whether the cordon is active
}
