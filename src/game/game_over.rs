use bevy::{ecs::system::Command, prelude::*};
use chess::BoardStatus;

use super::{
    board::{BoardState, ShowCheckmateIcons, ShowStalemateIcons},
    menu::MenuState,
};

pub struct GameOver;

impl Command for GameOver {
    fn apply(self, world: &mut World) {
        trace!("Game over");

        let board = world.resource::<BoardState>().board();
        match board.status() {
            BoardStatus::Checkmate => ShowCheckmateIcons.apply(world),
            BoardStatus::Stalemate => ShowStalemateIcons.apply(world),
            BoardStatus::Ongoing => {
                warn!("Running game over sequence when the game is still ongoing")
            }
        }

        world.resource_mut::<NextState<MenuState>>().set(MenuState::DoGameOver);
    }
}
