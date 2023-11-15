use bevy::prelude::*;

use crate::utils::AppNoop;

use self::{
    board::{BoardState, PromotionEvent, SelectionPlugin},
    camera::setup_camera,
    menu::GameMenuLogicPlugin,
    mouse::MouseLogicPlugin,
    moves::{move_piece, start_move},
    ui::GameUiPlugin,
};

pub mod audio;
pub mod board;
pub mod camera;
pub mod captures;
pub mod consts;
pub mod core;
pub mod game_over;
pub mod load;
pub mod menu;
pub mod mouse;
pub mod moves;
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
            // Events
            .add_event::<PromotionEvent>()
            // Startup
            .add_systems(Startup, setup_camera)
            // PostStartup
            .add_systems(PostUpdate, (start_move, move_piece).chain())
            .noop();
    }
}
