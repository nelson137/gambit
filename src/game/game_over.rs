use bevy::prelude::*;

use super::{
    LoadGame,
    board::{GameStatus, ShowCheckmateIcons, ShowDrawIcons},
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
    fn handle(_game_over: On<Self>, mut commands: Commands, game_status: Res<GameStatus>) {
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

        commands.delayed().secs(2.).queue(Self::execute);
    }

    fn execute(world: &mut World) {
        trace!("Reset game");
        world.trigger(LoadGame::in_menu());
    }
}
