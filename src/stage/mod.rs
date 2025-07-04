pub mod color;
pub mod path;
pub mod sense;

use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    f32::consts::PI,
    hash::{DefaultHasher, Hash, Hasher},
    mem::swap,
    rc::Rc,
    time::Instant,
};

use cosmic_text::AttrsOwned;
use itertools::Itertools;

use glam::{Affine2, Mat2, Vec2, Vec4, vec2};

use lyon::algorithms::walk;
use sense::{Interactions, SenseSave, SenseShape, SenseShapeType, test_in_shape};

use crate::{
    AppData, Path,
    render::{
        shaders::wgsl_main,
        text::{HashableMetrics, find_closest_attrs, glyph::prepare_glyph},
    },
    stage::color::Color,
    state::texture::{TextureInfo, TextureKey},
    util::cart_to_bary,
};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub struct ClipID(u32);

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RenderPass {
    pub start_instance: u32,
    pub draw_calls: Vec<DrawCall>,
}

pub struct Stage {
    // gpu related -------------------------------
    pub(crate) instances: Vec<wgsl_main::structs::InstanceInput>,
    pub(crate) clip_polygon_points: Vec<[f32; 2]>,
    pub(crate) clip_polygons: Vec<wgsl_main::structs::ClipPolygon>,

    pub(crate) render_passes: Vec<RenderPass>,

    // modifiable -------------------------------
    pub fill_color: Color,
    pub stroke_color: Color,
    pub stroke_weight: f32,

    pub draw_fill: bool,
    pub draw_stroke: bool,

    pub arc_segments: u16,

    pub transform: Affine2,

    pub(crate) current_blend_mode: BlendMode,
    pub(crate) current_texture: Option<TextureInfo>,

    pub(crate) current_clip: u32,

    // outside handled readonly -------------------------------
    pub(crate) mouse_pos: Vec2,
    pub(crate) mouse_down: Option<u64>,
    pub(crate) right_mouse_down: Option<u64>,
    pub(crate) delta: f64,

    // interaction -------------------------------
    pub(crate) old_senses: Vec<SenseSave>,
    pub(crate) build_senses: Vec<SenseSave>,
    pub(crate) sense_id_ctr: u64,

    pub(crate) interactions: Interactions<Option<u64>>,

    // cached/temp -------------------------------
    pub(crate) cached_buffers:
        HashMap<(HashableMetrics, cosmic_text::AttrsOwned, String), (cosmic_text::Buffer, bool)>,

    pub(crate) temp_states: HashMap<(TypeId, TypeId, u64), (Box<dyn Any>, bool)>,
}

impl Stage {
    pub(crate) fn new() -> Self {
        let mut out = Self {
            instances: vec![],
            clip_polygon_points: vec![],
            clip_polygons: vec![],
            render_passes: vec![],
            fill_color: Color::rgba8(0, 0, 0, 0),
            stroke_color: Color::rgba8(0, 0, 0, 0),
            stroke_weight: 0.0,
            draw_fill: false,
            draw_stroke: false,
            arc_segments: 0,
            transform: Affine2::IDENTITY,
            current_blend_mode: BlendMode::Normal,
            current_texture: None,
            current_clip: 0,
            old_senses: vec![],
            build_senses: vec![],
            sense_id_ctr: 0,
            mouse_pos: Vec2::INFINITY,
            mouse_down: None,
            right_mouse_down: None,
            delta: 0.0,
            interactions: Interactions {
                hovering: None,
                hovering_bypass: None,
                hover_started: None,
                hover_ended: None,
                holding: None,
                click_started: None,
                click_ended: None,
                right_holding: None,
                right_click_started: None,
                right_click_ended: None,
            },
            cached_buffers: HashMap::new(),
            temp_states: HashMap::new(),
        };
        out.start();
        out
    }
    pub(crate) fn start(&mut self) {
        self.instances.clear();
        self.clip_polygon_points.clear();
        self.clip_polygons.clear();
        // there need to be some items in
        self.clip_polygon_points.push([0.0; 2]);
        self.clip_polygons
            .push(wgsl_main::structs::ClipPolygon::new(0, 0, 0));

        self.render_passes.clear();
        self.render_passes.push(RenderPass {
            start_instance: 0,
            draw_calls: vec![DrawCall {
                start_instance: 0,
                set_blend_mode: None,
                set_texture: None,
            }],
        });

        self.fill_color = Color::rgb8(255, 255, 255);
        self.stroke_color = Color::rgb8(255, 255, 255);
        self.stroke_weight = 2.0;

        self.draw_fill = true;
        self.draw_stroke = false;

        self.arc_segments = 8;

        self.transform = Affine2::IDENTITY;

        self.current_blend_mode = BlendMode::Normal;
        self.current_texture = None;
        self.current_clip = 0;

        swap(&mut self.build_senses, &mut self.old_senses);
        self.build_senses.clear();

        self.sense_id_ctr = 0;

        self.update_interactions();

        // clear unused buffers then set them all to unused
        self.cached_buffers.retain(|_, (_, in_use)| *in_use);
        for (_, in_use) in self.cached_buffers.values_mut() {
            *in_use = false;
        }
        // clear unused temp states then set them all to unused
        self.temp_states.retain(|_, (_, in_use)| *in_use);
        for (_, in_use) in self.temp_states.values_mut() {
            *in_use = false;
        }
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
        self.interactions.click_started = if self.interactions.holding != old.holding {
            self.interactions.holding
        } else {
            None
        };
        self.interactions.click_ended = if self.interactions.holding != old.holding {
            old.holding
        } else {
            None
        };

        self.interactions.right_holding = self.right_mouse_down;
        self.interactions.right_click_started =
            if self.interactions.right_holding != old.right_holding {
                self.interactions.right_holding
            } else {
                None
            };
        self.interactions.right_click_ended =
            if self.interactions.right_holding != old.right_holding {
                old.right_holding
            } else {
                None
            };
    }

    pub fn add_clip(&mut self, path: &Path) {
        let start_point = self.clip_polygon_points.len();

        walk::walk_along_path(
            &path.inner,
            0.0,
            0.1,
            &mut walk::RegularPattern {
                callback: |event: walk::WalkerEvent| {
                    let pos = self
                        .transform
                        .transform_point2(vec2(event.position.x, event.position.y));

                    self.clip_polygon_points.push([pos.x, pos.y]);
                    true
                },
                interval: 5.0,
            },
        );

        let end_point = self.clip_polygon_points.len();
        self.clip_polygons
            .push(wgsl_main::structs::ClipPolygon::new(
                start_point as u32,
                end_point as u32,
                self.current_clip,
            ));
        self.current_clip = self.clip_polygons.len() as u32 - 1;
    }
    pub fn get_clip_id(&self) -> ClipID {
        ClipID(self.current_clip)
    }
    pub fn set_clip_id(&mut self, id: ClipID) {
        self.current_clip = id.0;
    }

    fn new_sense_id(&mut self) -> u64 {
        let v = self.sense_id_ctr;
        self.sense_id_ctr += 1;
        v
    }

    pub fn temp<K: Hash + 'static, T: 'static, F: FnOnce() -> T>(
        &mut self,
        key: K,
        new: F,
    ) -> &mut T {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let key = (key.type_id(), TypeId::of::<T>(), hasher.finish());
        let (val, in_use) = self
            .temp_states
            .entry(key)
            .or_insert_with(|| (Box::new(new()), true));
        *in_use = true;
        val.downcast_mut().unwrap()
    }

    pub fn delta(&self) -> f64 {
        self.delta
    }
    pub fn mouse_down(&self) -> bool {
        self.mouse_down.is_some()
    }
    pub fn right_mouse_down(&self) -> bool {
        self.right_mouse_down.is_some()
    }
    pub fn mouse_world_pos(&self) -> [f32; 2] {
        self.mouse_pos.to_array()
    }

    pub fn draw_stroke(&mut self, points: impl ExactSizeIterator<Item = [f32; 2]> + Clone) {
        let n_verts = points.len() as u32 * 2;

        let (stroke_color, stroke_weight) = (self.stroke_color, self.stroke_weight);

        let points = points
            .circular_tuple_windows()
            .flat_map(|(prev, current, next)| {
                let angle_prev = (prev[1] - current[1]).atan2(prev[0] - current[0]);
                let angle_next = (next[1] - current[1]).atan2(next[0] - current[0]);
                let angle = (angle_prev + angle_next) / 2.0;

                let angle_diff = angle_next - angle_prev;
                let scale = 1.0 / ((PI - angle_diff) / 2.0).cos();

                let cos = angle.cos();
                let sin = angle.sin();

                [
                    [
                        current[0] - cos * stroke_weight / 2.0 * scale,
                        current[1] - sin * stroke_weight / 2.0 * scale,
                    ],
                    [
                        current[0] + cos * stroke_weight / 2.0 * scale,
                        current[1] + sin * stroke_weight / 2.0 * scale,
                    ],
                ]
            })
            .collect_vec();

        for [a, b, c] in (0..n_verts).map(|i| [i, (i + 1) % n_verts, (i + 2) % n_verts]) {
            self.tri()
                .a(points[a as usize])
                .b(points[b as usize])
                .c(points[c as usize])
                .color_a(stroke_color)
                .color_b(stroke_color)
                .color_c(stroke_color)
                .draw();
        }
    }

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
        let calls = &mut self.render_passes.last_mut().unwrap().draw_calls;
        if self.current_blend_mode != mode {
            calls.push(DrawCall {
                start_instance: self.instances.len() as u32,
                set_blend_mode: Some(mode),
                set_texture: None,
            });
            self.current_blend_mode = mode;
        }
    }
    pub fn get_blend_mode(&self) -> BlendMode {
        self.current_blend_mode
    }
    pub fn set_texture(&mut self, texture: TextureInfo) {
        let calls = &mut self.render_passes.last_mut().unwrap().draw_calls;
        if self.current_texture != Some(texture) {
            calls.push(DrawCall {
                start_instance: self.instances.len() as u32,
                set_blend_mode: None,
                set_texture: Some(texture.key),
            });
            self.current_texture = Some(texture);
        }
    }
    pub fn get_texture(&self) -> Option<TextureInfo> {
        self.current_texture
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

// MARK: Draw builders
#[bon::bon]
impl Stage {
    #[builder(finish_fn = draw)]
    pub fn tri(
        &mut self,
        a: [f32; 2],
        b: [f32; 2],
        c: [f32; 2],
        color_a: Color,
        color_b: Color,
        color_c: Color,
        #[builder(default = [-10.0, 0.0])] uv_a: [f32; 2],
        #[builder(default = [-10.0, 0.0])] uv_b: [f32; 2],
        #[builder(default = [-10.0, 0.0])] uv_c: [f32; 2],
    ) {
        self.instances.push(wgsl_main::structs::InstanceInput::new(
            a,
            b,
            c,
            color_a.to_array(),
            color_b.to_array(),
            color_c.to_array(),
            uv_a,
            uv_b,
            uv_c,
            self.transform.matrix2.x_axis.to_array(),
            self.transform.matrix2.y_axis.to_array(),
            self.transform.translation.to_array(),
            0,
            self.current_clip,
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
    ) {
        let mut points = [[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]].map(|[p0, p1]| [p0 + x, p1 + y]);

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
    #[builder(finish_fn = draw)]
    pub fn image(
        &mut self,
        #[builder(default = 0.0)] x: f32,
        #[builder(default = 0.0)] y: f32,
        #[builder(default = 0.0)] w: f32,
        #[builder(default = 0.0)] h: f32,
        #[builder(default = false)] centered: bool,
        #[builder(default = false)] tint: bool,
        #[builder(default = [0.0, 0.0, 1.0, 1.0])] uv: [f32; 4],
    ) {
        let mut points = [[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]].map(|[p0, p1]| [p0 + x, p1 + y]);

        if centered {
            for i in &mut points {
                i[0] -= w / 2.0;
                i[1] -= h / 2.0;
            }
        }
        if self.draw_fill {
            let color = if tint {
                self.fill_color
            } else {
                Color::rgb8(255, 255, 255)
            };

            let [uv_x0, uv_y0, uv_x1, uv_y1] = self
                .current_texture
                .map(|_| uv)
                .unwrap_or([-10.0, -10.0, -10.0, -10.0]);

            let uv_pts = [
                [uv_x0, uv_y1],
                [uv_x1, uv_y1],
                [uv_x1, uv_y0],
                [uv_x0, uv_y0],
            ];

            self.tri()
                .a(points[0])
                .b(points[1])
                .c(points[2])
                .color_a(color)
                .color_b(color)
                .color_c(color)
                .uv_a(uv_pts[0])
                .uv_b(uv_pts[1])
                .uv_c(uv_pts[2])
                .draw();
            self.tri()
                .a(points[2])
                .b(points[3])
                .c(points[0])
                .color_a(color)
                .color_b(color)
                .color_c(color)
                .uv_a(uv_pts[2])
                .uv_b(uv_pts[3])
                .uv_c(uv_pts[0])
                .draw();
        }
    }

    #[builder(finish_fn = draw)]
    pub fn ellipse(
        &mut self,
        #[builder(default = 0.0)] x: f32,
        #[builder(default = 0.0)] y: f32,
        #[builder(default = 0.0)] w: f32,
        #[builder(default = 0.0)] h: f32,
    ) {
        let point_count = (((w + h) * self.transform.matrix2.determinant().sqrt()).ln() * 10.0)
            .clamp(3.0, 60.0) as usize;

        let angle = 2.0 * PI / point_count as f32;

        let points = (0..point_count)
            .map(|v| {
                [
                    x + (v as f32 * angle).cos() * w,
                    y + (v as f32 * angle).sin() * h,
                ]
            })
            .collect_vec();

        if self.draw_fill {
            let color = self.fill_color;

            for i in 1..(point_count - 1) {
                self.tri()
                    .a(points[0])
                    .b(points[i])
                    .c(points[i + 1])
                    .color_a(color)
                    .color_b(color)
                    .color_c(color)
                    .draw();
            }
        }
        if self.draw_stroke {
            self.draw_stroke(points.into_iter());
        }
    }

    #[builder(finish_fn = draw)]
    pub fn line(
        &mut self,
        #[builder(default = 0.0)] x1: f32,
        #[builder(default = 0.0)] y1: f32,
        #[builder(default = 0.0)] x2: f32,
        #[builder(default = 0.0)] y2: f32,
    ) {
        if self.draw_stroke {
            let to = (vec2(x2, y2) - vec2(x1, y1)).normalize() * self.stroke_weight / 2.0;
            let to = vec2(to.y, -to.x);
            let points = [
                [x1 + to.x, y1 + to.y],
                [x2 + to.x, y2 + to.y],
                [x2 - to.x, y2 - to.y],
                [x1 - to.x, y1 - to.y],
            ];

            let color = self.stroke_color;
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
    }

    #[builder(finish_fn = draw)]
    pub fn text<'a>(
        &mut self,
        app_data: &mut AppData,
        #[builder(default = 0.0)] x: f32,
        #[builder(default = 0.0)] y: f32,
        w: Option<f32>,
        h: Option<f32>,
        text: impl ToString,
        #[builder(default = 16.0)] size: f32,
        #[builder(default = 1.3)] line_height: f32,
        #[builder(default = cosmic_text::Family::SansSerif)] family: cosmic_text::Family<'a>,
        #[builder(default = cosmic_text::Weight::NORMAL)] weight: cosmic_text::Weight,
        #[builder(default = cosmic_text::Style::Normal)] style: cosmic_text::Style,
        #[builder(default = cosmic_text::Stretch::Normal)] stretch: cosmic_text::Stretch,
    ) {
        let metrics = cosmic_text::Metrics::relative(size, line_height);
        let attrs = AttrsOwned::new(&find_closest_attrs(
            app_data.gpu_data.font_system.db(),
            family,
            weight,
            style,
            stretch,
        ));
        let text = text.to_string();

        let (buffer, in_use) = self
            .cached_buffers
            .entry((HashableMetrics(metrics), attrs.clone(), text.clone()))
            .or_insert_with(|| {
                let mut buffer =
                    cosmic_text::Buffer::new(&mut app_data.gpu_data.font_system, metrics);

                buffer.set_text(
                    &mut app_data.gpu_data.font_system,
                    text.as_ref(),
                    &attrs.as_attrs(),
                    cosmic_text::Shaping::Advanced,
                );

                (buffer, true)
            });
        *in_use = true;

        buffer.set_size(&mut app_data.gpu_data.font_system, w, h);
        buffer.shape_until_scroll(&mut app_data.gpu_data.font_system, true);

        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let physical = glyph.physical((0.0, 0.0), 1.0);

                if let Some(instances) = prepare_glyph(
                    physical,
                    run.line_y,
                    &mut app_data.gpu_data,
                    self.fill_color.to_array(),
                    self.transform,
                    x,
                    y,
                    self.current_clip,
                ) {
                    self.instances.extend(instances);
                    // self.push_rect_direct(rect);
                }
            }
        }
    }
}

// MARK: Sense builders
#[bon::bon]
impl Stage {
    fn add_sense(&mut self, shape: SenseShape, id: u64) -> Interactions<bool> {
        self.build_senses.push(SenseSave { shape, id });

        let in_shape = test_in_shape(shape, self.mouse_pos);

        Interactions {
            hovering: self.interactions.hovering.is_some_and(|v| v == id),
            hover_started: self.interactions.hover_started.is_some_and(|v| v == id),
            hover_ended: self.interactions.hover_ended.is_some_and(|v| v == id),
            hovering_bypass: in_shape,

            holding: self.interactions.holding.is_some_and(|v| v == id),
            click_started: self.interactions.click_started.is_some_and(|v| v == id),
            click_ended: self.interactions.click_ended.is_some_and(|v| v == id),

            right_holding: self.interactions.right_holding.is_some_and(|v| v == id),
            right_click_started: self
                .interactions
                .right_click_started
                .is_some_and(|v| v == id),
            right_click_ended: self.interactions.right_click_ended.is_some_and(|v| v == id),
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

        self.add_sense(shape, id)
    }

    #[builder(finish_fn = test)]
    pub fn ellipse_sense(
        &mut self,
        #[builder(default = 0.0)] x: f32,
        #[builder(default = 0.0)] y: f32,
        #[builder(default = 0.0)] w: f32,
        #[builder(default = 0.0)] h: f32,
        #[builder(default = false)] centered: bool,
    ) -> Interactions<bool> {
        let id = self.new_sense_id();

        let shape = SenseShape {
            typ: SenseShapeType::Ellipse,
            x,
            y,
            w,
            h,
            centered,
            inv_transform: self.transform.inverse(),
        };

        self.add_sense(shape, id)
    }
}
