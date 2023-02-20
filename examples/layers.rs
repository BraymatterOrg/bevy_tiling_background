use bevy::prelude::*;
use bevy_tiling_background::{
    BackgroundImageBundle, BackgroundMaterial, BackgroundMovementScale, SetImageRepeatingExt,
    TilingBackgroundPlugin,
};

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TilingBackgroundPlugin::<BackgroundMaterial>::default())
        .add_startup_system(setup)
        .add_system(movement)
        .run()
}

pub fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let image = asset_server.load("space_test.png");
    // Queue a command to set the image to be repeating once the image is loaded.
    commands.set_image_repeating(image.clone());

    let front_layer = asset_server.load("space_dust_transparent.png");
    // Queue a command to set the image to be repeating once the image is loaded.
    commands.set_image_repeating(front_layer.clone());

    // Spawn camera
    commands.spawn(Camera2dBundle::default());

    // Spawn backgrounds
    commands.spawn(
        BackgroundImageBundle::from_image(image, materials.as_mut(), meshes.as_mut())
            .at_z_layer(0.1),
    );
    commands.spawn(
        BackgroundImageBundle::from_image(front_layer, materials.as_mut(), meshes.as_mut())
            .at_z_layer(2.1)
            .with_movement_scale(1.1),
    );

    // Instructions
    commands.spawn((
        TextBundle::from_section(
            "Arrow keys to move\n",
            TextStyle {
                font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                font_size: 32.0,
                ..default()
            },
        ),
        Instructions,
    ));

    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("ship.png"),

            transform: Transform::from_scale(Vec3::new(1.0, 1.0, 1.0))
                .with_translation(Vec3::new(0.0, 50.0, 1.0)),
            ..default()
        })
        .insert(Player);
}

#[derive(Component)]
struct Instructions;

#[derive(Component)]
struct Player;

fn movement(
    mut camera: Query<&mut Transform, With<Camera>>,
    mut sprite_transform: Query<(&mut Transform, &Player), Without<Camera>>,
    mut background_scales: Query<&mut BackgroundMovementScale>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let move_speed = 100.0;
    let mut camera_transform = camera.single_mut();
    let (mut sprite, _player) = sprite_transform.single_mut();
    if input.pressed(KeyCode::Left) {
        camera_transform.translation.x -= time.delta_seconds() * move_speed;
        sprite.translation.x -= time.delta_seconds() * move_speed;
    }

    if input.pressed(KeyCode::Right) {
        camera_transform.translation.x += time.delta_seconds() * move_speed;
        sprite.translation.x += time.delta_seconds() * move_speed;
    }

    if input.pressed(KeyCode::Down) {
        camera_transform.translation.y -= time.delta_seconds() * move_speed;
        sprite.translation.y -= time.delta_seconds() * move_speed;
    }

    if input.pressed(KeyCode::Up) {
        camera_transform.translation.y += time.delta_seconds() * move_speed;
        sprite.translation.y += time.delta_seconds() * move_speed;
    }

    for mut background_scale in background_scales.iter_mut() {
        if input.pressed(KeyCode::Plus) || input.pressed(KeyCode::NumpadAdd) {
            background_scale.scale += time.delta_seconds();
        }

        if input.pressed(KeyCode::Minus) || input.pressed(KeyCode::NumpadSubtract) {
            background_scale.scale -= time.delta_seconds();
        }
    }
}
