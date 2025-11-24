// See rendering::light
struct LightUniform {
    view_proj: mat4x4<f32>,
    direction: vec3<f32>,
    color: vec3<f32>,
};
@group(0) @binding(0)
var<uniform> sun: LightUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) texture_cords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vs_shadow(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_pos = vec4<f32>(in.position, 1.0);
    out.position = sun.view_proj * world_pos;

    return out;
}
