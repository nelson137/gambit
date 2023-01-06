use bevy::prelude::*;

use crate::data::BoardState;

pub mod audio;
pub mod camera;
pub mod captures;
pub mod mouse;
pub mod moves;
pub mod selection;

use self::{
    audio::GameAudioHandles,
    captures::CaptureState,
    mouse::MouseLogicPlugin,
    moves::{move_piece, DoMove},
    selection::{
        handle_selection_events, on_enter_selection_state, SelectionEvent, SelectionState,
    },
};

pub struct GameLogicPlugin;

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        app
            // Plugins
            .add_plugin(MouseLogicPlugin)
            // Resources
            .init_resource::<GameAudioHandles>()
            .init_resource::<BoardState>()
            .init_resource::<CaptureState>()
            // States
            .add_state(SelectionState::Unselected)
            // Events
            .add_event::<SelectionEvent>()
            .add_event::<DoMove>()
            // Systems
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
            )
            .add_system(move_piece);
    }
}
