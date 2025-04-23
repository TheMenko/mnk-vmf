use super::EditorData;
use super::Solid;

#[derive(Debug)]
pub struct World {
    pub id: u32,
    pub mapversion: u32,
    pub classname: String,
    pub detailmaterial: Option<String>,
    pub detailvbsp: Option<String>,
    pub maxpropscreenwidth: Option<i32>,
    pub skyname: Option<String>,
    pub sounds: Option<u32>,
    pub maxrange: Option<f32>,

    // Game-specific properties
    pub maxoccludeearea: Option<f32>,
    pub minoccluderarea: Option<f32>,
    pub maxoccludeearea_csgo: Option<f32>,
    pub minoccluderarea_csgo: Option<f32>,
    pub difficulty_level: Option<u32>,
    pub hdr_level: Option<u32>,

    // Geometry
    pub solids: Vec<Solid>,

    // Entity connections
    pub targetname: Option<String>,
    pub target: Option<String>,

    // Editor data
    pub hidden: Option<bool>,
    pub group: Option<u32>,
    pub editor: EditorData,
}
