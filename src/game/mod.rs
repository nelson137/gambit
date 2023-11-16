use bevy::prelude::*;

use crate::utils::AppNoop;

use self::{
    board::{BoardState, MovePlugin, PromotionEvent, SelectionPlugin},
    camera::setup_camera,
    menu::GameMenuLogicPlugin,
    mouse::MouseLogicPlugin,
    ui::GameUiPlugin,
};

pub mod audio;
pub mod board;
pub mod camera;
pub mod consts;
pub mod core;
pub mod game_over;
pub mod load;
pub mod menu;
pub mod mouse;
pub mod panels;
pub mod ui;
pub mod utils;

pub struct GameLogicPlugin;

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            // Plugins
            .add_plugins(GameUiPlugin)
            .add_plugins(MouseLogicPlugin)
            .add_plugins(GameMenuLogicPlugin)
            .add_plugins(SelectionPlugin)
            .add_plugins(MovePlugin)
            // Events
            .add_event::<PromotionEvent>()
            // Startup
            .add_systems(Startup, setup_camera)
            // PostStartup
            .noop();
    }
}
