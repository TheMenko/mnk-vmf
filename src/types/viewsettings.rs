#[derive(Debug)]
pub enum ViewSettings {
    SnapToGrid(bool),
    ShowGrid(bool),
    ShowLogicalGrid(bool),
    GridSpacing(u32),
    Show3DGrid(bool),
    HideObjects(bool),
    HideWalls(bool),
    HideStripes(bool),
    HideNeighbors(bool),
    HideDetail(bool),
    ShowBrushes(bool),
    ShowEntities(bool),
    ShowLightRadius(bool),
    ShowLightingPreview(bool),
    ShowWireframe(bool),
}
