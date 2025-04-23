use crate::types::Color;

/// Represents a visgroup in the VMF file
/// Visgroups can be nested and contain properties like name, id, and color
#[derive(Debug, Clone, PartialEq)]
pub struct VisGroup {
    /// The name of the visgroup
    pub name: String,

    /// The unique identifier for the visgroup
    pub visgroupid: u32,

    /// The color of the visgroup in RGB format
    pub color: Color,

    /// Child visgroups contained within this visgroup
    pub children: Vec<VisGroup>,
}
