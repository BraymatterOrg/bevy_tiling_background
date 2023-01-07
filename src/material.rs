use crate::TILED_BG_SHADER_HANDLE;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::Material2d;

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "4e31d7bf-a3f5-4a62-a86f-1e61a21076db"]
pub struct TilingBackgroundMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub(crate) texture: Handle<Image>,
}

impl Material2d for TilingBackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        TILED_BG_SHADER_HANDLE.typed().into()
    }
}

#[derive(Component)]
pub struct BackgroundImage {
    pub mat: Handle<TilingBackgroundMaterial>,
    pub mesh: Handle<Mesh>,
}
