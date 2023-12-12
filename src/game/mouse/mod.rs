use bevy::prelude::*;

use super::{board::PromoterSystem, menu::MenuState};

pub use self::{handler::*, position::*};

mod handler;
mod position;

pub struct MouseUiPlugin;

impl Plugin for MouseUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_drag_container);
    }
}

pub struct MouseLogicPlugin;

impl Plugin for MouseLogicPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<MouseWorldPosition>()
            .init_resource::<MouseBoardSquare>()
            // Systems
            .add_systems(
                PreUpdate,
                (mouse_screen_position_to_world, mouse_world_position_to_square)
                    .chain()
                    .after(PromoterSystem)
                    .run_if(in_state(MenuState::Game))
                    .run_if(mouse_is_in_world),
            )
            .add_systems(
                PreUpdate,
                (mouse_handler, update_drag_container)
                    .run_if(in_state(MenuState::Game))
                    .run_if(mouse_is_in_world),
            );
    }
}

#[cfg(feature = "debug-inspector")]
fn mouse_is_in_world(value: Res<crate::debug_inspector::DebugInspectorIsUsingMouse>) -> bool {
    !**value
}

#[cfg(not(feature = "debug-inspector"))]
fn mouse_is_in_world() -> bool {
    true
}
