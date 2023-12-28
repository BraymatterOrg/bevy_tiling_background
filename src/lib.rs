use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Once;

use bevy::app::{App, Plugin};
use bevy::asset::{load_internal_asset, LoadState};
use bevy::core_pipeline::fullscreen_vertex_shader::FULLSCREEN_SHADER_HANDLE;
use bevy::ecs::system::Command;
use bevy::prelude::*;
use bevy::render::mesh::MeshVertexBufferLayout;
use bevy::render::render_resource::{
    AsBindGroup, PrimitiveState, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};
use bevy::render::texture::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::sprite::{Material2d, Material2dKey, Material2dPlugin, Mesh2dHandle};
use bevy::window::{PrimaryWindow, WindowResized};

pub const TILED_BG_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(429593476423978);

pub const BGLIB_HANDLE: Handle<Shader> = Handle::weak_from_u128(429593476423988);

pub const BG_MESH_HANDLE: Handle<Mesh> = Handle::weak_from_u128(12316584166263728426);

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

    // This is doing the same thing as `load_internal_asset` just not from a file.
    let mut meshes = app.world.resource_mut::<Assets<Mesh>>();
    meshes.insert(
        BG_MESH_HANDLE,
        Mesh::from(shape::Quad {
            size: Vec2 { x: 1., y: 1. },
            ..default()
        }),
    );
}

/// Bevy plugin for tiling backgrounds.
///
/// Insert after Bevy's DefaultPlugins.
#[derive(Default)]
pub struct TilingBackgroundPlugin<T: AsBindGroup + Send + Sync + Clone + Asset + Sized + 'static> {
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

        app.add_plugins(Material2dPlugin::<T>::default())
            .register_type::<BackgroundMovementScale>()
            .insert_resource(UpdateSamplerRepeating::default())
            .add_systems(PostUpdate, Self::on_window_resize)
            .add_systems(Update, Self::on_background_added)
            .add_systems(Update, Self::queue_update_sampler)
            .add_systems(Update, Self::update_movement_scale_system)
            .add_systems(Update, update_sampler_on_loaded_system);
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
        events.read().for_each(|ev| {
            for mut transform in backgrounds.iter_mut() {
                transform.scale.x = ev.width;
                transform.scale.y = ev.height;
            }
        });
    }

    pub fn on_background_added(
        windows: Query<&Window, With<PrimaryWindow>>,
        mut backgrounds: Query<&mut Transform, Added<Handle<T>>>,
    ) {
        if let Ok(window) = windows.get_single() {
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

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath, Default)]
pub struct BackgroundMaterial {
    #[uniform(0)]
    pub movement_scale: f32,
    /// webgl2 requires 16 byte alignment
    #[uniform(0)]
    pub _wasm_padding: Vec3,
    /// This image must have its [`SamplerDescriptor`] address_mode_* fields set to
    /// [`AddressMode::Repeat`].
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl Material2d for BackgroundMaterial {
    fn vertex_shader() -> ShaderRef {
        FULLSCREEN_SHADER_HANDLE.into()
    }
    fn fragment_shader() -> ShaderRef {
        TILED_BG_SHADER_HANDLE.into()
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
            Some(LoadState::Failed) => {
                // Failed to load, don't need to keep checking it
                update_sampler.0.remove(index);
            }
            Some(LoadState::Loaded) => {
                let bg_texture = images
                    .get_mut(&handle)
                    .expect("the image should be loaded at this point");

                // If it already has a custom descriptor, update it otherwise create our own.
                if let ImageSampler::Descriptor(descriptor) = &mut bg_texture.sampler {
                    descriptor.address_mode_u = ImageAddressMode::Repeat;
                    descriptor.address_mode_v = ImageAddressMode::Repeat;
                    descriptor.address_mode_w = ImageAddressMode::Repeat;
                } else {
                    bg_texture.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        address_mode_w: ImageAddressMode::Repeat,
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
    pub view_visibility: ViewVisibility,
    pub inherited_visibility: InheritedVisibility,
    pub movement_scale: BackgroundMovementScale,
}

impl<T: Material2d + ScrollingBackground> CustomBackgroundImageBundle<T> {
    pub fn with_material(material: T, materials: &mut Assets<T>) -> Self {
        Self {
            material: materials.add(material),
            mesh: BG_MESH_HANDLE.into(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            view_visibility: Default::default(),
            inherited_visibility: Default::default(),
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
    pub view_visibility: ViewVisibility,
    pub inherited_visibility: InheritedVisibility,
    pub movement_scale: BackgroundMovementScale,
}

impl BackgroundImageBundle {
    pub fn from_image(
        image: Handle<Image>,
        background_materials: &mut Assets<BackgroundMaterial>,
    ) -> Self {
        Self {
            material: background_materials.add(BackgroundMaterial {
                texture: image,
                movement_scale: 1.0,
                _wasm_padding: Vec3::ZERO,
            }),
            mesh: BG_MESH_HANDLE.into(),
            transform: Default::default(),
            global_transform: Default::default(),
            visibility: Default::default(),
            view_visibility: Default::default(),
            inherited_visibility: Default::default(),
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
    fn apply(self, world: &mut World) {
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
