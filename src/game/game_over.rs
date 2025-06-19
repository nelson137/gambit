use bevy::prelude::*;

use super::{
    board::{BoardState, GameStatus, ShowCheckmateIcons, ShowDrawIcons},
    menu::MenuState,
};

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(GameOver::handle);
    }
}

#[derive(Event)]
pub struct GameOver;

impl GameOver {
    fn handle(
        _trigger: Trigger<Self>,
        mut commands: Commands,
        board_state: Res<BoardState>,
        mut next_menu_state: ResMut<NextState<MenuState>>,
    ) {
        trace!("Game over");

        match board_state.status() {
            GameStatus::GameOverCheckmate => commands.queue(ShowCheckmateIcons),
            GameStatus::GameOverStalemate
            | GameStatus::GameOver50Moves
            | GameStatus::GameOverRepetition => commands.queue(ShowDrawIcons),
            GameStatus::Ongoing => {
                warn!("Running game over sequence when the game is still ongoing")
            }
        }

        next_menu_state.set(MenuState::DoGameOver);
    }
}
