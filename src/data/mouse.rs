use bevy::prelude::*;
use chess::Square;

#[derive(Default, Deref, DerefMut, Resource)]
pub struct MouseWorldPosition(pub Vec2);

#[derive(Default, Deref, DerefMut, Resource)]
pub struct MouseSquare(pub Option<Square>);

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Selected;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Dragging {
    pub mouse_down_square: Square,
}

impl Dragging {
    pub fn new(mouse_down_square: Square) -> Self {
        Self { mouse_down_square }
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Dropped;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct DoUnselect;

#[derive(Component, Deref, DerefMut)]
#[component(storage = "SparseSet")]
pub struct DoMove(pub Square);

#[derive(Component, Deref, DerefMut)]
#[component(storage = "SparseSet")]
pub struct DoUpdatePieceSquare(pub Square);
