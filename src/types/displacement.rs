use super::point::Point3D;

/// Represents a displacement vertex
#[derive(Debug, Clone)]
pub struct DispVertex {
    position: Point3D,
    normal: Point3D,
    distance: f32,
    alpha: f32,
}

/// Represents a displacement triangle
#[derive(Debug, Clone)]
pub struct DispTri {
    indices: [u32; 3],
}

/// Represents displacement information for terrain
#[derive(Debug)]
pub struct DispInfo {
    power: u32,              // Power of 2 determining grid size (2^power + 1)
    start_position: Point3D, // Starting position of the displacement
    elevation: f32,          // Base height offset
    subdiv: bool,            // Whether to use subdivision

    // Normals and distances
    normals: Vec<Point3D>,
    distances: Vec<f32>,

    // Offsets (x,y,z) for each vertex
    offsets: Vec<Point3D>,

    // Offset normals
    offset_normals: Vec<Point3D>,

    // Alpha values (transparency/blending)
    alphas: Vec<f32>,

    // Triangle tags for collision
    triangle_tags: Vec<u32>,

    // Allowed vertex positions
    allowed_verts: Vec<u32>,
    flags: u32,
}
