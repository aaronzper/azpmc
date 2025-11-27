// See rendering::camera
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

// See rendering::light
struct LightUniform {
    view_proj: mat4x4<f32>,
    direction: vec3<f32>,
    color: vec3<f32>,
};
@group(2) @binding(0) 
var<uniform> sun: LightUniform;

// See rendering::vertex::Vertex;
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) texture_cords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) block: vec3<i32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_cords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) light_position: vec4<f32>,
    @location(4) block: vec3<i32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.texture_cords = in.texture_cords;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.world_normal = in.normal;
    out.world_position = in.position;
    out.light_position = sun.view_proj * vec4<f32>(in.position, 1.0);
    out.block = in.block;

    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(3) @binding(0)
var shadow_map: texture_depth_2d;
@group(3) @binding(1)
var shadow_sampler: sampler_comparison;

@group(4) @binding(0)
var<uniform> highlighted_block: vec3<i32>;

const SHADOW_BIAS: f32 = 1.00;

fn compute_shadow(light_pos: vec4<f32>) -> f32 {
    // 1. If behind the light, don't shadow
    if (light_pos.w <= 0.0) {
        return 1.0;
    }

    // 2. Go to light NDC
    let lp = light_pos / light_pos.w;

    // 3. NDC xy [-1,1] → uv [0,1]
    let uv = vec2(lp.x * 0.5 + 0.5, 1.0 - (lp.y * 0.5 + 0.5));

    // 4. Outside the shadow map → treat as unshadowed
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) {
        return 1.0;
    }

    // 5. lp.y is already 0..1 because your Sun VP uses OPENGL_TO_WGPU_MATRIX
    let depth = clamp(lp.z, 0.0, 1.0);

    // 6. Comparison: returns 1 = fully lit, 0 = fully in shadow
    let vis = textureSampleCompare(shadow_map, shadow_sampler, uv, depth);

    return vis;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base_color = textureSample(t_diffuse, s_diffuse, in.texture_cords);

    let lp = in.light_position / in.light_position.w;

    let shadow = compute_shadow(in.light_position);    

    let ambient_strength = 0.15;
    let ambient_color = sun.color * ambient_strength;

    let sun_dir = normalize(-sun.direction);

    let diffuse_strength = max(dot(in.world_normal, sun_dir), 0.0) * shadow;
    let diffuse_color = sun.color * diffuse_strength;
    
    let lighting = ambient_color + diffuse_color;
    var final_color = lighting * base_color.xyz;

    if all(highlighted_block == in.block) {
        let white = vec3<f32>(1.0, 1.0, 1.0);
        final_color = 0.85 * final_color + 0.15 * white;
    }

    return vec4<f32>(final_color, base_color.a);
}
