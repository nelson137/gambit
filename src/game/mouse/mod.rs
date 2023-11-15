use bevy::prelude::*;

use super::menu::{MenuState, PromoterSystem};

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
                    .run_if(in_state(MenuState::Game)),
            )
            .add_systems(
                Update,
                (mouse_handler, update_drag_container).run_if(in_state(MenuState::Game)),
            );
    }
}
