use bevy::{ecs::system::Command, prelude::*};
use chess::BoardStatus;

use crate::utils::StateExts;

use super::{
    board::{BoardState, ShowCheckmateIcons, ShowStalemateIcons},
    menu::MenuState,
};

pub struct GameOver;

impl Command for GameOver {
    fn write(self, world: &mut World) {
        trace!("Game over");

        let board = world.resource::<BoardState>().board();
        match board.status() {
            BoardStatus::Checkmate => ShowCheckmateIcons.write(world),
            BoardStatus::Stalemate => ShowStalemateIcons.write(world),
            BoardStatus::Ongoing => {
                warn!("Running game over sequence when the game is still ongoing")
            }
        }

        world.resource_mut::<State<MenuState>>().transition(MenuState::DoGameOver);
    }
}
