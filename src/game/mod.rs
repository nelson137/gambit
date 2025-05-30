use bevy::prelude::*;

use crate::{cli::CliArgs, utils::NoopExts};

use self::{
    board::{MovePlugin, PieceAnimationPlugin, SelectionPlugin},
    camera::setup_camera,
    menu::GameMenuLogicPlugin,
    menu::MenuState,
    mouse::MouseLogicPlugin,
    stockfish::StockfishPlugin,
    ui::GameUiPlugin,
};

pub mod audio;
pub mod board;
pub mod camera;
pub mod consts;
pub mod core;
pub mod eval_bar;
pub mod game_over;
pub mod menu;
pub mod mouse;
pub mod panels;
pub mod stockfish;
pub mod ui;

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
            .add_plugins(PieceAnimationPlugin)
            .add_plugins(StockfishPlugin)
            // Events
            .add_event::<LoadGame>()
            // Startup
            .add_systems(Startup, setup_camera)
            .add_systems(PostStartup, load_game_on_startup)
            // PostStartup
            .noop();
    }
}

fn load_game_on_startup(world: &mut World) {
    let cli_args = world.resource::<CliArgs>();
    let (data, menu_state) = match cli_args.fen.as_deref().map(board::parse_fen) {
        None => ((default(), 0, 1), MenuState::Menu),
        Some(Err(err)) => {
            error!("{err}");
            return;
        }
        Some(Ok(data)) => (data, MenuState::Game),
    };

    world.trigger(LoadGame::new(data.0, data.1, data.2, menu_state));
}

#[derive(Event)]
pub struct LoadGame {
    pub board: chess::Board,
    pub menu_state: MenuState,
    pub half_move_clock: u8,
    pub full_move_count: u16,
}

impl LoadGame {
    pub fn new(
        board: chess::Board,
        half_move_clock: u8,
        full_move_count: u16,
        menu_state: MenuState,
    ) -> Self {
        Self { board, half_move_clock, full_move_count, menu_state }
    }

    pub fn in_game(board: chess::Board, half_move_clock: u8, full_move_count: u16) -> Self {
        Self::new(board, half_move_clock, full_move_count, MenuState::Game)
    }

    #[cfg(test)]
    pub fn in_game_default() -> Self {
        Self::in_game(chess::Board::default(), 0, 0)
    }

    pub fn in_menu() -> Self {
        Self::new(default(), 0, 1, MenuState::Menu)
    }
}
