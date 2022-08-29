use bevy::{audio::AudioPlugin, prelude::*};

mod assets;
mod data;
mod systems;
mod window;

use data::MouseWorldPosition;
use systems::{
    click_handler, mouse_screen_position_to_world, resize_window, setup_board, setup_camera,
    update_translation_for_location,
};
use window::{WIN_HEIGHT, WIN_WIDTH};

fn main() {
    App::new()
        // Plugins
        .add_plugins_with(DefaultPlugins, |plugins| plugins.disable::<AudioPlugin>())
        // Resources
        .insert_resource(WindowDescriptor {
            title: "Gambit".into(),
            width: WIN_WIDTH,
            height: WIN_HEIGHT,
            resizable: false,
            ..default()
        })
        .init_resource::<MouseWorldPosition>()
        // Startup Systems
        .add_startup_system(resize_window)
        .add_startup_system(setup_camera)
        .add_startup_system(setup_board)
        // Systems
        .add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::new().with_system(mouse_screen_position_to_world),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new().with_system(update_translation_for_location),
        )
        .add_system(click_handler)
        // Run
        .run();
}
