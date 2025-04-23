use super::Point3D;

/// Represents a displacement vertex
#[derive(Debug, Clone)]
pub struct DispVertex {
    pub position: Point3D,
    pub normal: Point3D,
    pub distance: f32,
    pub alpha: f32,
}

/// Represents a displacement triangle
#[derive(Debug, Clone)]
pub struct DispTri {
    pub indices: [u32; 3], // Indices into the vertex array
}

/// Represents displacement information for terrain
#[derive(Debug)]
pub struct DispInfo {
    pub power: u32,              // Power of 2 determining grid size (2^power + 1)
    pub start_position: Point3D, // Starting position of the displacement
    pub elevation: f32,          // Base height offset
    pub subdiv: bool,            // Whether to use subdivision

    // Normals and distances
    pub normals: Vec<Point3D>,
    pub distances: Vec<f32>,

    // Offsets (x,y,z) for each vertex
    pub offsets: Vec<Point3D>,

    // Offset normals
    pub offset_normals: Vec<Point3D>,

    // Alpha values (transparency/blending)
    pub alphas: Vec<f32>,

    // Triangle tags for collision
    pub triangle_tags: Vec<u32>,

    // Allowed vertex positions
    pub allowed_verts: Vec<u32>,
    pub flags: u32,
}
