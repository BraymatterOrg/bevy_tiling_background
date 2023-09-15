#define_import_path braymatter::bglib

fn scroll(
    tex: texture_2d<f32>,
    texture_sampler: sampler,
    scale: f32,
    uv: vec2<f32>,
    offset: vec2<f32>,
    viewport_size: vec2<f32>,
) -> vec4<f32>{
    let offset = vec2<f32>(-offset.x, offset.y);

    var uv = uv - (offset * scale);
    let tex_dim = textureDimensions(tex);

    uv = uv * ( viewport_size / vec2<f32>(tex_dim) );
    let color = textureSample(tex, texture_sampler, uv);
    return color;
}