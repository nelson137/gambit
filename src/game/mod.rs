use std::str::FromStr;

use bevy::prelude::*;

use crate::{cli::CliArgs, utils::NoopExts};

use self::{
    board::{BoardState, MovePlugin, PieceAnimationPlugin, PromotionEvent, SelectionPlugin},
    camera::setup_camera,
    menu::GameMenuLogicPlugin,
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
pub mod load;
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
            .add_event::<PromotionEvent>()
            // Startup
            .add_systems(Startup, setup_camera)
            .add_systems(PostStartup, load_game)
            // PostStartup
            .noop();
    }
}

fn load_game(world: &mut World) {
    let cli_args = world.resource::<CliArgs>();
    let board = match cli_args.fen.as_deref().map(chess::Board::from_str) {
        None => return,
        Some(Err(err)) => {
            error!("{err}");
            return;
        }
        Some(Ok(board)) => board,
    };

    world.resource_mut::<BoardState>().set_board(&board);
    world.trigger(LoadGame(board));
}

#[derive(Event)]
pub struct LoadGame(pub chess::Board);
