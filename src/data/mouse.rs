use bevy::prelude::*;

use super::Location;

#[derive(Default)]
pub struct MouseWorldPosition(pub Vec2);

#[derive(Default)]
pub struct MouseLocation(pub Option<Location>);

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
    pub mouse_down_location: Location,
}

impl Dragging {
    pub fn new(mouse_down_location: Location) -> Self {
        Self { mouse_down_location }
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
