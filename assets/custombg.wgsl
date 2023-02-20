#import bevy_core_pipeline::fullscreen_vertex_shader
#import bevy_sprite::mesh2d_view_bindings
#import bevy_sprite::mesh2d_bindings
#import bevy_sprite::mesh2d_functions
#import bevy_pbr::utils


#import braymatter::bglib

struct Uniforms {
    scale: f32,
    blend_color: vec4<f32>
};

@group(1) @binding(0)
var<uniform> uniforms: Uniforms;
@group(1) @binding(1)
var texture: texture_2d<f32>;
@group(1) @binding(2)
var texture_sampler: sampler;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
) -> @location(0) vec4<f32> {
    let scale = uniforms.scale;
    let offset = mesh2d_position_world_to_clip(vec4<f32>(view.world_position.xy, 0.0, 0.0)).xy;
    let color = scroll(texture, texture_sampler, scale, uv, offset) + uniforms.blend_color;
    return color;
}

