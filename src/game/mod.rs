use bevy::{prelude::*, ui::UiSystem};
use bevy_startup_tree::{startup_tree, AddStartupTree};

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
    audio::GameAudioHandles,
    board::{
        end_game_icon_size, spawn_board, spawn_end_game_icons, spawn_highlight_tiles, spawn_hints,
        spawn_pieces, spawn_tiles, BoardState, SelectionPlugin,
    },
    camera::setup_camera,
    captures::CaptureState,
    load::load_capture_state,
    menu::GameMenuPlugin,
    mouse::{spawn_drag_container, MouseLogicPlugin},
    ui::{spawn_panels, spawn_ui},
};

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
            // Startup
            .add_startup_system(setup_camera)
            .add_startup_system(spawn_drag_container)
            .add_startup_tree(startup_tree! {
                spawn_ui => {
                    spawn_board => {
                        spawn_tiles => {
                            spawn_highlight_tiles,
                            spawn_hints,
                            spawn_pieces,
                            spawn_end_game_icons,
                        },
                        spawn_panels => load_capture_state,
                    }
                }
            })
            // Systems
            .add_system_to_stage(CoreStage::PostUpdate, end_game_icon_size.before(UiSystem::Flex));
    }
}
