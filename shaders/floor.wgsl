const sqrt2 = 1.414213562373095; // sqrt of 2
const sqrt1_2 = 0.707106781186547; // sqrt of 1/2
const bg_color = vec4<f32>(0.8, 0.8, 0.8, 1.0);
const line_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
const border_color = vec4<f32>(0.2, 0.2, 0.2, 1.0);

struct FloorVertex {
    @builtin(position) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) line_pattern: u32,
};

@group(0)
@binding(0)
var<uniform> transform: mat4x4<f32>;

@group(0)
@binding(2)
var<uniform> time_vec: vec2<u32>;

@vertex
fn vs_floor(
    @location(0) position: vec4<f32>,
    @location(2) line_pattern: u32,
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
) -> FloorVertex {

    //  2         3
    //  +---------+
    //  | \       |
    //  |   \     |
    //  |     \   |
    //  |       \ |
    //  +---------+
    //  0         1

    let is_right = (vertex_index & 0x01) != 0;
    let is_top = (vertex_index & 0x02) != 0;

    var vertex: FloorVertex;
    vertex.tex_coord = vec2(f32(is_right), f32(is_top));
    vertex.position = transform * (position + vec4(vertex.tex_coord, 0.0, 0.0));
    vertex.line_pattern = line_pattern;

    return vertex;
}

@fragment
fn fs_floor_tile(vertex: FloorVertex) -> @location(0) vec4<f32> {
    let cc: vec2<f32> = vertex.tex_coord * 2 - vec2<f32>(1.0); // corrected coordinates (from -1.0 to 1.0)
    
    let line: bool =
        on_line(cc.x, cc.y, 0.1, vertex.line_pattern >> 0u) ||
        on_line(cc.y, cc.x, 0.1, vertex.line_pattern >> 2u) ||
        on_line((cc.x + cc.y) * sqrt1_2, cc.x - cc.y, 0.1, vertex.line_pattern >> 4u) ||
        on_line((cc.x - cc.y) * sqrt1_2, cc.x + cc.y, 0.1, vertex.line_pattern >> 6u);
    
    let line_end: bool = (cc.x * cc.x + cc.y * cc.y) < (0.1 * 0.1) && (vertex.line_pattern & 0xFF) != 0;
    
    let line_corner: bool =
        (cc.x - cc.y < (-2.0 + sqrt2 * 0.1) && (vertex.line_pattern & (1u << 8u)) != 0) ||
        (cc.x - cc.y > (2.0 - sqrt2 * 0.1) && (vertex.line_pattern & (1u << 9u)) != 0) ||
        (cc.x + cc.y < (-2.0 + sqrt2 * 0.1) && (vertex.line_pattern & (1u << 10u)) != 0) ||
        (cc.x + cc.y > (2.0 - sqrt2 * 0.1) && (vertex.line_pattern & (1u << 11u)) != 0);
    
    let border: bool = cc.x < -0.95 || cc.x > 0.95 || cc.y < -0.95 || cc.y > 0.95;
    
    if(line || line_end || line_corner) {
        return line_color;
    } else if(border) {
        return border_color;
    } else {
        return bg_color;
    }
}

fn on_line(axis: f32, part_axis: f32, line_width: f32, axis_pattern: u32) -> bool {
    let on_axis: bool = axis < line_width && axis > -line_width;
    return
        (on_axis && part_axis < 0 && (axis_pattern & (1u << 0u)) != 0) ||
        (on_axis && part_axis > 0 && (axis_pattern & (1u << 1u)) != 0);
}
