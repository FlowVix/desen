// Vertex shader

struct Globals {
    resolution: vec2<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) mode: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) mode: u32,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position / globals.resolution * 2.0, 0.0, 1.0);
    out.tex_coords = model.tex_coords;
    out.mode = model.mode;
    return out;
}

// Fragment shader

@group(1) @binding(0)
var t_atlas: texture_2d<f32>;
@group(1) @binding(1)
var s_atlas: sampler;

fn to_srgb(v: f32) -> f32 {
    if v < 0.04045 / 12.92 {
        return v * 12.82;
    }
    return pow(v, 1.0 / 2.4) * 1.055 - 0.055;
}

fn to_linear(v: f32) -> f32 {
    if v > 0.04045 {
        return pow((v + 0.055) / 1.055, 2.4);
    }
    return v / 12.92;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_atlas, s_atlas, in.tex_coords);

    if in.mode == u32(0) {
        return in.color;
    }
    if in.mode == u32(1) {
        return tex_color;
    }
    return tex_color * vec4<f32>((in.color.r), (in.color.g), (in.color.b), in.color.a);
}