use lyon::lyon_tessellation::{FillVertexConstructor, StrokeVertexConstructor};
use nalgebra::{Matrix3, Vector3};

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct Vertex {
    pub(crate) position: [f32; 2],
    pub(crate) color: [f32; 4],
    // pub(crate) _pad: f32,
    pub(crate) tex_coords: [f32; 2],
    pub(crate) mode: u32,
}
impl Vertex {
    pub(crate) fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}
impl std::fmt::Debug for Vertex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ {:?}:{:?} }}", self.position, self.color)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VertexMode {
    OnlyColor,
    Textured,
    TexturedAndTinted,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct VertexConstructor {
    pub(crate) color: (f32, f32, f32, f32),
    pub(crate) transform: Matrix3<f32>,
    pub(crate) tex_coords: (f32, f32),
    pub(crate) mode: VertexMode,
}

impl VertexConstructor {
    pub fn new_only_color(color: (f32, f32, f32, f32), transform: Matrix3<f32>) -> Self {
        Self {
            color,
            transform,
            tex_coords: (0.0, 0.0),
            mode: VertexMode::OnlyColor,
        }
    }
    pub fn new_textured(tex_coords: (f32, f32), transform: Matrix3<f32>) -> Self {
        Self {
            color: (0.0, 0.0, 0.0, 0.0),
            transform,
            tex_coords,
            mode: VertexMode::Textured,
        }
    }
    pub fn new_textured_tinted(
        color: (f32, f32, f32, f32),
        tex_coords: (f32, f32),
        transform: Matrix3<f32>,
    ) -> Self {
        Self {
            color,
            transform,
            tex_coords,
            mode: VertexMode::TexturedAndTinted,
        }
    }
    pub fn with_pos(self, x: f32, y: f32) -> Vertex {
        let v = self.transform * Vector3::new(x, y, 1.0);

        Vertex {
            position: [v.x, v.y],
            color: [self.color.0, self.color.1, self.color.2, self.color.3],
            tex_coords: [self.tex_coords.0, self.tex_coords.1],
            mode: self.mode as u32,
        }
    }
}

impl FillVertexConstructor<Vertex> for VertexConstructor {
    fn new_vertex(&mut self, vertex: lyon::tessellation::FillVertex) -> Vertex {
        let [x, y] = vertex.position().to_array();
        self.with_pos(x, y)
    }
}

impl StrokeVertexConstructor<Vertex> for VertexConstructor {
    fn new_vertex(&mut self, vertex: lyon::tessellation::StrokeVertex) -> Vertex {
        let [x, y] = vertex.position().to_array();
        self.with_pos(x, y)
    }
}
