use bevy::prelude::*;

pub mod audio;
pub mod board;
pub mod camera;
pub mod captures;
pub mod mouse;
pub mod moves;
pub mod utils;

use self::{
    audio::GameAudioHandles,
    board::BoardState,
    captures::CaptureState,
    mouse::MouseLogicPlugin,
    moves::{move_piece, DoMove},
};

pub struct GameLogicPlugin;

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        app
            // Plugins
            .add_plugin(MouseLogicPlugin)
            // Resources
            .init_resource::<GameAudioHandles>()
            .init_resource::<BoardState>()
            .init_resource::<CaptureState>()
            // Events
            .add_event::<DoMove>()
            // Systems
            .add_system(move_piece);
    }
}
