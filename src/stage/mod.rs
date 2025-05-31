pub mod color;
pub mod sense;

use std::{f32::consts::PI, mem::swap};

use itertools::Itertools;

use glam::{Affine2, Mat2, Vec2, vec2};
use sense::{Interactions, SenseSave, SenseShape, SenseShapeType};

use crate::{
    shaders::wgsl_main::structs::{InstanceInput, VertexInput},
    state::data::{TextureInfo, TextureKey},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlendMode {
    Normal,
    Additive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DrawCall {
    pub start_instance: u32,
    pub set_blend_mode: Option<BlendMode>,
    pub set_texture: Option<TextureKey>,
}

pub struct Stage {
    pub(crate) instances: Vec<InstanceInput>,

    pub(crate) draw_calls: Vec<DrawCall>,

    pub fill_color: [f32; 4],
    pub stroke_color: [f32; 4],
    pub stroke_weight: f32,

    pub draw_fill: bool,
    pub draw_stroke: bool,

    pub arc_segments: u16,

    pub transform: Affine2,

    pub(crate) current_blend_mode: BlendMode,
    pub(crate) current_texture: Option<TextureInfo>,

    pub(crate) mouse_pos: Vec2,
    pub(crate) mouse_down: Option<u64>,

    pub(crate) old_senses: Vec<SenseSave>,
    pub(crate) build_senses: Vec<SenseSave>,
    pub(crate) sense_id_ctr: u64,

    pub(crate) interactions: Interactions<Option<u64>>,
}

impl Stage {
    pub(crate) fn new() -> Self {
        let mut out = Self {
            instances: vec![],
            draw_calls: vec![],
            fill_color: [0.0; 4],
            stroke_color: [0.0; 4],
            stroke_weight: 0.0,
            draw_fill: false,
            draw_stroke: false,
            arc_segments: 0,
            transform: Affine2::IDENTITY,
            current_blend_mode: BlendMode::Normal,
            current_texture: None,
            old_senses: vec![],
            build_senses: vec![],
            sense_id_ctr: 0,
            mouse_pos: Vec2::INFINITY,
            mouse_down: None,
            interactions: Interactions {
                hovering: None,
                hovering_bypass: None,
                hover_started: None,
                hover_ended: None,
                holding: None,
                clicked: None,
                click_ended: None,
            },
        };
        out.reset();
        out
    }
    pub(crate) fn reset(&mut self) {
        self.instances.clear();
        self.draw_calls.clear();
        self.draw_calls.push(DrawCall {
            start_instance: 0,
            set_blend_mode: None,
            set_texture: None,
        });

        self.fill_color = [0.3, 0.3, 0.3, 1.0];
        self.stroke_color = [0.8, 0.8, 0.8, 1.0];
        self.stroke_weight = 2.0;

        self.draw_fill = true;
        self.draw_stroke = true;

        self.arc_segments = 8;

        self.transform = Affine2::IDENTITY;

        self.current_blend_mode = BlendMode::Normal;
        self.current_texture = None;

        swap(&mut self.build_senses, &mut self.old_senses);
        self.build_senses.clear();

        self.sense_id_ctr = 0;
    }
    pub(crate) fn update_interactions(&mut self) {
        let old = self.interactions;

        self.interactions.hovering = self.find_top_old_sense().map(|v| v.id);
        self.interactions.hover_started = if self.interactions.hovering != old.hovering {
            self.interactions.hovering
        } else {
            None
        };
        self.interactions.hover_ended = if self.interactions.hovering != old.hovering {
            old.hovering
        } else {
            None
        };

        self.interactions.holding = self.mouse_down;

        self.interactions.clicked = if self.interactions.holding != old.holding {
            self.interactions.holding
        } else {
            None
        };
        self.interactions.click_ended = if self.interactions.holding != old.holding {
            old.holding
        } else {
            None
        };
    }
    fn new_sense_id(&mut self) -> u64 {
        let v = self.sense_id_ctr;
        self.sense_id_ctr += 1;
        v
    }

    pub fn draw_stroke(&mut self, points: impl ExactSizeIterator<Item = [f32; 2]> + Clone) {
        // let n_verts = points.len() as u32 * 2;

        // let (stroke_color, stroke_weight, transform) =
        //     (self.stroke_color, self.stroke_weight, self.transform);

        // self.add_geometry(
        //     points
        //         .circular_tuple_windows()
        //         .flat_map(|(prev, current, next)| {
        //             let angle_prev = (prev[1] - current[1]).atan2(prev[0] - current[0]);
        //             let angle_next = (next[1] - current[1]).atan2(next[0] - current[0]);
        //             let angle = (angle_prev + angle_next) / 2.0;

        //             let angle_diff = angle_next - angle_prev;
        //             let scale = 1.0 / ((PI - angle_diff) / 2.0).cos();

        //             let cos = angle.cos();
        //             let sin = angle.sin();

        //             [
        //                 [
        //                     current[0] - cos * stroke_weight / 2.0 * scale,
        //                     current[1] - sin * stroke_weight / 2.0 * scale,
        //                 ],
        //                 [
        //                     current[0] + cos * stroke_weight / 2.0 * scale,
        //                     current[1] + sin * stroke_weight / 2.0 * scale,
        //                 ],
        //             ]
        //             .map(|v| {
        //                 VertexInput::new(
        //                     v,
        //                     stroke_color,
        //                     [-10.0, 0.0],
        //                     transform.matrix2.x_axis.to_array(),
        //                     transform.matrix2.y_axis.to_array(),
        //                     transform.translation.to_array(),
        //                 )
        //             })
        //         }),
        //     (0..n_verts).flat_map(|i| [i, (i + 1) % n_verts, (i + 2) % n_verts]),
        // );
    }

    // pub fn set_fill_color(&mut self, color: [f32; 4]) {
    //     self.fill_color = color;
    // }
    // pub fn get_fill_color(&mut self) -> [f32; 4] {
    //     self.fill_color
    // }
    // pub fn set_stroke_color(&mut self, color: [f32; 4]) {
    //     self.stroke_color = color;
    // }
    // pub fn get_stroke_color(&mut self) -> [f32; 4] {
    //     self.stroke_color
    // }
    // pub fn set_stroke_weight(&mut self, weight: f32) {
    //     self.stroke_weight = weight;
    // }
    // pub fn get_stroke_weight(&mut self) -> f32 {
    //     self.stroke_weight
    // }

    // pub fn set_draw_fill(&mut self, draw: bool) {
    //     self.draw_fill = draw;
    // }
    // pub fn get_draw_fill(&mut self) -> bool {
    //     self.draw_fill
    // }
    // pub fn set_draw_stroke(&mut self, draw: bool) {
    //     self.draw_stroke = draw;
    // }
    // pub fn get_draw_stroke(&mut self) -> bool {
    //     self.draw_stroke
    // }

    // pub fn set_arc_segments(&mut self, count: u16) {
    //     self.arc_segments = count.max(1);
    // }
    // pub fn get_arc_segments(&mut self) -> u16 {
    //     self.arc_segments
    // }

    // pub fn set_transform(&mut self, transform: Affine2) {
    //     self.transform = transform;
    // }
    // pub fn get_transform(&mut self) -> Affine2 {
    //     self.transform
    // }

    pub fn add_transform(&mut self, transform: Affine2) {
        self.transform *= transform;
    }
    pub fn translate(&mut self, x: f32, y: f32) {
        self.add_transform(Affine2::from_translation(vec2(x, y)));
    }
    pub fn rotate(&mut self, angle: f32) {
        self.add_transform(Affine2::from_angle(angle));
    }
    pub fn rotate_xy(&mut self, x: f32, y: f32) {
        self.add_transform(Affine2::from_mat2(Mat2::from_cols(
            vec2(x.cos(), x.sin()),
            vec2(-y.sin(), y.cos()),
        )));
    }
    pub fn scale(&mut self, x: f32, y: f32) {
        self.add_transform(Affine2::from_scale(vec2(x, y)));
    }
    pub fn skew(&mut self, x: f32, y: f32) {
        self.add_transform(Affine2::from_mat2(Mat2::from_cols(
            vec2(1.0, y),
            vec2(x, 1.0),
        )));
    }

    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        if self.current_blend_mode != mode {
            self.draw_calls.push(DrawCall {
                start_instance: self.instances.len() as u32,
                set_blend_mode: Some(mode),
                set_texture: None,
            });
            self.current_blend_mode = mode;
        }
    }
    pub fn set_texture(&mut self, texture: TextureInfo) {
        if self.current_texture != Some(texture) {
            self.draw_calls.push(DrawCall {
                start_instance: self.instances.len() as u32,
                set_blend_mode: None,
                set_texture: Some(texture.key),
            });
            self.current_texture = Some(texture);
        }
    }

    pub(crate) fn find_top_old_sense(&self) -> Option<SenseSave> {
        for sense in self.old_senses.iter().rev() {
            if test_in_shape(sense.shape, self.mouse_pos) {
                return Some(*sense);
            }
        }
        None
    }
}

fn test_in_shape(shape: SenseShape, mouse: Vec2) -> bool {
    let pos = shape.inv_transform.transform_point2(mouse);
    let [x, y] = [(shape.x, shape.w), (shape.y, shape.h)]
        .map(|(p, d)| if shape.centered { p - d / 2.0 } else { p });

    match shape.typ {
        SenseShapeType::Rect => {
            pos.x >= x && pos.y >= y && pos.x <= (x + shape.w) && pos.y <= (y + shape.h)
        }
        SenseShapeType::Ellipse => todo!(),
    }
}

#[bon::bon]
impl Stage {
    #[builder(finish_fn = draw)]
    pub fn tri(
        &mut self,
        a: [f32; 2],
        b: [f32; 2],
        c: [f32; 2],
        color_a: [f32; 4],
        color_b: [f32; 4],
        color_c: [f32; 4],
        #[builder(default = [-10.0, 0.0])] uv_a: [f32; 2],
        #[builder(default = [-10.0, 0.0])] uv_b: [f32; 2],
        #[builder(default = [-10.0, 0.0])] uv_c: [f32; 2],
    ) {
        self.instances.push(InstanceInput::new(
            a,
            b,
            c,
            color_a,
            color_b,
            color_c,
            uv_a,
            uv_b,
            uv_c,
            self.transform.matrix2.x_axis.to_array(),
            self.transform.matrix2.y_axis.to_array(),
            self.transform.translation.to_array(),
        ));
    }

    #[builder(finish_fn = draw)]
    pub fn rect(
        &mut self,
        #[builder(default = 0.0)] x: f32,
        #[builder(default = 0.0)] y: f32,
        #[builder(default = 0.0)] w: f32,
        #[builder(default = 0.0)] h: f32,
        #[builder(default = false)] centered: bool,
        rounded: Option<f32>,
    ) {
        match rounded {
            Some(_) => todo!(),
            None => {
                let mut points =
                    [[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]].map(|[p0, p1]| [p0 + x, p1 + y]);

                if centered {
                    for i in &mut points {
                        i[0] -= w / 2.0;
                        i[1] -= h / 2.0;
                    }
                }
                if self.draw_fill {
                    let color = self.fill_color;
                    self.tri()
                        .a(points[0])
                        .b(points[1])
                        .c(points[2])
                        .color_a(color)
                        .color_b(color)
                        .color_c(color)
                        .draw();
                    self.tri()
                        .a(points[2])
                        .b(points[3])
                        .c(points[0])
                        .color_a(color)
                        .color_b(color)
                        .color_c(color)
                        .draw();
                }
                if self.draw_stroke {
                    self.draw_stroke(points.into_iter());
                }
            }
        }
    }

    #[builder(finish_fn = test)]
    pub fn rect_sense(
        &mut self,
        #[builder(default = 0.0)] x: f32,
        #[builder(default = 0.0)] y: f32,
        #[builder(default = 0.0)] w: f32,
        #[builder(default = 0.0)] h: f32,
        #[builder(default = false)] centered: bool,
    ) -> Interactions<bool> {
        let id = self.new_sense_id();

        let shape = SenseShape {
            typ: SenseShapeType::Rect,
            x,
            y,
            w,
            h,
            centered,
            inv_transform: self.transform.inverse(),
        };

        self.build_senses.push(SenseSave { shape, id });

        let in_shape = test_in_shape(shape, self.mouse_pos);

        Interactions {
            hovering: self.interactions.hovering.is_some_and(|v| v == id),
            hover_started: self.interactions.hover_started.is_some_and(|v| v == id),
            hover_ended: self.interactions.hover_ended.is_some_and(|v| v == id),
            hovering_bypass: in_shape,

            holding: self.interactions.holding.is_some_and(|v| v == id),
            clicked: self.interactions.clicked.is_some_and(|v| v == id),
            click_ended: self.interactions.click_ended.is_some_and(|v| v == id),
        }
    }
}
