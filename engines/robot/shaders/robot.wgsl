struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
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
var<uniform> robot_color: vec4<f32>;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
) -> VertexOutput {
    var vertex: VertexOutput;
    vertex.position = transform * position;
    vertex.tex_coord = tex_coord;
    return vertex;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    // let tex = textureLoad(r_color, vec2<i32>(vertex.tex_coord * 256.0), 0);
    // let v = f32(tex.x) / 255.0;
    // return vec4<f32>(1.0 - (v * 5.0), 1.0 - (v * 15.0), 1.0 - (v * 50.0), 1.0);

    let subseconds = f32(time_vec.y) / 4294967296.0;
    let time = f32(time_vec.x) + subseconds;

    let y = abs(vertex.tex_coord.y);
    let phase = (vertex.tex_coord.x * 2 - subseconds * 2.0 + y) * 1 * 3.1416;
    // let r = fract((vertex.tex_coord.x - subseconds * 2.0 + y) * 1);

    let is_border = abs(vertex.tex_coord.y) > 0.8 || abs(vertex.tex_coord.x) > 0.9;

    let darken = select(0.1, 0.8, is_border);
    let dark = mix(robot_color, vec4(0.0, 0.0, 0.0, 1.0), darken);
    let bright = mix(robot_color, vec4(1.0, 1.0, 1.0, 1.0), 0.1);

    return mix(dark, bright, 0.5 + 0.5 * sin(phase)); // vec4(r * r, r * r * r, r, 1.0);
}

@fragment
fn fs_wire(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return robot_color;
}
