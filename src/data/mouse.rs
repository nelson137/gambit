use bevy::prelude::*;
use chess::Square;

use super::BoardPiece;

#[derive(Default, Deref, DerefMut, Resource)]
pub struct MouseWorldPosition(pub Vec2);

#[derive(Default, Deref, DerefMut, Resource)]
pub struct MouseSquare(pub Option<Square>);

#[derive(Component)]
pub struct DragContainer;

pub struct MakeMove {
    pub piece: BoardPiece,
    pub from_sq: Square,
    pub to_sq: Square,
}

#[derive(Component, Deref, DerefMut)]
#[component(storage = "SparseSet")]
pub struct DoUpdatePieceSquare(pub Square);
