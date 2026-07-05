#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use game::consts::*;

mod cli;
mod debug_inspector;
mod game;
mod utils;

fn main() {
    App::new()
        // App
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Gambit".into(),
                        resolution: bevy::window::WindowResolution::new(
                            INIT_WIN_WIDTH,
                            INIT_WIN_HEIGHT,
                        ),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    level: LOG_LEVEL,
                    filter: LOG_FILTER.into(),
                    ..default()
                }),
        )
        // Game
        .add_plugins(cli::CliPlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(utils::SortableChildrenPlugin)
        .add_plugins(game::GameLogicPlugin)
        .add_plugins(debug_inspector::DebugInspectorPlugin)
        // Resources
        .insert_resource(ClearColor(COLOR_BG))
        // Run
        .run();
}
