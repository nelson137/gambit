use bevy::{ecs::schedule::ShouldRun, prelude::*};

use super::menu::MenuState;

mod handler;
mod position;
mod selection;

#[allow(unused_imports)]
pub use self::{handler::*, position::*, selection::*};

pub struct MouseLogicPlugin;

impl Plugin for MouseLogicPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<MouseWorldPosition>()
            .init_resource::<MouseBoardLocation>()
            // States
            .add_state(SelectionState::Unselected)
            // Events
            .add_event::<SelectionEvent>()
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
            .add_system(update_drag_container)
            .add_system(handle_selection_events.at_end())
            .add_system_set(
                SystemSet::on_enter(SelectionState::SELECTING_DRAGGING)
                    .with_system(on_enter_selection_state),
            )
            .add_system_set(
                SystemSet::on_enter(SelectionState::SELECTED).with_system(on_enter_selection_state),
            )
            .add_system_set(
                SystemSet::on_enter(SelectionState::SELECTED_DRAGGING)
                    .with_system(on_enter_selection_state),
            )
            .add_system_set(
                SystemSet::on_enter(SelectionState::DO_CHANGE_SELECTION)
                    .with_system(on_enter_selection_state),
            )
            .add_system_set(
                SystemSet::on_enter(SelectionState::DO_MOVE).with_system(on_enter_selection_state),
            )
            .add_system_set(
                SystemSet::on_enter(SelectionState::DO_UNSELECT)
                    .with_system(on_enter_selection_state),
            );
    }
}

fn is_in_game(menu_state: Res<State<MenuState>>) -> ShouldRun {
    match menu_state.current() {
        MenuState::Game => ShouldRun::Yes,
        _ => ShouldRun::No,
    }
}
