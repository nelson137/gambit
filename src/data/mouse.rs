use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use chess::Square;

#[derive(Default, Deref, DerefMut, Resource)]
pub struct MouseWorldPosition(pub Vec2);

#[derive(Default, Deref, DerefMut, Resource)]
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
pub struct DoMove {
    pub square: Square,
    pub update_state: bool,
}

impl Deref for DoMove {
    type Target = Square;
    fn deref(&self) -> &Self::Target {
        &self.square
    }
}

impl DerefMut for DoMove {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.square
    }
}

impl DoMove {
    pub fn new(square: Square) -> Self {
        Self { square, update_state: true }
    }

    pub fn with_update_state(square: Square, update_state: bool) -> Self {
        Self { square, update_state }
    }
}
