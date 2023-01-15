use bevy::{ecs::system::Command, prelude::*};
use chess::Square;

use crate::game::board::{BoardLocation, BoardState};

#[derive(Clone, Copy)]
pub struct DoMove {
    pub piece: Entity,
    pub from_sq: Square,
    pub to_sq: Square,
}

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

pub fn move_piece(
    mut commands: Commands,
    mut board_state: ResMut<BoardState>,
    mut do_move_reader: EventReader<DoMove>,
) {
    for do_move in do_move_reader.iter() {
        commands.add(board_state.move_piece(*do_move));
    }
}
