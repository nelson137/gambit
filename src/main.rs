#![allow(clippy::type_complexity)]

use bevy::prelude::*;

mod assets;
mod data;
mod game;
mod utils;
mod window;

use self::{
    data::COLOR_BG,
    game::GameLogicPlugin,
    window::{WIN_HEIGHT, WIN_WIDTH},
};

fn main() {
    App::new()
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
        .add_plugin(GameLogicPlugin)
        // Resources
        .insert_resource(ClearColor(COLOR_BG))
        // Run
        .run();
}
