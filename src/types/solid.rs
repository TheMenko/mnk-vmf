use super::EditorData;
use super::Side;

#[derive(Debug)]
pub struct Solid {
    pub id: u32,
    pub sides: Vec<Side>,
    pub editor: EditorData,
}
