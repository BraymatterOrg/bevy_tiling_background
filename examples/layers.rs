use bevy::prelude::*;
use bevy_tiling_background::{
    BackgroundImageBundle, BackgroundMaterial, BackgroundMovementScale, SetImageRepeatingExt,
    TilingBackgroundPlugin,
};

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TilingBackgroundPlugin::<BackgroundMaterial>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, movement)
        .run();
}

pub fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
) {
    let image = asset_server.load("space_test.png");
    // Queue a command to set the image to be repeating once the image is loaded.
    commands.set_image_repeating(image.clone());

    let front_layer = asset_server.load("space_dust_transparent.png");
    // Queue a command to set the image to be repeating once the image is loaded.
    commands.set_image_repeating(front_layer.clone());

    // Spawn camera
    commands.spawn(Camera2d);

    // Spawn backgrounds
    commands.spawn(BackgroundImageBundle::from_image(image, materials.as_mut()).at_z_layer(0.1));
    commands.spawn(
        BackgroundImageBundle::from_image(front_layer, materials.as_mut())
            .at_z_layer(2.1)
            .with_movement_scale(1.1),
    );

    // Instructions
    commands.spawn((
        TextFont::from_font_size(32.0),
        Text::new("Arrow keys to move\n"),
        Instructions,
    ));

    commands
        .spawn((
            Sprite::from_image(asset_server.load("ship.png")),
            Transform::from_scale(Vec3::new(1.0, 1.0, 1.0))
                .with_translation(Vec3::new(0.0, 50.0, 1.0)),
        ))
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
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let move_speed = 100.0;
    let mut camera_transform = camera.single_mut();
    let (mut sprite, _player) = sprite_transform.single_mut();
    if input.pressed(KeyCode::ArrowLeft) {
        camera_transform.translation.x -= time.delta_secs() * move_speed;
        sprite.translation.x -= time.delta_secs() * move_speed;
    }

    if input.pressed(KeyCode::ArrowRight) {
        camera_transform.translation.x += time.delta_secs() * move_speed;
        sprite.translation.x += time.delta_secs() * move_speed;
    }

    if input.pressed(KeyCode::ArrowDown) {
        camera_transform.translation.y -= time.delta_secs() * move_speed;
        sprite.translation.y -= time.delta_secs() * move_speed;
    }

    if input.pressed(KeyCode::ArrowUp) {
        camera_transform.translation.y += time.delta_secs() * move_speed;
        sprite.translation.y += time.delta_secs() * move_speed;
    }

    for mut background_scale in background_scales.iter_mut() {
        if input.pressed(KeyCode::Equal) || input.pressed(KeyCode::NumpadAdd) {
            background_scale.scale += time.delta_secs();
        }

        if input.pressed(KeyCode::Minus) || input.pressed(KeyCode::NumpadSubtract) {
            background_scale.scale -= time.delta_secs();
        }
    }
}
