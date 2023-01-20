use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};
use bevy_tiling_background::{
    CustomBackgroundImageBundle, SetImageRepeatingExt, TilingBackgroundPlugin,
};

/// Bevy doesn't render things that are attached to the camera, so this component will be used
/// on a parent entity to move our camera and background.
#[derive(Component)]
pub struct CameraRig;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TilingBackgroundPlugin::<CustomMaterial>::default())
        .add_startup_system(setup)
        .add_system(movement)
        .run()
}

pub fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let image = asset_server.load("space_test.png");
    // Queue a command to set the image to be repeating once the image is loaded.
    commands.set_image_repeating(image.clone());

    //Set up a material
    let custom_mat = CustomMaterial {
        movement_scale: 0.15,
        texture: image,
        blend_color: Color::CRIMSON,
    };

    // Spawn camera rig with camera and background as children
    commands
        .spawn((CameraRig, SpatialBundle::default()))
        .with_children(|child_builder| {
            child_builder.spawn(Camera2dBundle::default());
            child_builder.spawn(CustomBackgroundImageBundle::with_material(
                custom_mat,
                materials.as_mut(),
                meshes.as_mut(),
            ));
        });

    // Instructions
    commands.spawn((
        TextBundle::from_section(
            "Arrow keys to move",
            TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 32.0,
                ..default()
            },
        ),
        Instructions,
        Name::new("Instructions"),
    ));

    // Boxes as a simple environment to compare background movement to.
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::DARK_GREEN,
                ..default()
            },
            transform: Transform::from_scale(Vec3::new(10000.0, 100.0, 1.0))
                .with_translation(Vec3::new(0.0, -50.0, 1.0)),
            ..default()
        },
        Name::new("Green Box (Ground)"),
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::RED,
                ..default()
            },
            transform: Transform::from_scale(Vec3::new(100.0, 100.0, 1.0))
                .with_translation(Vec3::new(0.0, 50.0, 1.0)),
            ..default()
        },
        Name::new("Red Box"),
    ));
}

#[derive(Component)]
struct Instructions;
fn movement(
    mut camera: Query<&mut Transform, With<CameraRig>>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let move_speed = 100.0;
    let mut camera_transform = camera.single_mut();
    if input.pressed(KeyCode::Left) {
        camera_transform.translation.x -= time.delta_seconds() * move_speed;
    }

    if input.pressed(KeyCode::Right) {
        camera_transform.translation.x += time.delta_seconds() * move_speed;
    }

    if input.pressed(KeyCode::Down) {
        camera_transform.translation.y -= time.delta_seconds() * move_speed;
    }

    if input.pressed(KeyCode::Up) {
        camera_transform.translation.y += time.delta_seconds() * move_speed;
    }
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid, Default)]
#[uuid = "09756d79-32e9-4dc4-bb95-b373370815e3"]
pub struct CustomMaterial {
    /// This image must have its [`SamplerDescriptor`] address_mode_* fields set to
    /// [`AddressMode::Repeat`].
    #[uniform(0)]
    pub movement_scale: f32,

    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,

    #[uniform(0)]
    pub blend_color: Color,
}

impl Material2d for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "custombg.wgsl".into()
    }
}
