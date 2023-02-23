use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Once;

use bevy::app::{App, Plugin};
use bevy::asset::{load_internal_asset, LoadState};
use bevy::core_pipeline::fullscreen_vertex_shader::FULLSCREEN_SHADER_HANDLE;
use bevy::ecs::system::Command;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::mesh::MeshVertexBufferLayout;
use bevy::render::render_resource::{
    AddressMode, AsBindGroup, PrimitiveState, RenderPipelineDescriptor, SamplerDescriptor,
    ShaderRef, SpecializedMeshPipelineError,
};
use bevy::render::texture::ImageSampler;
use bevy::sprite::{Material2d, Material2dKey, Material2dPlugin, Mesh2dHandle};
use bevy::window::WindowResized;

pub const TILED_BG_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 429593476423978);

pub const BGLIB_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 429593476423988);

/// Prevent shaders from being loaded multiple times, emitting events etc.
pub static BEVY_TILING_PLUGIN_SHADERS_LOADED: Once = Once::new();

fn load_plugin_shadercode(app: &mut App) {
    load_internal_asset!(
        app,
        TILED_BG_SHADER_HANDLE,
        "shaders/background.wgsl",
        Shader::from_wgsl
    );

    load_internal_asset!(app, BGLIB_HANDLE, "shaders/bglib.wgsl", Shader::from_wgsl);
}

/// Bevy plugin for tiling backgrounds.
///
/// Insert after Bevy's DefaultPlugins.
#[derive(Default, TypeUuid)]
#[uuid = "14268b6c-927e-41e3-affe-410e7609a3fa"]
pub struct TilingBackgroundPlugin<T: AsBindGroup + Send + Sync + Clone + TypeUuid + Sized + 'static>
{
    _phantom: PhantomData<T>,
}

impl<T: Material2d + AsBindGroup + Clone + ScrollingBackground> Plugin for TilingBackgroundPlugin<T>
where
    T::Data: Clone + Eq + Send + Sync + Clone + Sized + Hash,
{
    fn build(&self, app: &mut App) {
        BEVY_TILING_PLUGIN_SHADERS_LOADED.call_once(|| {
            info!("Loading bevy_tiling_background shaders");
            load_plugin_shadercode(app);
        });

        app.add_plugin(Material2dPlugin::<T>::default())
            .register_type::<BackgroundMovementScale>()
            .insert_resource(UpdateSamplerRepeating::default())
            .add_system_to_stage(CoreStage::PostUpdate, Self::on_window_resize)
            .add_system(Self::on_background_added)
            .add_system(Self::queue_update_sampler)
            .add_system(Self::update_movement_scale_system)
            .add_system(update_sampler_on_loaded_system);
    }
}

impl<T: Material2d + AsBindGroup + Clone + ScrollingBackground> TilingBackgroundPlugin<T>
where
    <T as AsBindGroup>::Data: Clone + Eq + Send + Sync + Clone + Sized + Hash,
{
    pub fn new() -> Self {
        TilingBackgroundPlugin::<T> {
            _phantom: PhantomData {},
        }
    }

    pub fn on_window_resize(
        mut events: EventReader<WindowResized>,
        mut backgrounds: Query<&mut Transform, With<Handle<T>>>,
    ) {
        events.iter().for_each(|ev| {
            for mut transform in backgrounds.iter_mut() {
                transform.scale.x = ev.width;
                transform.scale.y = ev.height;
            }
        });
    }

    pub fn on_background_added(
        windows: Res<Windows>,
        mut backgrounds: Query<&mut Transform, Added<Handle<T>>>,
    ) {
        if let Some(window) = windows.get_primary() {
            for mut transform in backgrounds.iter_mut() {
                transform.scale.x = window.width();
                transform.scale.y = window.height();
            }
        };
    }

    fn queue_update_sampler(
        query: Query<&Handle<Image>, Added<Handle<T>>>,
        mut update_samplers: ResMut<UpdateSamplerRepeating>,
    ) {
        for handle in query.iter() {
            update_samplers.0.push(handle.clone());
        }
    }

    pub fn update_movement_scale_system(
        mut query: Query<
            (&mut Handle<T>, &BackgroundMovementScale),
            Changed<BackgroundMovementScale>,
        >,
        mut background_materials: ResMut<Assets<T>>,
    ) {
        for (bg_material_handle, scale) in query.iter_mut() {
            if let Some(background_material) = background_materials.get_mut(&*bg_material_handle) {
                background_material.set_movement(scale.scale);
            }
        }
    }
}

pub trait ScrollingBackground {
    ///Use this as a hook to set the materials movement scale if applicable to your shader.
    fn set_movement(&mut self, movement: f32);
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid, Default, Reflect)]
#[uuid = "4e31d7bf-a3f5-4a62-a86f-1e61a21076db"]
pub struct BackgroundMaterial {
    #[uniform(0)]
    pub movement_scale: f32,
    /// This image must have its [`SamplerDescriptor`] address_mode_* fields set to
    /// [`AddressMode::Repeat`].
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl Material2d for BackgroundMaterial {
    fn vertex_shader() -> ShaderRef {
        FULLSCREEN_SHADER_HANDLE.typed().into()
    }
    fn fragment_shader() -> ShaderRef {
        TILED_BG_SHADER_HANDLE.typed().into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayout,
        _: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive = PrimitiveState::default();
        descriptor.vertex.entry_point = "fullscreen_vertex_shader".into();
        Ok(())
    }
}

impl ScrollingBackground for BackgroundMaterial {
    fn set_movement(&mut self, movement: f32) {
        self.movement_scale = movement;
    }
}

impl ScrollingBackground for &mut BackgroundMaterial {
    fn set_movement(&mut self, movement: f32) {
        self.movement_scale = movement;
    }
}
/// A queue of images that need their sampler updated when they are loaded.
#[derive(Resource, Default)]
struct UpdateSamplerRepeating(Vec<Handle<Image>>);

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
#[reflect(Component, Default)]
pub struct BackgroundMovementScale {
    /// Determines how fast the background will scroll when the camera moves.
    ///
    /// # Examples
    ///
    /// - A scale of 0.0 the background will move with the camera.
    /// - A scale of 1.0 the background will move opposite the camera at the same speed as the camera,
    /// making it stationary in the world.
    /// - A scale of 2.0 the background will move twice as fast as the camera.
    pub scale: f32,
}

impl Default for BackgroundMovementScale {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}

#[derive(Bundle)]
pub struct CustomBackgroundImageBundle<T: Material2d> {
    pub material: Handle<T>,
    pub mesh: Mesh2dHandle,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub movement_scale: BackgroundMovementScale,
}

impl<T: Material2d + ScrollingBackground> CustomBackgroundImageBundle<T> {
    pub fn with_material(
        material: T,
        materials: &mut Assets<T>,
        meshes: &mut Assets<Mesh>,
    ) -> Self {
        Self {
            material: materials.add(material),
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

    pub fn at_z_layer(mut self, z: f32) -> Self {
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
