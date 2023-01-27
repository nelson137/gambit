use bevy::{ecs::system::Command, prelude::*};
use chess::Square;

use crate::game::board::{BoardLocation, BoardState};

pub struct MoveUiPiece {
    pub piece: Entity,
    pub to_sq: Square,
}

impl Command for MoveUiPiece {
    fn write(self, world: &mut World) {
        let mut entity = world.entity_mut(self.piece);
        if let Some(mut square) = entity.get_mut::<BoardLocation>() {
            square.move_to(self.to_sq);
        }

        let board_state = world.resource::<BoardState>();
        let to_tile = board_state.tile(self.to_sq);
        world.entity_mut(to_tile).push_children(&[self.piece]);
    }
}
