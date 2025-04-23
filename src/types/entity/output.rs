/// Represents an output connection between entities
#[derive(Debug)]
pub struct EntityOutput {
    pub target: String,
    pub input: String,
    pub parameter: Option<String>,
    pub delay: f32,
    pub times_to_fire: i32,
}
