use bevy::{prelude::*, ui::UiSystem};

pub mod audio;
pub mod board;
pub mod camera;
pub mod captures;
pub mod consts;
pub mod load;
pub mod menu;
pub mod mouse;
pub mod moves;
pub mod ui;
pub mod utils;

use crate::utils::AppPushOrderedStartupStages;

use self::{
    audio::GameAudioHandles,
    board::{
        end_game_icon_size, spawn_board, spawn_end_game_icons, spawn_highlight_tiles, spawn_hints,
        spawn_pieces, spawn_tiles, BoardState, SelectionPlugin,
    },
    camera::setup_camera,
    captures::CaptureState,
    menu::GameMenuPlugin,
    mouse::{spawn_drag_container, MouseLogicPlugin},
    moves::{move_piece, DoMove},
    ui::{spawn_panels, spawn_ui},
};

#[derive(Clone, StageLabel)]
enum SpawnStage {
    Phase1,
    Phase2,
    Phase3,
    Phase4,
}

pub struct GameLogicPlugin;

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        app
            // Plugins
            .add_plugin(MouseLogicPlugin)
            .add_plugin(GameMenuPlugin)
            .add_plugin(SelectionPlugin)
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
                    SystemStage::parallel().with_system(spawn_tiles).with_system(spawn_panels),
                ),
                (
                    SpawnStage::Phase4,
                    SystemStage::parallel()
                        .with_system(spawn_highlight_tiles)
                        .with_system(spawn_hints)
                        .with_system(spawn_pieces)
                        .with_system(spawn_end_game_icons),
                ),
            ])
            // Systems
            .add_system_to_stage(CoreStage::PostUpdate, end_game_icon_size.before(UiSystem::Flex))
            .add_system(move_piece);
    }
}
