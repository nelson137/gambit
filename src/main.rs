#![allow(clippy::type_complexity)]

use bevy::{audio::AudioPlugin, prelude::*};

mod assets;
mod data;
mod game;
mod systems;
mod window;

use data::{BoardState, COLOR_BG};
use game::{capture::CaptureState, GameLogicPlugin};
use systems::{setup_board, setup_camera, update_translation_for_square, MousePositionPlugin};
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
        .add_plugin(MousePositionPlugin)
        .add_plugin(GameLogicPlugin)
        // Resources
        .insert_resource(ClearColor(COLOR_BG))
        .init_resource::<BoardState>()
        .init_resource::<CaptureState>()
        // Startup Systems
        .add_startup_system(setup_camera)
        .add_startup_system(setup_board)
        // Systems
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new().with_system(update_translation_for_square),
        )
        // Run
        .run();
}
