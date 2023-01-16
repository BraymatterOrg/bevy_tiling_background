#define_import_path braymatter::scrolling_background

#import bevy_sprite::mesh2d_view_bindings
#import bevy_sprite::mesh2d_bindings
#import bevy_sprite::mesh2d_functions
#import bevy_pbr::utils

struct Uniforms {
    scale: f32,
};

@group(1) @binding(0)
var<uniform> uniforms: Uniforms;
@group(1) @binding(1)
var texture: texture_2d<f32>;
@group(1) @binding(2)
var texture_sampler: sampler;

fn scroll(
    texture: texture_2d<f32>,
    texture_sampler: sampler, 
    scale: f32,
    uv: vec2<f32>,
    offset: vec2<f32>,
) -> vec4<f32>{
    let offset = vec2<f32>(-offset.x, offset.y);
    
    var uv = (uv - offset * scale);
    let tex_dim = textureDimensions(texture);
    
    uv = uv * ( view.viewport.zw / vec2<f32>(tex_dim) );
    let color = textureSample(texture, texture_sampler, uv  );
    return color;
}

@fragment
fn fragment(
    @builtin(position) position: vec4<f32>,
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    let scale = uniforms.scale;
    let offset = mesh2d_position_world_to_clip(vec4<f32>(view.world_position.xy, 0.0, 0.0)).xy;
    let color = scroll(texture, texture_sampler, scale, uv, offset);
    return color;
}

