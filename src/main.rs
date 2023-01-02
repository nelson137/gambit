#![allow(clippy::type_complexity)]

use bevy::{audio::AudioPlugin, prelude::*};

mod assets;
mod data;
mod game;
mod systems;
mod utils;
mod window;

use self::{
    data::{BoardState, COLOR_BG},
    game::{captures::CaptureState, GameLogicPlugin},
    systems::{
        setup_camera, spawn_board, spawn_drag_container, spawn_tiles_hints_pieces, spawn_ui,
        MousePositionPlugin, SpawnStage,
    },
    utils::AppPushOrderedStartupStages,
    window::{WIN_HEIGHT, WIN_WIDTH},
};

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
        .add_startup_system(spawn_drag_container)
        .push_ordered_startup_stages([
            (SpawnStage::Ui, SystemStage::single(spawn_ui)),
            (SpawnStage::Board, SystemStage::single(spawn_board)),
            (SpawnStage::TilesHintsPieces, SystemStage::single(spawn_tiles_hints_pieces)),
        ])
        // Run
        .run();
}
