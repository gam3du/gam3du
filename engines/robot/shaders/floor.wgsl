const SQRT2 = sqrt(2.0);
const SQRT1_2 = sqrt(0.5);
const BG_COLOR = vec4<f32>(0.6, 0.7, 0.8, 1.0);
const LINE_COLOR = vec4<f32>(0.1, 0.1, 0.1, 1.0);
const BORDER_COLOR = vec4<f32>(0.4, 0.5, 0.6, 1.0);
const LINE_RADIUS = 0.1;

struct FloorVertex {
    @builtin(position) position: vec4<f32>,
    @location(1) plane_uv: vec2<f32>,
    @location(2) tile_uv: vec2<f32>,
    @location(3) line_pattern: u32,
    @location(4) color: vec4<f32>,
    @location(5) face_index: u32,
    @location(6) normal: vec3<f32>,
};

@group(0)
@binding(0)
var<uniform> transform: mat4x4<f32>;

@group(0)
@binding(1)
var r_color: texture_2d<u32>;

@group(0)
@binding(2)
var<uniform> time_vec: vec2<u32>;

@group(0)
@binding(3)
var<uniform> floor_size: vec2<u32>;

@group(0)
@binding(4)
var<uniform> camera_pos: vec3<f32>;

@group(0)
@binding(5)
var<uniform> light_pos: vec3<f32>;

const COLUMN_MODE = true;

@vertex
fn vs_floor(
    @location(0) position: vec4<f32>,
    @location(2) line_pattern: u32,
    @location(3) color: vec4<f32>,
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> FloorVertex {

    let primitive_index = vertex_to_primitive(vertex_index);
    let face_index = primitive_to_face(primitive_index);
    let position_index = vertex_to_position_index(vertex_index);

    // column and row of the tile
    let tile_xy = vec2(i32(instance_index % floor_size.x), i32(instance_index / floor_size.x));

    /// vertex position within a tile
    var tile_uvw: vec3<f32> = vec3(0.0, 0.0, 0.0);
    if COLUMN_MODE {
        tile_uvw = position_index_to_uvw_blocky(position_index);
    } else {
        tile_uvw = position_index_to_uvw_smooth(position_index);
    }

    var vertex: FloorVertex;
    vertex.line_pattern = line_pattern;
    vertex.color = color;
    vertex.face_index = face_index;

    let plane_uvw = (vec2<f32>(tile_xy) + tile_uvw.xy) / vec2<f32>(floor_size);
    vertex.plane_uv = plane_uvw.xy;
    vertex.tile_uv = tile_uvw.xy;

    if COLUMN_MODE {
        let texel = textureLoad(r_color, tile_xy * 25, 0);
        let height = position.z; // f32(texel.x) / 255.0;
        vertex.position = transform * vec4(position.xy + tile_uvw.xy, position.z * tile_uvw.z, 1.0);
        switch face_index {
            case 0u: { vertex.normal = vec3(0.0, 0.0, 1.0); }
            case 1u: { vertex.normal = vec3(1.0, 0.0, 0.0); }
            case 2u: { vertex.normal = vec3(-1.0, 0.0, 0.0); }
            case 3u: { vertex.normal = vec3(0.0, 1.0, 0.0); }
            case 4u: { vertex.normal = vec3(0.0, -1.0, 0.0); }
            default: {}
        }
    } else {
        let texel = textureLoad(r_color, vec2<i32>(plane_uvw.xy * 256.0), 0);
        let height = f32(texel.x) / 255.0;
        vertex.position = transform * (position + vec4(tile_uvw.xy, tile_uvw.z * height, 0.0));
        // TODO
        vertex.normal = vec3(0.0, 0.0, 1.0);
    }

    // let height = f32(textureLoad(r_color, vec2<i32>((plane_uv + vec2(0.5, 0.5)) * 256.0), 0)) / 255.0;
    // let v = textureLoad(r_color, vec2<i32>(vertex.tex_coord * 256.0), 0) / 255.0;
    // vertex.color = textureLoad(r_color, vec2<i32>(vertex.base_color_texture_coordinates * 256.0), 0);


    return vertex;
}

@fragment
fn fs_floor_tile(vertex: FloorVertex) -> @location(0) vec4<f32> {

    let tex = textureLoad(r_color, vec2<i32>(vertex.tile_uv * 256.0), 0);
    let v = f32(tex.x) / 255.0;
    let tex_color = vec4<f32>(1.0 - (v * 5.0), 1.0 - (v * 15.0), 1.0 - (v * 50.0), 1.0);


    let cc: vec2<f32> = vertex.tile_uv * 2 - vec2<f32>(1.0); // corrected coordinates (from -1.0 to 1.0)

    let line: bool = on_line(cc.y, cc.x, LINE_RADIUS, vertex.line_pattern >> 0u) || on_line((cc.x - cc.y) * SQRT1_2, cc.x + cc.y, LINE_RADIUS, vertex.line_pattern >> 1u) || on_line(cc.x, cc.y, LINE_RADIUS, vertex.line_pattern >> 2u) || on_line((cc.x + cc.y) * SQRT1_2, cc.y - cc.x, LINE_RADIUS, vertex.line_pattern >> 3u);

    let line_end: bool = dot(cc.xy, cc.xy) < (LINE_RADIUS * LINE_RADIUS) && (vertex.line_pattern & 0xFF) != 0;

    let line_corner: bool = (cc.x + cc.y > (2.0 - SQRT2 * LINE_RADIUS) && (vertex.line_pattern & (1u << 9u)) != 0) || (cc.x - cc.y < (-2.0 + SQRT2 * LINE_RADIUS) && (vertex.line_pattern & (1u << 11u)) != 0) || (cc.x + cc.y < (-2.0 + SQRT2 * LINE_RADIUS) && (vertex.line_pattern & (1u << 13u)) != 0) || (cc.x - cc.y > (2.0 - SQRT2 * LINE_RADIUS) && (vertex.line_pattern & (1u << 15u)) != 0);

    let border: bool = cc.x < -0.95 || cc.x > 0.95 || cc.y < -0.95 || cc.y > 0.95;

    // let light = dot(normalize(vertex.normal), normalize(vec3(-1.0, -1.0, 1.0))) * 0.8 + 0.2;
    let light = max(dot(normalize(vertex.normal), normalize(light_pos)), 0.0) * 0.8 + 0.2;

    if line || line_end || line_corner {
        return LINE_COLOR * light;
    } else if border {
        return BORDER_COLOR * light;
    } else {
        return vertex.color * light;
    }
}

fn on_line(axis: f32, part_axis: f32, line_width: f32, axis_pattern: u32) -> bool {
    let on_axis: bool = axis < line_width && axis > -line_width;
    return
        (on_axis && part_axis > 0 && (axis_pattern & (1u << 0u)) != 0) || (on_axis && part_axis < 0 && (axis_pattern & (1u << 4u)) != 0);
}


// primitive_index
//
//   +-----------------------+-----------------------+
//   | \                   /   \                   / |
//   |   \       0       /       \       1       /   |
//   |     \           /           \           /     |
//   |       \       /       10      \       /       |
//   |         \   /                   \   /         |
//   |   4       +-----------------------+       5   |
//   |         / | \                   / | \         |
//   |       /   |   \       14      /   |   \       |
//   |     /     |     \           /     |     \     |
//   |   /       |       \       /       |       \   |
//   | /         |         \   /         |         \ |
//   +       8   |   12      +      13   |    9      +
//   | \         |         /   \         |         / |
//   |   \       |       /       \       |       /   |
//   |     \     |     /           \     |     /     |
//   |       \   |   /       15      \   |   /       |
//   |         \ | /                   \ | /         |
//   |   6       +-----------------------+       7   |
//   |         /   \                   /   \         |
//   |       /       \       11      /       \       |
//   |     /           \           /           \     |
//   |   /       2       \       /       3       \   |
//   | /                   \   /                   \ |
//   +-----------------------+-----------------------+
fn vertex_to_primitive(vertex_index: u32) -> u32 {
    return vertex_index / 3;
}

// face_index
//                                                      
//   +-----------------------+-----------------------+
//   | \                                           / |
//   |   \                                       /   |
//   |     \                 4                 /     |
//   |       \                               /       |
//   |         \                           /         |
//   |           +-----------------------+           |
//   |           |                       |           |
//   |           |                       |           |
//   |           |                       |           |
//   |           |                       |           |
//   |           |                       |           |
//   +     2     |           0           |     1     +
//   |           |                       |           |
//   |           |                       |           |
//   |           |                       |           |
//   |           |                       |           |
//   |           |                       |           |
//   |           +-----------------------+           |
//   |         /                           \         |
//   |       /                               \       |
//   |     /                 3                 \     |
//   |   /                                       \   |
//   | /                                           \ |
//   +-----------------------+-----------------------+
//                                                      
fn primitive_to_face(primitive_index: u32) -> u32 {
    switch primitive_index {
        case 12u, 13u, 14u, 15u: { return 0u; }
        case 5u, 7u, 9u: { return 1u; }
        case 4u, 6u, 8u: { return 2u; }
        case 2u, 3u, 11u: { return 3u; }
        case 0u, 1u, 10u: { return 4u; }
        default: { return 0u; }
    }
}

// vertex_index
//
//   +-----------------------+-----------------------+
//   | \ 2               1 / 30\ 5               4 / |
//   |13 \               /       \               / 17|
//   |     \           /           \           /     |
//   |       \       /               \       /       |
//   |         \ 0 / 31             32 \ 3 /         |
//   |        12 +-----------------------+ 15        |
//   |         / | \ 44             43 / | \         |
//   |       / 26|37 \               / 41|28 \       |
//   |     /     |     \           /     |     \     |
//   |14 /       |       \       /       |       \ 16|
//   | /         |         \ 42/         |         \ |
//   +  24       |       36  +  39       |       27  +
//   | \         |         / 45\         |         / |
//   |19 \       |       /       \       |       / 23|
//   |     \     |     /           \     |     /     |
//   |       \ 25|38 /               \ 40|29 /       |
//   |         \ | / 46             47 \ | /         |
//   |   G    18 +-----------------------+ 21        |
//   |         / 6 \ 35             34 / 9 \         |
//   |       /       \               /       \       |
//   |     /           \           /           \     |
//   |20 /               \       /               \ 22|
//   | / 7              8  \ 33/ 10             11 \ |
//   +-----------------------+-----------------------+

// position_index
// 0                         1                         2
//   +-----------------------+-----------------------+
//   | \                   /   \                   / |
//   |   \               /       \               /   |
//   |     \           /           \           /     |
//   |       \       /               \       /       |
//   |         \9  /                   \ 10/         |
//   |           +-----------------------+           |
//   |         / | \                   / | \         |
//   |       /   |   \               /   |   \       |
//   |     /     |     \           /     |     \     |
//   |   /       |       \       /       |       \   |
//   | /         |         \   /         |         \ |
// 3 +           |         4 +           |           + 5
//   | \         |         /   \         |         / |
//   |   \       |       /       \       |       /   |
//   |     \     |     /           \     |     /     |
//   |       \   |   /               \   |   /       |
//   |         \ | /                   \ | /         |
//   |           +-----------------------+           |
//   |         /11 \                   / 12\         |
//   |       /       \               /       \       |
//   |     /           \           /           \     |
//   |   /               \       /               \   |
//   | /                   \   /                   \ |
//   +-----------------------+-----------------------+
// 6                         7                         8
fn vertex_to_position_index(vertex_index: u32) -> u32 {
    switch vertex_index {
        case 2u, 13u: { return 0u; }
        case 1u, 5u, 30u: { return 1u; }
        case 4u, 17u: { return 2u; }
        case 14u, 19u, 24u: { return 3u; }
        case 36u, 39u, 42u, 45u: { return 4u; }
        case 16u, 23u, 27u: { return 5u; }
        case 7u, 20u: { return 6u; }
        case 8u, 10u, 33u: { return 7u; }
        case 11u, 22u: { return 8u; }
        case 0u, 12u, 26u, 31u, 37u, 44u: { return 9u; }
        case 3u, 15u, 28u, 32u, 41u, 43u: { return 10u; }
        case 6u, 18u, 25u, 35u, 38u, 46u: { return 11u; }
        case 9u, 21u, 29u, 34u, 40u, 47u: { return 12u; }
        default: { return 0u; }
    }
}

fn position_index_to_uvw_smooth(position_index: u32) -> vec3<f32> {
    return vec3(
        position_index_to_u_smooth(position_index),
        position_index_to_v_smooth(position_index),
        1.0,
    );
}

fn position_index_to_u_smooth(position_index: u32) -> f32 {
    switch position_index {
        case 0u, 3u, 6u: { return 0.0; }
        case 9u, 11u: { return 0.25; }
        case 1u, 4u, 7u: { return 0.5; }
        case 10u, 12u: { return 0.75; }
        case 2u, 5u, 8u: { return 1.0; }
        default: { return 0.0; }
    }
}

fn position_index_to_v_smooth(position_index: u32) -> f32 {
    switch position_index {
        case 0u, 1u, 2u: { return 0.0; }
        case 9u, 10u: { return 0.25; }
        case 3u, 4u, 5u: { return 0.5; }
        case 11u, 12u: { return 0.75; }
        case 6u, 7u, 8u: { return 1.0; }
        default: { return 0.0; }
    }
}

fn position_index_to_uvw_blocky(position_index: u32) -> vec3<f32> {
    return vec3(
        position_index_to_u_blocky(position_index),
        position_index_to_v_blocky(position_index),
        position_index_to_w_blocky(position_index),
    );
}

fn position_index_to_u_blocky(position_index: u32) -> f32 {
    switch position_index {
        case 0u, 3u, 6u: { return 0.0; }
        case 9u, 11u: { return 0.0; }
        case 1u, 4u, 7u: { return 0.5; }
        case 10u, 12u: { return 1.0; }
        case 2u, 5u, 8u: { return 1.0; }
        default: { return 0.0; }
    }
}

fn position_index_to_v_blocky(position_index: u32) -> f32 {
    switch position_index {
        case 0u, 1u, 2u: { return 0.0; }
        case 9u, 10u: { return 0.0; }
        case 3u, 4u, 5u: { return 0.5; }
        case 11u, 12u: { return 1.0; }
        case 6u, 7u, 8u: { return 1.0; }
        default: { return 0.0; }
    }
}

fn position_index_to_w_blocky(position_index: u32) -> f32 {
    switch position_index {
        case 0u, 1u, 2u, 3u, 5u, 6u, 7u, 8u: { return 0.0; }
        case 4u , 9u, 10u, 11u, 12u: { return 1.0; }
        default: { return 0.0; }
    }
}
