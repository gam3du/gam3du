struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) base_color_factor: vec4<f32>,
    @location(3) base_color_texture_coordinates: vec2<f32>,
};

@group(0)
@binding(0)
var<uniform> world: mat4x4<f32>;

@group(0)
@binding(1)
var<uniform> camera: mat4x4<f32>;

@group(0)
@binding(2)
var<uniform> projection: mat4x4<f32>;

@group(0)
@binding(3)
var r_color: texture_2d<u32>;

@group(0)
@binding(4)
var<uniform> time_vec: vec4<u32>;

@group(0)
@binding(5)
var<uniform> robot_color: vec4<f32>;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) base_color_factor: vec4<f32>,
    @location(3) base_color_texture_coordinates: vec2<f32>,
) -> VertexOutput {
    var vertex: VertexOutput;
    vertex.position = projection * camera * world * position;
    let scale_rotation = mat4x4<f32>(world[0], world[1], world[2], vec4(0.0, 0.0, 0.0, 1.0));
    vertex.normal = scale_rotation * normal;
    vertex.base_color_factor = base_color_factor;
    vertex.base_color_texture_coordinates = base_color_texture_coordinates;
    return vertex;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    // let tex = textureLoad(r_color, vec2<i32>(vertex.base_color_texture_coordinates * 256.0), 0);
    // let v = f32(tex.x) / 255.0;
    // return vec4<f32>(1.0 - (v * 5.0), 1.0 - (v * 15.0), 1.0 - (v * 50.0), 1.0);

    let subseconds = f32(time_vec.y) / 4294967296.0;
    let time = f32(time_vec.x) + subseconds;

    let y = abs(vertex.base_color_texture_coordinates.y * 8.0);
    let phase = (vertex.base_color_texture_coordinates.x * 2 * 8 - subseconds * 2.0 + y) * 1 * 3.1416;
    // let r = fract((vertex.base_color_texture_coordinates.x * 8 - subseconds * 2.0 + y) * 1);

    let is_border = abs(vertex.base_color_texture_coordinates.y) > 0.8 || abs(vertex.base_color_texture_coordinates.x) > 0.9;

    let darken = select(0.1, 0.8, is_border);
    let dark = mix(robot_color, vec4(0.0, 0.0, 0.0, 1.0), darken);
    let bright = mix(robot_color, vec4(1.0, 1.0, 1.0, 1.0), 0.1);

    let light = max(dot(normalize(vertex.normal.xyz), normalize(vec3(-1.0, -1.0, 1.0))), 0.0);
    let specular = light * light * light * light * light * light; 

    let color = mix(dark, bright, 0.5 + 0.5 * sin(phase)) * max(light, 0.1); // vec4(r * r, r * r * r, r, 1.0);
    return mix(color, vec4(1.0, 1.0, 1.0, 1.0), specular);
}

@fragment
fn fs_wire(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return robot_color;
}
