
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

    // 0: no, 1: mask, 2: color
    @location(13) is_text: u32,
    @location(14) clip_poly: u32,
};

struct VertexOutput {
    @builtin(position) pos: vec4f,
    @location(0) color: vec4f,
    @location(1) uv: vec2f,
    @location(2) is_text: u32,
    @location(3) clip_poly: u32,
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

    var pos = mat2x2f(instance.affine_t_x, instance.affine_t_y) * positions[v_idx] + instance.affine_offset;
    if instance.is_text > 0 {
        // pos.x = round()
        // if GLOBALS.screen_size.x % 2 == 1 {
        //     pos.x += 0.5;
        // }
        // if GLOBALS.screen_size.y % 2 == 1 {
        //     pos.y += 0.5;
        // }
    }
    out.pos = vec4f(pos / GLOBALS.screen_size * 2.0, 0.0, 1.0);


    out.color = colors[v_idx];
    out.uv = uvs[v_idx];
    switch instance.is_text {
        case 1u: {
            out.uv /= vec2f(textureDimensions(TEXT_MASK_T));
        }
        case 2u: {
            out.uv /= vec2f(textureDimensions(TEXT_COLOR_T));
        }
        default: {}
    }
    out.is_text = instance.is_text;
    out.clip_poly = instance.clip_poly;

    return out;
}


struct ClipPolygon {
    start_point: u32,
    end_point: u32,
    // 0 is no parent
    parent: u32,
}

@group(0) @binding(0) var<uniform> GLOBALS: Globals;

@group(1) @binding(0) var TEX_T: texture_2d<f32>;
@group(1) @binding(1) var TEX_S: sampler;

@group(2) @binding(0) var TEXT_MASK_T: texture_2d<f32>;
@group(2) @binding(1) var TEXT_MASK_S: sampler;
@group(2) @binding(2) var TEXT_COLOR_T: texture_2d<f32>;
@group(2) @binding(3) var TEXT_COLOR_S: sampler;

@group(3) @binding(0) var<storage> CLIP_POLYGON_POINTS: array<vec2f>;
@group(3) @binding(1) var<storage> CLIP_POLYGONS: array<ClipPolygon>;


fn fs_color(in: VertexOutput) -> vec4f {

    if in.uv.x <= -1.0 {
        return in.color;
    } else {
        switch in.is_text {
            case 1u: {
                var color = in.color;
                color.a *= textureSampleLevel(TEXT_MASK_T, TEXT_MASK_S, in.uv, 0.0).r;
                return color;
            }
            case 2u: {
                return textureSampleLevel(TEXT_COLOR_T, TEXT_COLOR_S, in.uv, 0.0) * in.color;
            }
            default: {
                return textureSampleLevel(TEX_T, TEX_S, in.uv, 0.0) * in.color;
            }
        }
    }
}

fn point_in_poly(pos: vec2f, poly: ClipPolygon) -> bool {
    var c = false;
    let point_count = poly.end_point - poly.start_point;
    for(var i = 0u; i < point_count; i++) {
        let idx1 = i + poly.start_point;
        let idx2 = ((i + 1) % point_count) + poly.start_point;

        let a = CLIP_POLYGON_POINTS[idx1];
        let b = CLIP_POLYGON_POINTS[idx2];

        if pos.x == a.x && pos.y == a.y {
            return true;
        }
        if (a.y > pos.y) != (b.y > pos.y) {
            let slope = (pos.x - a.x) * (b.y - a.y) - (b.x - a.x) * (pos.y - a.y);
            if slope == 0 {
                return true;
            }
            if (slope < 0) != (b.y < a.y) {
                c = !c;
            }
        }
    }
    return c;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let color = fs_color(in);

    var antialias = array(
        vec2(-3.0 / 8.0, 1.0 / 8.0),
        vec2(1.0 / 8.0, 3.0 / 8.0),
        vec2(3.0 / 8.0, -1.0 / 8.0),
        vec2(-1.0 / 8.0, -3.0 / 8.0),
        vec2(0.0, 0.0),
    );

    let world_pos = (in.pos.xy - GLOBALS.screen_size / 2.0) * vec2(1.0, -1.0);

    var final_weight = 1.0;

    var clip_poly = in.clip_poly;
    while clip_poly != 0 {
        let poly = CLIP_POLYGONS[clip_poly];

        var weight = 0.0;

        for (var i = 0; i < 5; i++) {
            let pos = world_pos + antialias[i];
            if point_in_poly(pos, poly) {
                weight += 1.0;
            }
        }

        final_weight *= weight / 5;

        clip_poly = poly.parent;
    }

    return vec4(color.rgb, color.a * final_weight);
}