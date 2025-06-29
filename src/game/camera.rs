use bevy::prelude::*;

#[derive(Component)]
#[require(Camera2d)]
pub struct MainCamera;

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(MainCamera);
}
