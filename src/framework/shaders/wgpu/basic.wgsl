struct CameraUniform {
    proj_mtx: mat4x4<f32>,
}

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(vtx: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = u_cam.proj_mtx * vec4<f32>(vtx.position, 0.0, 1.0);
    out.color = vtx.color;
    out.tex_coords = vtx.tex_coords;
    return out;
}

@group(0) @binding(0)
var u_texture: texture_2d<f32>;
@group(0) @binding(1)
var u_sampler: sampler;

@group(1) @binding(0)
var<uniform> u_cam: CameraUniform;

@fragment
fn fs_main_textured(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(u_texture, u_sampler, in.tex_coords) * in.color;
}

@fragment
fn fs_main_colored(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
