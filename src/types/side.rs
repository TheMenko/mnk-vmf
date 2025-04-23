use super::DispInfo;
use super::Point3D;

#[derive(Debug)]
pub struct Side {
    pub id: u32,
    pub plane: (Point3D, Point3D, Point3D), // Three points defining the plane
    pub material: String,
    pub uaxis: TextureAxis,
    pub vaxis: TextureAxis,
    pub rotation: f32,
    pub lightmapscale: u32,
    pub smoothing_groups: u32,
    pub dispinfo: Option<DispInfo>, // Displacement information for terrain
}
#[derive(Debug, Clone)]
pub struct TextureAxis {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub shift: f32,
    pub scale: f32,
}
