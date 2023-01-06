use bevy::prelude::*;
use chess::Square;

#[derive(Default, Deref, DerefMut, Resource)]
pub struct MouseWorldPosition(pub Vec2);

#[derive(Default, Deref, DerefMut, Resource)]
pub struct MouseSquare(pub Option<Square>);

#[derive(Component)]
pub struct DragContainer;
