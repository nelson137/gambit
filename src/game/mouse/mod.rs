use bevy::prelude::*;

mod handler;
mod position;
mod selection;

use self::{
    handler::{mouse_handler, update_drag_container},
    position::{
        mouse_screen_position_to_world, mouse_world_position_to_square, MouseBoardLocation,
        MouseWorldPosition,
    },
    selection::{
        handle_selection_events, on_enter_selection_state, SelectionEvent, SelectionState,
    },
};

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
                    mouse_world_position_to_square.after(mouse_screen_position_to_world),
                ),
            )
            .add_system(mouse_handler)
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
