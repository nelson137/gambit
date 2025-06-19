use bevy::prelude::*;

use super::{
    board::{GameStatus, ShowCheckmateIcons, ShowDrawIcons},
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
        game_status: Res<GameStatus>,
        mut next_menu_state: ResMut<NextState<MenuState>>,
    ) {
        trace!("Game over");

        match *game_status {
            GameStatus::GameOverCheckmate => commands.trigger(ShowCheckmateIcons),
            GameStatus::GameOverStalemate
            | GameStatus::GameOver50Moves
            | GameStatus::GameOverRepetition => commands.trigger(ShowDrawIcons),
            GameStatus::Ongoing => {
                warn!("Running game over sequence when the game is still ongoing")
            }
        }

        next_menu_state.set(MenuState::DoGameOver);
    }
}
