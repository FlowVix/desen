use glam::{Affine2, Vec2, vec2};

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

pub fn test_in_shape(shape: SenseShape, pos: Vec2) -> bool {
    let pos = shape.inv_transform.transform_point2(pos);
    let [x, y] = [(shape.x, shape.w), (shape.y, shape.h)]
        .map(|(p, d)| if shape.centered { p - d / 2.0 } else { p });

    match shape.typ {
        SenseShapeType::Rect => {
            pos.x >= x && pos.y >= y && pos.x <= (x + shape.w) && pos.y <= (y + shape.h)
        }
        SenseShapeType::Ellipse => {
            let scale = shape.w / shape.h;
            let mut pos = pos;
            pos.y = y + (pos.y - y) * scale;
            let radius = shape.w / 2.0;
            let center = vec2(x, y) + radius;
            (pos - center).length() <= radius
        }
    }
}
