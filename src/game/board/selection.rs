use std::{
    fmt,
    hash::{Hash, Hasher},
    mem,
};

use bevy::prelude::*;
use chess::Square;

use crate::{
    game::{board::BoardState, menu::MenuState, mouse::DragContainer},
    utils::StateExts,
};

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app
            // States
            .add_state(SelectionState::Unselected)
            // Events
            .add_event::<SelectionEvent>()
            // Systems
            .add_system_set(
                SystemSet::on_update(MenuState::Game).with_system(handle_selection_events.at_end()),
            )
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

#[derive(Clone, Copy, Debug, Eq)]
pub enum SelectionState {
    Unselected,
    SelectingDragging(Square),
    Selected(Square),
    SelectedDragging(Square),
    DoChangeSelection(Square),
    DoMove(Square, Square),
    DoUnselect(Square),
}

impl fmt::Display for SelectionState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SelectionState::")?;
        match self {
            SelectionState::Unselected => write!(f, "Unselected"),
            SelectionState::SelectingDragging(sq) => write!(f, "SelectingDragging({sq})"),
            SelectionState::Selected(sq) => write!(f, "Selected({sq})"),
            SelectionState::SelectedDragging(sq) => write!(f, "SelectedDragging({sq})"),
            SelectionState::DoChangeSelection(to_sq) => write!(f, "DoChangeSelected({to_sq})"),
            SelectionState::DoMove(from_sq, to_sq) => write!(f, "DoMove({from_sq} -> {to_sq})"),
            SelectionState::DoUnselect(sq) => write!(f, "DoUnselect({sq})"),
        }
    }
}

impl Hash for SelectionState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
    }
}

impl PartialEq for SelectionState {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl SelectionState {
    const SELECTING_DRAGGING: SelectionState = SelectionState::SelectingDragging(Square::A1);
    const SELECTED: SelectionState = SelectionState::Selected(Square::A1);
    const SELECTED_DRAGGING: SelectionState = SelectionState::SelectedDragging(Square::A1);
    const DO_CHANGE_SELECTION: SelectionState = SelectionState::DoChangeSelection(Square::A1);
    const DO_MOVE: SelectionState = SelectionState::DoMove(Square::A1, Square::A1);
    const DO_UNSELECT: SelectionState = SelectionState::DoUnselect(Square::A1);
}

#[derive(Clone, Copy)]
pub enum SelectionEvent {
    MouseDown(Square),
    MouseUp(Square),
}

fn handle_selection_events(
    mut selection_state: ResMut<State<SelectionState>>,
    board_state: Res<BoardState>,
    mut event_reader: EventReader<SelectionEvent>,
) {
    for &event in event_reader.iter() {
        match *selection_state.current() {
            SelectionState::Unselected => match event {
                SelectionEvent::MouseDown(square) => {
                    if board_state.has_piece_at(square) {
                        selection_state.transition(SelectionState::SelectingDragging(square));
                    }
                }
                SelectionEvent::MouseUp(_) => (),
            },
            SelectionState::SelectingDragging(selecting_sq) => match event {
                SelectionEvent::MouseDown(_) => todo!("reset previous drag target"), // TODO
                SelectionEvent::MouseUp(square) => {
                    if board_state.move_is_valid(selecting_sq, square) {
                        selection_state.transition(SelectionState::DoMove(selecting_sq, square));
                    } else {
                        selection_state.transition(SelectionState::Selected(selecting_sq));
                    }
                }
            },
            SelectionState::Selected(selected_sq) => match event {
                SelectionEvent::MouseDown(square) => {
                    if square == selected_sq {
                        selection_state.transition(SelectionState::SelectedDragging(selected_sq));
                    } else if board_state.move_is_valid(selected_sq, square) {
                        selection_state.transition(SelectionState::DoMove(selected_sq, square));
                    } else if board_state.has_piece_at(square) {
                        selection_state.transition(SelectionState::DoChangeSelection(square));
                    } else {
                        selection_state.transition(SelectionState::DoUnselect(selected_sq));
                    }
                }
                SelectionEvent::MouseUp(_) => (),
            },
            SelectionState::SelectedDragging(selected_sq) => match event {
                SelectionEvent::MouseDown(_) => todo!("reset previous drag target"), // TODO
                SelectionEvent::MouseUp(square) => {
                    if square == selected_sq {
                        selection_state.transition(SelectionState::DoUnselect(selected_sq));
                    } else if board_state.move_is_valid(selected_sq, square) {
                        selection_state.transition(SelectionState::DoMove(selected_sq, square));
                    } else {
                        selection_state.transition(SelectionState::Selected(selected_sq));
                    }
                }
            },
            SelectionState::DoChangeSelection(_) => (),
            SelectionState::DoMove(_, _) => (),
            SelectionState::DoUnselect(_) => (),
        }
    }
}

fn on_enter_selection_state(
    mut commands: Commands,
    mut selection_state: ResMut<State<SelectionState>>,
    mut board_state: ResMut<BoardState>,
    q_drag_container: Query<Entity, With<DragContainer>>,
) {
    match *selection_state.current() {
        SelectionState::Unselected => (),
        SelectionState::SelectingDragging(square) => {
            // Re-parent piece to drag container
            let piece = board_state.piece(square);
            commands.entity(piece).set_parent(q_drag_container.single());
            // Select square
            commands.add(board_state.select_square(square));
        }
        SelectionState::Selected(square) => {
            // Re-parent piece back to its tile
            let piece = board_state.piece(square);
            let tile = board_state.tile(square);
            commands.entity(piece).set_parent(tile);
        }
        SelectionState::SelectedDragging(square) => {
            // Re-parent piece to drag container
            let piece = board_state.piece(square);
            commands.entity(piece).set_parent(q_drag_container.single());
        }
        SelectionState::DoChangeSelection(to_sq) => {
            // Unselect square
            commands.add(board_state.unselect_square());
            // Transition to SelectingDragging
            selection_state.transition_overwrite(SelectionState::SelectingDragging(to_sq));
        }
        SelectionState::DoMove(from_sq, to_sq) => {
            // Re-parent piece to destination tile & start move
            commands.add(board_state.move_piece(from_sq, to_sq));
            // Unselect square
            commands.add(board_state.unselect_square());
            // Transition to Unselected
            selection_state.transition_overwrite(SelectionState::Unselected);
        }
        SelectionState::DoUnselect(square) => {
            // Re-parent piece back to its tile
            let piece = board_state.piece(square);
            let tile = board_state.tile(square);
            commands.entity(piece).set_parent(tile);
            // Unselect square
            commands.add(board_state.unselect_square());
            // Transition to Unselected
            selection_state.transition_overwrite(SelectionState::Unselected);
        }
    }
}
