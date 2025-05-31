
struct Globals {
    screen_size: vec2f,
};


struct VertexInput {
    @location(0) pos: vec2f,
};
struct InstanceInput {
    @location(1) pos0: vec2f,
    @location(2) pos1: vec2f,
    @location(3) pos2: vec2f,
    @location(4) color0: vec4f,
    @location(5) color1: vec4f,
    @location(6) color2: vec4f,
    @location(7) uv0: vec2f,
    @location(8) uv1: vec2f,
    @location(9) uv2: vec2f,

    @location(10) affine_t_x: vec2f,
    @location(11) affine_t_y: vec2f,
    @location(12) affine_offset: vec2f,
};

struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(0) color: vec4f,
    @location(1) uv: vec2f,
};

@vertex
fn vs_main(
    @builtin(vertex_index) v_idx: u32,
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    var positions = array(instance.pos0, instance.pos1, instance.pos2);
    var colors = array(instance.color0, instance.color1, instance.color2);
    var uvs = array(instance.uv0, instance.uv1, instance.uv2);

    let pos = mat2x2f(instance.affine_t_x, instance.affine_t_y) * positions[v_idx] + instance.affine_offset;
    out.pos = vec4f(pos / GLOBALS.screen_size * 2.0, 0.0, 1.0);


    out.color = colors[v_idx];
    out.uv = uvs[v_idx];

    return out;
}


@group(0) @binding(0) var<uniform> GLOBALS: Globals;

@group(1) @binding(0) var TEX_T: texture_2d<f32>;
@group(1) @binding(1) var TEX_S: sampler;


fn fs_color(in: VertexOutput) -> vec4f {

    if in.uv.x <= -1.0 {
        return in.color;
    } else {
        return textureSampleLevel(TEX_T, TEX_S, in.uv, 0.0) * in.color;
    }
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let color = fs_color(in);
    return vec4(color.rgb, color.a);
}