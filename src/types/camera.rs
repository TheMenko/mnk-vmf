use super::entity::PointEntity;
use super::Point3D;

#[derive(Debug)]
pub struct Cameras {
    activecamera: String,
    camera: Vec<Camera>,
}

/// Represents a point_viewcontrol (camera) entity
#[derive(Debug)]
pub struct Camera {
    pub base: PointEntity,

    // Camera specific properties
    pub angles: Point3D,
    pub targetname: String,
    pub spawnflags: Option<u32>,
    pub wait: Option<f32>,
    pub acceleration: Option<f32>,
    pub deceleration: Option<f32>,
    pub speed: Option<f32>,
    pub fov: Option<f32>,
    pub fov_rate: Option<f32>,
    pub use_screen_aspect_ratio: Option<bool>,
    pub interp_time: Option<f32>,
}
