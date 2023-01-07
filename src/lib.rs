mod material;

use bevy::app::{App, Plugin};
use bevy::asset::{load_internal_asset, LoadState};

const TILED_BG_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 929599476923908);

#[derive(Default)]
pub struct TilingBackgroundPlugin;

impl Plugin for TilingBackgroundPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            TILED_BG_SHADER_HANDLE,
            "shaders/background.wgsl",
            Shader::from_wgsl
        );
        app.add_plugin(Material2dPlugin::<BackgroundMaterial>::default())
            .insert_resource(UpdateSampler::default())
            .add_system(queue_update_sampler)
            .add_system(update_sampler_on_loaded_system);
    }
}

use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AddressMode, AsBindGroup, SamplerDescriptor, ShaderRef};
use bevy::render::texture::ImageSampler;
use bevy::sprite::{Material2d, Material2dPlugin, Mesh2dHandle};

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "4e31d7bf-a3f5-4a62-a86f-1e61a21076db"]
pub struct BackgroundMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub(crate) texture: Handle<Image>,
}

impl Material2d for BackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        TILED_BG_SHADER_HANDLE.typed().into()
    }
}

/// A queue of images that need their sampler updated when they are loaded.
#[derive(Resource, Default)]
struct UpdateSampler(Vec<Handle<Image>>);

fn queue_update_sampler(
    query: Query<&Handle<Image>, Added<Handle<BackgroundMaterial>>>,
    mut update_samplers: ResMut<UpdateSampler>,
) {
    for handle in query.iter() {
        update_samplers.0.push(handle.clone());
    }
}

fn update_sampler_on_loaded_system(
    asset_server: Res<AssetServer>,
    mut update_sampler: ResMut<UpdateSampler>,
    mut images: ResMut<Assets<Image>>,
) {
    // Iterating over them backwards so removing one doesn't offset the index of the rest
    let handles = update_sampler
        .0
        .iter()
        .cloned()
        .enumerate()
        .rev()
        .collect::<Vec<_>>();
    for (index, handle) in handles {
        match asset_server.get_load_state(&handle) {
            LoadState::Failed => {
                // one of our assets had an error
            }
            LoadState::Loaded => {
                let mut bg_texture = images
                    .get_mut(&handle)
                    .expect("the image should be loaded at this point");

                bg_texture.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                    address_mode_u: AddressMode::Repeat,
                    address_mode_v: AddressMode::Repeat,
                    address_mode_w: AddressMode::Repeat,
                    ..default()
                });
                update_sampler.0.remove(index);
            }
            _ => {
                // NotLoaded/Loading: not fully ready yet
            }
        }
    }
}

#[derive(Bundle)]
pub struct BackgroundImageBundle {
    pub material: Handle<BackgroundMaterial>,
    pub mesh: Mesh2dHandle,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

impl BackgroundImageBundle {
    pub fn from_image(
        texture: Handle<Image>,
        background_materials: &mut Assets<BackgroundMaterial>,
        meshes: &mut Assets<Mesh>,
    ) -> Self {
        Self {
            material: background_materials.add(BackgroundMaterial { texture }),
            mesh: meshes
                .add(Mesh::from(shape::Quad {
                    size: Vec2 { x: 1600., y: 1600. },
                    ..default()
                }))
                .into(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            computed_visibility: Default::default(),
        }
    }
}
