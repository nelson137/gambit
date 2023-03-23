use bevy::{ecs::system::Command, prelude::*};

use super::board::{BoardState, PieceColor, Square};

pub struct MoveUiPiece {
    pub piece: Entity,
    pub color: PieceColor,
    pub from_sq: Square,
    pub to_sq: Square,
}

impl Command for MoveUiPiece {
    fn write(self, world: &mut World) {
        let mut entity = world.entity_mut(self.piece);
        if let Some(mut square) = entity.get_mut::<Square>() {
            square.move_to(self.to_sq);
        }

        let mut board_state = world.resource_mut::<BoardState>();
        let to_tile = board_state.tile(self.to_sq);

        let maybe_cmd = board_state.update_piece(self.color, self.from_sq, self.to_sq);
        if let Some(cmd) = maybe_cmd {
            cmd.write(world);
        }

        world.entity_mut(to_tile).push_children(&[self.piece]);
    }
}
