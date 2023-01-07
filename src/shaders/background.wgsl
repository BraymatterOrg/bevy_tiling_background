#import bevy_sprite::mesh2d_view_bindings
#import bevy_sprite::mesh2d_bindings
#import bevy_pbr::utils

@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var texture_sampler: sampler;

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    return textureSample(texture, texture_sampler, uv);
}