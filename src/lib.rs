mod material;

use bevy::app::{App, Plugin};
use bevy::asset::{load_internal_asset, LoadState};
use bevy::ecs::system::Command;

const TILED_BG_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 429593476423978);

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
            .insert_resource(UpdateSamplerRepeating::default())
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
    /// This image must have its [`SamplerDescriptor`] address_mode_* fields set to
    /// [`AddressMode::Repeat`].
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
struct UpdateSamplerRepeating(Vec<Handle<Image>>);

fn queue_update_sampler(
    query: Query<&Handle<Image>, Added<Handle<BackgroundMaterial>>>,
    mut update_samplers: ResMut<UpdateSamplerRepeating>,
) {
    for handle in query.iter() {
        update_samplers.0.push(handle.clone());
    }
}

fn update_sampler_on_loaded_system(
    asset_server: Res<AssetServer>,
    mut update_sampler: ResMut<UpdateSamplerRepeating>,
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

                // If it already has a custom descriptor, update it otherwise create our own.
                if let ImageSampler::Descriptor(descriptor) = &mut bg_texture.sampler_descriptor {
                    descriptor.address_mode_u = AddressMode::Repeat;
                    descriptor.address_mode_v = AddressMode::Repeat;
                    descriptor.address_mode_w = AddressMode::Repeat;
                } else {
                    bg_texture.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                        address_mode_u: AddressMode::Repeat,
                        address_mode_v: AddressMode::Repeat,
                        address_mode_w: AddressMode::Repeat,
                        ..default()
                    });
                }
                update_sampler.0.remove(index);
                debug!("Updated image sampler to be repeating");
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
        image: Handle<Image>,
        background_materials: &mut Assets<BackgroundMaterial>,
        meshes: &mut Assets<Mesh>,
    ) -> Self {
        Self {
            material: background_materials.add(BackgroundMaterial { texture: image }),
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

struct SetImageRepeatingCommand {
    image: Handle<Image>,
}

impl Command for SetImageRepeatingCommand {
    fn write(self, world: &mut World) {
        let mut samplers = world.resource_mut::<UpdateSamplerRepeating>();
        samplers.0.push(self.image);
    }
}

pub trait SetImageRepeatingExt {
    fn set_image_repeating(&mut self, image: Handle<Image>);
}

impl<'w, 's> SetImageRepeatingExt for Commands<'w, 's> {
    /// Queues this image to have it's [`SamplerDescriptor`] changed to be repeating once the
    /// image is loaded. This may take more than a frame to apply.
    fn set_image_repeating(&mut self, image: Handle<Image>) {
        self.add(SetImageRepeatingCommand { image })
    }
}
