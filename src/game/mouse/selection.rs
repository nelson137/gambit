use std::{
    hash::{Hash, Hasher},
    mem,
};

use bevy::prelude::*;
use chess::Square;

use crate::game::{
    board::{BoardState, HideHighlight, ShowHighlight},
    mouse::handler::DragContainer,
    moves::DoMove,
};

#[derive(Clone, Copy, Debug, Eq)]
pub enum SelectionState {
    Unselected,
    SelectingDragging(Square),
    Selected(Square),
    SelectedDragging(Square),
    DoChangeSelection(Square, Square),
    DoMove(Square, Square),
    DoUnselect(Square),
}

#[cfg(debug_assertions)]
impl std::fmt::Display for SelectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SelectionState::Unselected => f.write_str("Unselected"),
            SelectionState::SelectingDragging(sq) => {
                f.write_fmt(format_args!("SelectingDragging({sq})"))
            }
            SelectionState::Selected(sq) => f.write_fmt(format_args!("Selected({sq})")),
            SelectionState::SelectedDragging(sq) => {
                f.write_fmt(format_args!("SelectedDragging({sq})"))
            }
            SelectionState::DoChangeSelection(from_sq, to_sq) => {
                f.write_fmt(format_args!("DoChangeSelected({from_sq} -> {to_sq})"))
            }
            SelectionState::DoMove(from_sq, to_sq) => {
                f.write_fmt(format_args!("DoMove({from_sq} -> {to_sq})"))
            }
            SelectionState::DoUnselect(sq) => f.write_fmt(format_args!("DoUnselect({sq})")),
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
    pub const SELECTING_DRAGGING: SelectionState = SelectionState::SelectingDragging(Square::A1);
    pub const SELECTED: SelectionState = SelectionState::Selected(Square::A1);
    pub const SELECTED_DRAGGING: SelectionState = SelectionState::SelectedDragging(Square::A1);
    pub const DO_CHANGE_SELECTION: SelectionState =
        SelectionState::DoChangeSelection(Square::A1, Square::A1);
    pub const DO_MOVE: SelectionState = SelectionState::DoMove(Square::A1, Square::A1);
    pub const DO_UNSELECT: SelectionState = SelectionState::DoUnselect(Square::A1);
}

#[derive(Clone, Copy)]
pub enum SelectionEvent {
    MouseDown(Square),
    MouseUp(Square),
}

pub(super) fn handle_selection_events(
    mut selection_state: ResMut<State<SelectionState>>,
    board_state: Res<BoardState>,
    mut event_reader: EventReader<SelectionEvent>,
) {
    for &event in event_reader.iter() {
        match *selection_state.current() {
            SelectionState::Unselected => match event {
                SelectionEvent::MouseDown(square) => {
                    if board_state.has_piece_at(square) {
                        selection_state.set(SelectionState::SelectingDragging(square)).unwrap();
                    }
                }
                SelectionEvent::MouseUp(_) => (),
            },
            SelectionState::SelectingDragging(selecting_sq) => match event {
                SelectionEvent::MouseDown(_) => todo!("reset previous drag target"), // TODO
                SelectionEvent::MouseUp(square) => {
                    if board_state.move_is_valid(selecting_sq, square) {
                        selection_state.set(SelectionState::DoMove(selecting_sq, square)).unwrap();
                    } else {
                        selection_state.set(SelectionState::Selected(selecting_sq)).unwrap();
                    }
                }
            },
            SelectionState::Selected(selected_sq) => match event {
                SelectionEvent::MouseDown(square) => {
                    if square == selected_sq {
                        selection_state.set(SelectionState::SelectedDragging(selected_sq)).unwrap();
                    } else if board_state.move_is_valid(selected_sq, square) {
                        selection_state.set(SelectionState::DoMove(selected_sq, square)).unwrap();
                    } else if board_state.has_piece_at(square) {
                        selection_state
                            .set(SelectionState::DoChangeSelection(selected_sq, square))
                            .unwrap();
                    } else {
                        selection_state.set(SelectionState::DoUnselect(selected_sq)).unwrap();
                    }
                }
                SelectionEvent::MouseUp(_) => (),
            },
            SelectionState::SelectedDragging(selected_sq) => match event {
                SelectionEvent::MouseDown(_) => todo!("reset previous drag target"), // TODO
                SelectionEvent::MouseUp(square) => {
                    if square == selected_sq {
                        selection_state.set(SelectionState::DoUnselect(selected_sq)).unwrap();
                    } else if board_state.move_is_valid(selected_sq, square) {
                        selection_state.set(SelectionState::DoMove(selected_sq, square)).unwrap();
                    } else {
                        selection_state.set(SelectionState::Selected(selected_sq)).unwrap();
                    }
                }
            },
            SelectionState::DoChangeSelection(_, _) => (),
            SelectionState::DoMove(_, _) => (),
            SelectionState::DoUnselect(_) => (),
        }
    }
}

pub(super) fn on_enter_selection_state(
    mut commands: Commands,
    mut selection_state: ResMut<State<SelectionState>>,
    mut board_state: ResMut<BoardState>,
    q_drag_container: Query<Entity, With<DragContainer>>,
    mut do_move_writer: EventWriter<DoMove>,
) {
    match *selection_state.current() {
        SelectionState::Unselected => (),
        SelectionState::SelectingDragging(square) => {
            // Re-parent piece to drag container
            let piece = board_state.piece(square);
            commands.entity(piece).set_parent(q_drag_container.single());
            // Show highlight tile
            let hl_tile = board_state.highlight(square);
            commands.add(ShowHighlight(hl_tile));
            // Show move hints
            commands.add(board_state.show_move_hints_for(square));
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
        SelectionState::DoChangeSelection(from_sq, to_sq) => {
            // Hide highlight tile
            let hl_tile = board_state.highlight(from_sq);
            commands.add(HideHighlight(hl_tile));
            // Hide move hints
            commands.add(board_state.hide_move_hints());
            // Transition to SelectingDragging
            selection_state.overwrite_set(SelectionState::SelectingDragging(to_sq)).unwrap();
        }
        SelectionState::DoMove(from_sq, to_sq) => {
            // Re-parent piece to destination tile & start move
            let piece = board_state.piece(from_sq);
            do_move_writer.send(DoMove { piece, from_sq, to_sq });
            // Hide highlight tile
            let hl_tile = board_state.highlight(from_sq);
            commands.add(HideHighlight(hl_tile));
            // Hide move hints
            commands.add(board_state.hide_move_hints());
            // Transition to Unselected
            selection_state.overwrite_set(SelectionState::Unselected).unwrap();
        }
        SelectionState::DoUnselect(square) => {
            // Re-parent piece back to its tile
            let piece = board_state.piece(square);
            let tile = board_state.tile(square);
            commands.entity(piece).set_parent(tile);
            // Hide highlight tile
            let hl_tile = board_state.highlight(square);
            commands.add(HideHighlight(hl_tile));
            // Hide move hints
            commands.add(board_state.hide_move_hints());
            // Transition to Unselected
            selection_state.overwrite_set(SelectionState::Unselected).unwrap();
        }
    }
}
