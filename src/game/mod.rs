use bevy::prelude::*;

pub mod audio;
pub mod board;
pub mod camera;
pub mod captures;
pub mod consts;
pub mod game_over;
pub mod load;
pub mod menu;
pub mod mouse;
pub mod moves;
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
            .add_plugin(GameUiPlugin)
            .add_plugin(MouseLogicPlugin)
            .add_plugin(GameMenuLogicPlugin)
            .add_plugin(SelectionPlugin)
            // Events
            .add_event::<PromotionEvent>()
            // Startup
            .add_startup_system(setup_camera);
    }
}
