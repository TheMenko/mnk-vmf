use super::super::Color;
use super::super::EditorData;
use super::super::Point3D;
use super::super::Solid;
use super::EntityOutput;

/// Represents a generic entity in a VMF file
#[derive(Debug)]
pub struct Entity {
    pub id: u32,
    pub classname: String,
    pub origin: Option<Point3D>,
    pub angles: Option<Point3D>,

    // Common entity properties
    pub targetname: Option<String>,
    pub parentname: Option<String>,
    pub target: Option<String>,
    pub model: Option<String>,
    pub skin: Option<u32>,
    pub spawnflags: Option<u32>,
    pub rendermode: Option<u32>,
    pub renderamt: Option<u32>,
    pub rendercolor: Option<Color>,
    pub disableshadows: Option<bool>,
    pub disablereceiveshadows: Option<bool>,
    pub startdisabled: Option<bool>,

    // Entity connections (outputs)
    pub outputs: Vec<EntityOutput>,

    // Custom key-value pairs for entity-specific properties
    pub properties: std::collections::HashMap<String, String>,

    // Solids (for brush entities)
    pub solids: Vec<Solid>,

    // Editor data
    pub editor: EditorData,
}
