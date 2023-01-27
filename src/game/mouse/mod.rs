use bevy::{ecs::schedule::ShouldRun, prelude::*};

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
            .init_resource::<MouseBoardLocation>()
            // Systems
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new().with_system(mouse_screen_position_to_world).with_system(
                    mouse_world_position_to_square
                        .after(mouse_screen_position_to_world)
                        .with_run_criteria(is_in_game),
                ),
            )
            .add_system(mouse_handler.with_run_criteria(is_in_game))
            .add_system(update_drag_container);
    }
}

fn is_in_game(menu_state: Res<State<MenuState>>) -> ShouldRun {
    match menu_state.current() {
        MenuState::Game => ShouldRun::Yes,
        _ => ShouldRun::No,
    }
}
