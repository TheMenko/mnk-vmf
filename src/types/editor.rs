use super::Color;

#[derive(Debug)]
pub struct EditorData {
    pub color: Color,
    pub visgroupshown: bool,
    pub visgroupautoshown: bool,
    pub comments: Option<String>,
    pub logicalpos: Option<String>,
}
