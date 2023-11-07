use bevy::prelude::*;

use super::menu::MenuState;

mod handler;
mod position;

#[allow(unused_imports)]
pub use self::{handler::*, position::*};

pub struct MouseLogicPlugin;

impl Plugin for MouseLogicPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<MouseWorldPosition>()
            .init_resource::<MouseBoardSquare>()
            // Systems
            .add_systems(
                (
                    mouse_screen_position_to_world,
                    mouse_world_position_to_square.after(mouse_screen_position_to_world),
                )
                    .distributive_run_if(is_in_game)
                    .in_base_set(CoreSet::PreUpdate),
            )
            .add_systems((
                mouse_handler.run_if(is_in_game),
                update_drag_container.run_if(is_in_game),
            ));
    }
}

fn is_in_game(menu_state: Res<State<MenuState>>) -> bool {
    matches!(menu_state.0, MenuState::Game)
}
