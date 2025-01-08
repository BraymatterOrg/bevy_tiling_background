use bevy::core_pipeline::fullscreen_vertex_shader::FULLSCREEN_SHADER_HANDLE;
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::{
    PrimitiveState, RenderPipelineDescriptor, SpecializedMeshPipelineError,
};
use bevy::sprite::Material2dKey;
use bevy::{
    color::palettes::css,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};
use bevy_tiling_background::{
    BackgroundMaterial, BackgroundMovementScale, CustomBackgroundImageBundle, ScrollingBackground,
    SetImageRepeatingExt, TilingBackgroundPlugin,
};

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilingBackgroundPlugin::<CustomMaterial>::default())
        // Not actually used, putting this here to test the shader_loading flags
        .add_plugins(TilingBackgroundPlugin::<BackgroundMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, movement)
        .run();
}

pub fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    let image = asset_server.load("space_test.png");
    // Queue a command to set the image to be repeating once the image is loaded.
    commands.set_image_repeating(image.clone());

    // Set up a material
    let custom_mat = CustomMaterial {
        movement_scale: -0.15,
        texture: image,
        blend_color: css::CRIMSON.into(),
    };

    // Spawn Camera
    commands.spawn(Camera2d);

    // Spawn Background
    commands
        .spawn(CustomBackgroundImageBundle::with_material(
            custom_mat,
            materials.as_mut(),
        ))
        .insert(BackgroundMovementScale { scale: 0.00 });

    // Instructions
    commands.spawn((
        TextFont::from_font_size(32.0),
        Text::new("Arrow keys to move"),
        Instructions,
        Name::new("Instructions"),
    ));

    // Boxes as a simple environment to compare background movement to.
    commands.spawn((
        Sprite {
            color: Srgba::rgb(0.0, 0.5, 0.0).into(),
            ..default()
        },
        Transform::from_scale(Vec3::new(10000.0, 100.0, 1.0))
            .with_translation(Vec3::new(0.0, -50.0, 1.0)),
        Name::new("Green Box (Ground)"),
    ));
    commands.spawn((
        Sprite {
            color: css::RED.into(),
            ..default()
        },
        Transform::from_scale(Vec3::new(100.0, 100.0, 1.0))
            .with_translation(Vec3::new(0.0, 50.0, 1.0)),
        Name::new("Red Box"),
    ));
}

#[derive(Component)]
struct Instructions;
fn movement(
    mut camera: Query<&mut Transform, With<Camera>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let move_speed = 100.0;
    let mut camera_transform = camera.single_mut();
    if input.pressed(KeyCode::ArrowLeft) {
        camera_transform.translation.x -= time.delta_secs() * move_speed;
    }

    if input.pressed(KeyCode::ArrowRight) {
        camera_transform.translation.x += time.delta_secs() * move_speed;
    }

    if input.pressed(KeyCode::ArrowDown) {
        camera_transform.translation.y -= time.delta_secs() * move_speed;
    }

    if input.pressed(KeyCode::ArrowUp) {
        camera_transform.translation.y += time.delta_secs() * move_speed;
    }
}

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath, Default)]
pub struct CustomMaterial {
    #[uniform(0)]
    pub movement_scale: f32,
    /// This image must have its [`SamplerDescriptor`] address_mode_* fields set to
    /// [`AddressMode::Repeat`].
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,

    #[uniform(0)]
    pub blend_color: LinearRgba,
}

impl Material2d for CustomMaterial {
    fn vertex_shader() -> ShaderRef {
        FULLSCREEN_SHADER_HANDLE.into()
    }
    fn fragment_shader() -> ShaderRef {
        "custombg.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayoutRef,
        _: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive = PrimitiveState::default();
        descriptor.vertex.entry_point = "fullscreen_vertex_shader".into();
        Ok(())
    }
}

impl ScrollingBackground for CustomMaterial {
    fn set_movement(&mut self, movement: f32) {
        self.movement_scale = movement;
    }
}
