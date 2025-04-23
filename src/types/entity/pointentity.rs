use super::Entity;

/// Represents a point entity (light, prop, etc.)
#[derive(Debug)]
pub struct PointEntity {
    pub base: Entity,

    // Point entity specific properties
    pub scale: Option<f32>,
    pub fademindist: Option<f32>,
    pub fademaxdist: Option<f32>,
}
