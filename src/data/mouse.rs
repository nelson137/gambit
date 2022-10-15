use bevy::prelude::*;
use chess::Square;

#[derive(Default, Deref, DerefMut)]
pub struct MouseWorldPosition(pub Vec2);

#[derive(Default, Deref, DerefMut)]
pub struct MouseSquare(pub Option<Square>);

#[derive(Component)]
pub struct Hoverable;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Hover;

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

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct DoMove;
