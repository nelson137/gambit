#![allow(clippy::type_complexity)]

use bevy::{audio::AudioPlugin, prelude::*};

mod assets;
mod data;
mod game;
mod systems;
mod window;

use data::{BoardState, MouseSquare, MouseWorldPosition, COLOR_BG};
use game::GameLogicPlugin;
use systems::{
    mouse_screen_position_to_world, mouse_world_position_to_square, setup_board, setup_camera,
    update_translation_for_square,
};
use window::{WIN_HEIGHT, WIN_WIDTH};

fn main() {
    App::new()
        // Plugins
        .add_plugins(
            DefaultPlugins
                .build()
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: "Gambit".into(),
                        width: WIN_WIDTH,
                        height: WIN_HEIGHT,
                        resizable: true,
                        ..default()
                    },
                    ..default()
                })
                .disable::<AudioPlugin>(),
        )
        .add_plugin(GameLogicPlugin)
        // Resources
        .insert_resource(ClearColor(COLOR_BG))
        .init_resource::<BoardState>()
        .init_resource::<MouseWorldPosition>()
        .init_resource::<MouseSquare>()
        // Startup Systems
        .add_startup_system(setup_camera)
        .add_startup_system(setup_board)
        // Systems
        .add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::new()
                .with_system(mouse_screen_position_to_world)
                .with_system(mouse_world_position_to_square.after(mouse_screen_position_to_world)),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new().with_system(update_translation_for_square),
        )
        // Run
        .run();
}
