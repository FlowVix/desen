use glam::Affine2;

#[derive(Debug, Clone, Copy)]
pub struct Interactions<T> {
    pub hovering: T,
    pub hovering_bypass: T,
    pub hover_started: T,
    pub hover_ended: T,

    pub holding: T,
    pub clicked: T,
    pub click_ended: T,
}

#[derive(Debug, Clone, Copy)]
pub enum SenseShapeType {
    Rect,
    Ellipse,
}
#[derive(Debug, Clone, Copy)]
pub struct SenseShape {
    pub typ: SenseShapeType,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub centered: bool,
    pub inv_transform: Affine2,
}
#[derive(Debug, Clone, Copy)]
pub struct SenseSave {
    pub(crate) shape: SenseShape,
    pub(crate) id: u64,
}
