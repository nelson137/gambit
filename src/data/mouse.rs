use bevy::prelude::*;

use super::Location;

#[derive(Default)]
pub struct MouseWorldPosition(pub Vec2);

#[derive(Default)]
pub struct MouseLocation(pub Option<Location>);

#[derive(Component)]
pub struct Hoverable;

#[derive(Component)]
pub struct Hover;

#[derive(Component)]
pub struct Selecting {
    pub mouse_down_location: Location,
}

impl Selecting {
    pub fn new(mouse_down_location: Location) -> Self {
        Self { mouse_down_location }
    }
}

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct Dragging;

#[derive(Component)]
pub struct Dropped;

#[derive(Component)]
pub struct Unselected;
