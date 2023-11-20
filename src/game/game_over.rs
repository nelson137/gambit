use bevy::{ecs::system::Command, prelude::*};

use super::{
    board::{BoardState, GameStatus, ShowCheckmateIcons, ShowStalemateIcons},
    menu::MenuState,
};

pub struct GameOver;

impl Command for GameOver {
    fn apply(self, world: &mut World) {
        trace!("Game over");

        match world.resource::<BoardState>().status() {
            GameStatus::GameOverCheckmate => ShowCheckmateIcons.apply(world),
            GameStatus::GameOverStalemate
            | GameStatus::GameOver50Moves
            | GameStatus::GameOverRepetition => ShowStalemateIcons.apply(world),
            GameStatus::Ongoing => {
                warn!("Running game over sequence when the game is still ongoing")
            }
        }

        world.resource_mut::<NextState<MenuState>>().set(MenuState::DoGameOver);
    }
}
