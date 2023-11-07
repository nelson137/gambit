#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::{log::LogPlugin, prelude::*, window::WindowResolution};
use bevy_egui::EguiPlugin;
use clap::Parser;

mod assets;
mod cli;
mod game;
mod utils;

use self::{
    cli::CliArgs,
    game::{
        consts::{COLOR_BG, INIT_WIN_HEIGHT, INIT_WIN_WIDTH, LOG_FILTER, LOG_LEVEL},
        GameLogicPlugin,
    },
    utils::DebugBevyInspectorPlugin,
};

fn main() {
    App::new()
        // Cli
        .insert_resource(CliArgs::parse())
        // Plugins
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Gambit".into(),
                        resolution: WindowResolution::new(INIT_WIN_WIDTH, INIT_WIN_HEIGHT),
                        resizable: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(LogPlugin { level: LOG_LEVEL, filter: LOG_FILTER.into() }),
        )
        .add_plugin(EguiPlugin)
        .add_plugin(GameLogicPlugin)
        .add_plugin(DebugBevyInspectorPlugin)
        // Resources
        .insert_resource(ClearColor(COLOR_BG))
        // Run
        .run();
}
