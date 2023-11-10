use bevy::prelude::*;

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

use self::{
    board::{BoardState, PromotionEvent, SelectionPlugin},
    camera::setup_camera,
    menu::GameMenuLogicPlugin,
    mouse::MouseLogicPlugin,
    ui::GameUiPlugin,
};

pub struct GameLogicPlugin;

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        app
            // Plugins
            .add_plugins(GameUiPlugin)
            .add_plugins(MouseLogicPlugin)
            .add_plugins(GameMenuLogicPlugin)
            .add_plugins(SelectionPlugin)
            // Events
            .add_event::<PromotionEvent>()
            // Startup
            .add_systems(Startup, setup_camera);
    }
}
