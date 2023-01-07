#import bevy_sprite::mesh2d_view_bindings
#import bevy_sprite::mesh2d_bindings
#import bevy_sprite::mesh2d_functions
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
    let scale = 0.15;
    let offset = mesh2d_position_world_to_clip(vec4<f32>(view.world_position.xy, 0.0, 0.0)).xy;
    let offset = vec2<f32>(-offset.x, offset.y);

    var uv = (uv - offset * scale);
    let tex_dim = textureDimensions(texture);

    uv = uv * ( view.viewport.zw / vec2<f32>(tex_dim) );
    let color = textureSample(texture, texture_sampler, uv  );
    return color;
}