use bevy::{ecs::system::Command, prelude::*};
use chess::Square;

use super::{BoardPiece, BoardState, UiSquare};

#[derive(Default, Deref, DerefMut, Resource)]
pub struct MouseWorldPosition(pub Vec2);

#[derive(Default, Deref, DerefMut, Resource)]
pub struct MouseSquare(pub Option<Square>);

#[derive(Component)]
pub struct DragContainer;

pub struct DoMove {
    pub piece: BoardPiece,
    pub from_sq: Square,
    pub to_sq: Square,
}

pub struct MoveUiPiece {
    pub piece: BoardPiece,
    pub to_sq: Square,
}

impl Command for MoveUiPiece {
    fn write(self, world: &mut World) {
        let mut entity = world.entity_mut(self.piece.entity);
        if let Some(mut square) = entity.get_mut::<UiSquare>() {
            square.move_to(self.to_sq);
        }

        let board_state = world.resource::<BoardState>();
        let to_tile = board_state.tile(self.to_sq);
        world.entity_mut(to_tile).push_children(&[self.piece.entity]);
    }
}
