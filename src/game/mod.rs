use bevy::prelude::*;

pub mod audio;
pub mod board;
pub mod camera;
pub mod captures;
pub mod consts;
pub mod mouse;
pub mod moves;
pub mod ui;
pub mod utils;

use crate::utils::AppPushOrderedStartupStages;

use self::{
    audio::GameAudioHandles,
    board::{spawn_board, spawn_board_elements, BoardState},
    camera::setup_camera,
    captures::CaptureState,
    mouse::{spawn_drag_container, MouseLogicPlugin},
    moves::{move_piece, DoMove},
    ui::{spawn_panels, spawn_ui},
};

#[derive(Clone, StageLabel)]
enum SpawnStage {
    Phase1,
    Phase2,
    Phase3,
}

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
            // Startup
            .add_startup_system(setup_camera)
            .add_startup_system(spawn_drag_container)
            .push_ordered_startup_stages([
                (SpawnStage::Phase1, SystemStage::single(spawn_ui)),
                (SpawnStage::Phase2, SystemStage::single(spawn_board)),
                (
                    SpawnStage::Phase3,
                    SystemStage::parallel()
                        .with_system(spawn_board_elements)
                        .with_system(spawn_panels),
                ),
            ])
            // Systems
            .add_system(move_piece);
    }
}
