use super::EditorData;
use super::Side;

#[derive(Debug)]
pub struct Solid<'a> {
    pub id: u32,
    pub sides: Vec<Side<'a>>,
    pub editor: EditorData,
}
