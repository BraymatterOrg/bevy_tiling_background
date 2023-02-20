#define_import_path braymatter::bglib

fn scroll(
    texture: texture_2d<f32>,
    texture_sampler: sampler, 
    scale: f32,
    uv: vec2<f32>,
    offset: vec2<f32>,
) -> vec4<f32>{
    // Get the Normalized Device Coordinates.
    // NDC defines the screen space with a range from -1 to 1.
    // This works better when resizing the window as the background keeps it's position relative to other objects.
    //
    // Top Left           Top Right
    // (-1, -1)  ( 0, -1)  ( 1, -1)
    // (-1,  0)  ( 0,  0)  ( 1,  0)
    // (-1,  1)  ( 0,  1)  ( 1,  1)
    // Bottom Left     Bottom Right
    let ndc = (uv - vec2<f32>(0.5, 0.5)) * 2.0;

    let offset = vec2<f32>(-offset.x, offset.y);

    var uv = ndc - (offset * scale);
    let tex_dim = textureDimensions(texture);
    
    uv = uv * ( view.viewport.zw / vec2<f32>(tex_dim) );
    let color = textureSample(texture, texture_sampler, uv  );
    return color;
}