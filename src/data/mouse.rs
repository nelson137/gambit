use std::time::Duration;

use bevy::prelude::*;

use super::Location;

#[derive(Default)]
pub struct MouseWorldPosition(pub Vec2);

#[derive(Default)]
pub struct MouseLocation(pub Option<Location>);

#[derive(Default)]
pub struct LastMouseDownLocation(pub Option<Location>);

#[derive(Component)]
pub struct Hoverable;

#[derive(Component)]
pub struct Hover;

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct Dragging;

#[derive(Component)]
pub struct Dropped;

pub const CLICK_DURATION: Duration = Duration::from_millis(200);

#[derive(Default)]
pub struct ClickState {
    pub last_mouse_down: Duration,
    pub last_mouse_up: Duration,
}

impl ClickState {
    pub fn last_click_duration(&self) -> Duration {
        self.last_mouse_up - self.last_mouse_down
    }

    pub fn input_was_click(&self) -> bool {
        self.last_click_duration() <= CLICK_DURATION
    }
}
