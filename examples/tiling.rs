use bevy::app::App;
use bevy::asset::Assets;
use bevy::prelude::{AssetServer, Camera2dBundle, Commands, Mesh, ResMut};
use bevy::DefaultPlugins;
use bevy_tiling_background::{BackgroundImageBundle, BackgroundMaterial, TilingBackgroundPlugin};

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TilingBackgroundPlugin)
        .add_startup_system(setup)
        .run()
}

pub fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(BackgroundImageBundle::from_image(
        asset_server.load("test.png"),
        materials.as_mut(),
        meshes.as_mut(),
    ));
}
