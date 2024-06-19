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
    
    //vertex.tex_coord.x + vertex.tex_coord.y < 1 ?
    //    vec4<f32>(0.0, 0.0, 0.0, 1.0) :
    //    vec4<f32>(abs(sin((vertex.tex_coord.x * 3.1416 * 10))), abs(sin((vertex.tex_coord.y * 3.1416 * 10))), 0.5, 1.0);

    let subseconds = f32(time_vec.y) / 4294967296.0;
    let time = f32(time_vec.x) + subseconds;

    if vertex.tex_coord.x + vertex.tex_coord.y < sin((vertex.tex_coord.x + time * 0.1) * 3.1416 * 10) {
        return vec4<f32>(sin(cos(vertex.tex_coord.y * 3.1416 * 10)), 0.0, 0.0, 1.0);
    } else {
        return vec4<f32>(abs(sin((vertex.tex_coord.x * 3.1416 * 10))), abs(sin(((vertex.tex_coord.y + time * 0.25) * 3.1416 * 10))), 0.5, 1.0);
    }

    //let result = (vertex.tex_coord.x + vertex.tex_coord.y < 1) ?
    //    vec4<f32>(0.0, 0.0, 0.0, 1.0) :
    //    vec4<f32>(abs(sin((vertex.tex_coord.x * 3.1416 * 10))), abs(sin((vertex.tex_coord.y * 3.1416 * 10))), 0.5, 1.0);
//
    //return result;
}

@fragment
fn fs_wire(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 0.5, 0.0, 0.5);
}
