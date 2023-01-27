#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use clap::Parser;

mod assets;
mod cli;
mod game;
mod utils;
mod window;

use self::{
    cli::CliArgs,
    game::{consts::COLOR_BG, GameLogicPlugin},
    utils::DebugBevyInspectorPlugin,
    window::{WIN_HEIGHT, WIN_WIDTH},
};

fn main() {
    App::new()
        // Cli
        .insert_resource(CliArgs::parse())
        // Plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Gambit".into(),
                width: WIN_WIDTH,
                height: WIN_HEIGHT,
                resizable: true,
                ..default()
            },
            ..default()
        }))
        .add_plugin(EguiPlugin)
        .add_plugin(GameLogicPlugin)
        .add_plugin(DebugBevyInspectorPlugin)
        // Resources
        .insert_resource(ClearColor(COLOR_BG))
        // Run
        .run();
}
