#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod assets;
mod cli;
mod game;
mod utils;

use self::{
    cli::CliPlugin,
    game::{
        consts::{COLOR_BG, LOG_FILTER, LOG_LEVEL},
        core::{GameHeadPlugin, GameHeadlessPlugin},
        GameLogicPlugin,
    },
    utils::DebugBevyInspectorPlugin,
};

fn main() {
    App::new()
        .add_plugins(bevy::log::LogPlugin { level: LOG_LEVEL, filter: LOG_FILTER.into() })
        // App
        .add_plugins((GameHeadlessPlugin, GameHeadPlugin))
        // Game
        .add_plugins(CliPlugin)
        .add_plugins(EguiPlugin)
        .add_plugins(GameLogicPlugin)
        .add_plugins(DebugBevyInspectorPlugin)
        // Resources
        .insert_resource(ClearColor(COLOR_BG))
        // Run
        .run();
}
