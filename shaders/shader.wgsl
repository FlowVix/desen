// Vertex shader

struct Globals {
    resolution: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) mode: u32,
    @location(4) bind_group: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) mode: u32,
    @location(3) bind_group: u32,
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
    out.bind_group = model.bind_group;
    return out;
}

// Fragment shader

@group(1) @binding(0)
var t_current_tex_1: texture_2d<f32>;
@group(1) @binding(1)
var s_current_tex_1: sampler;

@group(2) @binding(0)
var t_current_tex_2: texture_2d<f32>;
@group(2) @binding(1)
var s_current_tex_2: sampler;

@group(3) @binding(0)
var t_current_tex_3: texture_2d<f32>;
@group(3) @binding(1)
var s_current_tex_3: sampler;

// const T_CURRENT_TEX = array(
//     t_current_tex_1,
//     t_current_tex_2,
//     t_current_tex_3,
// );
// const S_CURRENT_TEX = array(
//     s_current_tex_1,
//     s_current_tex_2,
//     s_current_tex_3,
// );

// fn to_srgb(v: f32) -> f32 {
//     if v < 0.04045 / 12.92 {
//         return v * 12.82;
//     }
//     return pow(v, 1.0 / 2.4) * 1.055 - 0.055;
// }

// fn to_linear(v: f32) -> f32 {
//     if v > 0.04045 {
//         return pow((v + 0.055) / 1.055, 2.4);
//     }
//     return v / 12.92;
// }

fn fs_color(in: VertexOutput) -> vec4<f32> {
    let v = array(1, 2, 3);
    switch in.mode {
        case 0u: {
            return in.color;
        }
        case 1u: {
            switch in.bind_group {
                case 0u: {
                    return textureSampleLevel(t_current_tex_1, s_current_tex_1, in.tex_coords, 0.0);
                }
                case 1u: {
                    return textureSampleLevel(t_current_tex_2, s_current_tex_2, in.tex_coords, 0.0);
                }
                default: {
                    return textureSampleLevel(t_current_tex_3, s_current_tex_3, in.tex_coords, 0.0);
                }
            }
        }
        default: {
            switch in.bind_group {
                case 0u: {
                    return textureSampleLevel(t_current_tex_1, s_current_tex_1, in.tex_coords, 0.0) * in.color;
                }
                case 1u: {
                    return textureSampleLevel(t_current_tex_2, s_current_tex_2, in.tex_coords, 0.0) * in.color;
                }
                default: {
                    return textureSampleLevel(t_current_tex_3, s_current_tex_3, in.tex_coords, 0.0) * in.color;
                }
            }
        }
    }
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return fs_color(in);
}

@fragment
fn fs_main_squared_alpha(in: VertexOutput) -> @location(0) vec4<f32> {
    let c = fs_color(in);
    return vec4<f32>(c.rgb * c.a, c.a);
}