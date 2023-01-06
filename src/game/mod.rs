use bevy::prelude::*;
use chess::{File, Square};

use crate::{
    data::{BoardState, DoMove, DragContainer, HideHighlight, MoveUiPiece, ShowHighlight},
    game::audio::PlayGameAudio,
};

pub mod audio;
pub mod captures;
pub mod mouse;
pub mod selection;

use self::{
    audio::GameAudioHandles,
    captures::Captured,
    mouse::{mouse_handler, update_drag_container},
    selection::{SelectionEvent, SelectionState},
};

pub struct GameLogicPlugin;

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<GameAudioHandles>()
            // States
            .add_state(SelectionState::Unselected)
            // Events
            .add_event::<SelectionEvent>()
            .add_event::<DoMove>()
            // Systems
            .add_system(mouse_handler)
            .add_system(event_handler.at_end())
            .add_system_set(
                SystemSet::on_enter(SelectionState::SELECTING_DRAGGING).with_system(on_enter),
            )
            .add_system_set(SystemSet::on_enter(SelectionState::SELECTED).with_system(on_enter))
            .add_system_set(
                SystemSet::on_enter(SelectionState::SELECTED_DRAGGING).with_system(on_enter),
            )
            .add_system_set(
                SystemSet::on_enter(SelectionState::DO_CHANGE_SELECTION).with_system(on_enter),
            )
            .add_system_set(SystemSet::on_enter(SelectionState::DO_MOVE).with_system(on_enter))
            .add_system_set(SystemSet::on_enter(SelectionState::DO_UNSELECT).with_system(on_enter))
            .add_system(update_drag_container)
            .add_system(move_piece);
    }
}

fn event_handler(
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

fn on_enter(
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
            let piece = board_state.piece(square).entity;
            commands.entity(piece).set_parent(q_drag_container.single());
            // Show highlight tile
            let hl_tile = board_state.highlight(square);
            commands.add(ShowHighlight(hl_tile));
            // Show move hints
            commands.add(board_state.show_move_hints_for(square));
        }
        SelectionState::Selected(square) => {
            // Re-parent piece back to its tile
            let piece = board_state.piece(square).entity;
            let tile = board_state.tile(square);
            commands.entity(piece).set_parent(tile);
        }
        SelectionState::SelectedDragging(square) => {
            // Re-parent piece to drag container
            let piece = board_state.piece(square).entity;
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
            let piece = board_state.piece(square).entity;
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

fn move_piece(
    mut commands: Commands,
    mut board_state: ResMut<BoardState>,
    mut do_move_reader: EventReader<DoMove>,
) {
    for &DoMove { piece, from_sq, to_sq } in do_move_reader.iter() {
        let mut was_castle = false;

        if *piece.typ == chess::Piece::King {
            let castle_rights = board_state.board().my_castle_rights();
            let back_rank = piece.color.to_my_backrank();
            let kingside_sq = Square::make_square(back_rank, File::G);
            let queenside_sq = Square::make_square(back_rank, File::C);

            // Move rook
            if castle_rights.has_kingside() && to_sq == kingside_sq {
                let piece = board_state.piece(Square::make_square(back_rank, File::H));
                let to_sq = Square::make_square(back_rank, File::F);
                commands.add(MoveUiPiece { piece, to_sq });
                was_castle = true;
            } else if castle_rights.has_queenside() && to_sq == queenside_sq {
                let piece = board_state.piece(Square::make_square(back_rank, File::A));
                let to_sq = Square::make_square(back_rank, File::D);
                commands.add(MoveUiPiece { piece, to_sq });
                was_castle = true;
            }
        }

        // Move piece & play audio
        commands.add(MoveUiPiece { piece, to_sq });
        if let Some(piece) = board_state.move_piece(from_sq, to_sq) {
            commands.add(Captured(piece));
            commands.add(PlayGameAudio::Capture);
        } else if was_castle {
            commands.add(PlayGameAudio::Castle);
        } else {
            commands.add(match *piece.color {
                chess::Color::Black => PlayGameAudio::MoveOpponent,
                chess::Color::White => PlayGameAudio::MoveSelf,
            });
        }
    }
}
