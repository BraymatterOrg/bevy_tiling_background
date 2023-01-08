mod material;

use bevy::app::{App, Plugin};
use bevy::asset::{load_internal_asset, LoadState};
use bevy::ecs::system::Command;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AddressMode, AsBindGroup, SamplerDescriptor, ShaderRef};
use bevy::render::texture::ImageSampler;
use bevy::sprite::{Material2d, Material2dPlugin, Mesh2dHandle};
use bevy::window::WindowResized;

const TILED_BG_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 429593476423978);

/// Bevy plugin for tiling backgrounds.
///
/// Insert after Bevy's DefaultPlugins.
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
            .add_system_to_stage(CoreStage::PostUpdate, on_window_resize)
            .add_system_to_stage(CoreStage::PostUpdate, follow_camera)
            .add_system_to_stage(CoreStage::PostUpdate, update_movement_scale_system)
            .add_system(queue_update_sampler)
            .add_system(update_sampler_on_loaded_system);
    }
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "4e31d7bf-a3f5-4a62-a86f-1e61a21076db"]
pub struct BackgroundMaterial {
    /// This image must have its [`SamplerDescriptor`] address_mode_* fields set to
    /// [`AddressMode::Repeat`].
    #[uniform(0)]
    pub movement_scale: f32,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
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

///Polls the update_sampler resource and swaps the asset's sampler out for a repeating sampler
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
                // Failed to load, don't need to keep checking it
                update_sampler.0.remove(index);
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

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct BackgroundMovementScale {
    pub scale: f32,
}

impl Default for BackgroundMovementScale {
    fn default() -> Self {
        Self { scale: 0.15 }
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
    pub movement_scale: BackgroundMovementScale,
}

impl BackgroundImageBundle {
    pub fn from_image(
        image: Handle<Image>,
        background_materials: &mut Assets<BackgroundMaterial>,
        meshes: &mut Assets<Mesh>,
    ) -> Self {
        Self {
            material: background_materials.add(BackgroundMaterial {
                texture: image,
                movement_scale: 1.0,
            }),
            mesh: meshes
                .add(Mesh::from(shape::Quad {
                    size: Vec2 { x: 1., y: 1. },
                    ..default()
                }))
                .into(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            computed_visibility: Default::default(),
            movement_scale: Default::default(),
        }
    }

    pub fn with_movement_scale(mut self, scale: f32) -> Self {
        self.movement_scale.scale = scale;
        self
    }

    pub fn at_z_layer(mut self, z: f32) -> Self{
        self.transform.translation.z = z;
        self
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

pub fn on_window_resize(
    mut events: EventReader<WindowResized>,
    mut backgrounds: Query<&mut Transform, With<Handle<BackgroundMaterial>>>,
) {
    events.iter().for_each(|ev| {
        for mut transform in backgrounds.iter_mut() {
            transform.scale.x = ev.width;
            transform.scale.y = ev.height;
        }
    });
}

pub fn follow_camera(
    mut backgrounds: Query<&mut Transform, With<Handle<BackgroundMaterial>>>,
    cameras: Query<&GlobalTransform, With<Camera>>,
) {
    for mut transform in backgrounds.iter_mut() {
        let camera = cameras.get_single().expect("Currently only one camera is supported by bevy_tiling_background. Contributions welcome!");
        transform.translation.y = camera.translation().y;
        transform.translation.x = camera.translation().x;
    }
}

pub fn update_movement_scale_system(
    mut query: Query<
        (&mut Handle<BackgroundMaterial>, &BackgroundMovementScale),
        Changed<BackgroundMovementScale>,
    >,
    mut background_materials: ResMut<Assets<BackgroundMaterial>>,
) {
    for (bg_material_handle, scale) in query.iter_mut() {
        if let Some(background_material) = background_materials.get_mut(&*bg_material_handle) {
            background_material.movement_scale = scale.scale;
        }
    }
}
