use bevy::{color::palettes::css, prelude::*};
use bevy_tiling_background::{
    BackgroundImageBundle, BackgroundMaterial, BackgroundMovementScale, SetImageRepeatingExt,
    TilingBackgroundPlugin,
};

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_linear()))
        .add_plugins(TilingBackgroundPlugin::<BackgroundMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, movement)
        .add_systems(Update, update_instructions)
        .add_systems(PostUpdate, update_movement_scale_system)
        .run();
}

pub fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
) {
    let image = asset_server.load("test.png");
    // Queue a command to set the image to be repeating once the image is loaded.
    commands.set_image_repeating(image.clone());

    commands.spawn(Camera2dBundle::default());

    commands.spawn(BackgroundImageBundle::from_image(image, materials.as_mut()).at_z_layer(0.1));

    // Instructions
    commands.spawn((
        TextBundle::from_section(
            "Arrow keys to move\n\
        +/- for Parallax effect",
            TextStyle {
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
                color: Srgba::rgb(0.0, 0.5, 0.0).into(),
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
                color: css::RED.into(),
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
    mut camera: Query<&mut Transform, With<Camera>>,
    mut background_scales: Query<&mut BackgroundMovementScale>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let move_speed = 100.0;
    let mut camera_transform = camera.single_mut();
    if input.pressed(KeyCode::ArrowLeft) {
        camera_transform.translation.x -= time.delta_seconds() * move_speed;
    }

    if input.pressed(KeyCode::ArrowRight) {
        camera_transform.translation.x += time.delta_seconds() * move_speed;
    }

    if input.pressed(KeyCode::ArrowDown) {
        camera_transform.translation.y -= time.delta_seconds() * move_speed;
    }

    if input.pressed(KeyCode::ArrowUp) {
        camera_transform.translation.y += time.delta_seconds() * move_speed;
    }

    for mut background_scale in background_scales.iter_mut() {
        if input.pressed(KeyCode::Equal) || input.pressed(KeyCode::NumpadAdd) {
            background_scale.scale += time.delta_seconds();
        }

        if input.pressed(KeyCode::Minus) || input.pressed(KeyCode::NumpadSubtract) {
            background_scale.scale -= time.delta_seconds();
        }
    }
}

fn update_instructions(
    mut query: Query<&mut Text, With<Instructions>>,
    background_movement: Query<&BackgroundMovementScale>,
) {
    let mut instructions = query.single_mut();
    instructions.sections.first_mut().unwrap().value = format!(
        "Arrow keys to move\n\
        +/- to change parallax \n\
        Current parallax multiplier {}",
        background_movement.single().scale
    );
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
