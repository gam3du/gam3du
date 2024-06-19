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
    if vertex.tex_coord.x > 0.02 && vertex.tex_coord.x < 0.98 && vertex.tex_coord.y > 0.02 && vertex.tex_coord.y < 0.98 {
        return vec4<f32>(vertex.tex_coord.xy * 0.7, 0.3, 1.0);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
}
