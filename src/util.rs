use glam::{Mat2, Vec2, vec2};

pub fn cart_to_bary(r: Vec2, r1: Vec2, r2: Vec2, r3: Vec2) -> [f32; 3] {
    let t = Mat2::from_cols(
        vec2(r1.x - r3.x, r1.y - r3.y),
        vec2(r2.x - r3.x, r2.y - r3.y),
    );
    let t_det = t.determinant();
    let l1 = (r - r3).perp_dot(r2 - r3) / (r1 - r3).perp_dot(r2 - r3);
    let l2 = (r - r3).perp_dot(r3 - r1) / (r1 - r3).perp_dot(r2 - r3);
    let l3 = (r - r1).perp_dot(r1 - r2) / (r1 - r3).perp_dot(r2 - r3);
    [l1, l2, l3]
}
