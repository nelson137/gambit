#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

use self::{
    cli::CliPlugin,
    debug_inspector::DebugInspectorPlugin,
    game::{
        GameLogicPlugin,
        consts::{COLOR_BG, LOG_FILTER, LOG_LEVEL},
        core::{GameHeadPlugin, GameHeadlessPlugin},
    },
};

mod cli;
mod debug_inspector;
mod game;
mod utils;

fn main() {
    App::new()
        .add_plugins(bevy::log::LogPlugin {
            level: LOG_LEVEL,
            filter: LOG_FILTER.into(),
            ..default()
        })
        // App
        .add_plugins((GameHeadlessPlugin, GameHeadPlugin))
        // Game
        .add_plugins(CliPlugin)
        .add_plugins(EguiPlugin { enable_multipass_for_primary_context: false })
        .add_plugins(GameLogicPlugin)
        .add_plugins(DebugInspectorPlugin)
        // Resources
        .insert_resource(ClearColor(COLOR_BG))
        // Run
        .run();
}
