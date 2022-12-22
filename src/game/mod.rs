use bevy::prelude::*;
use chess::{File, Square};

use crate::data::{
    BoardState, DoMove, DoUpdatePieceSquare, Dragging, Dropped, HideHighlight, HideHint,
    ShowHighlight, ShowHint, ShowingMovesFor, UiPiece, UiSquare,
};

pub mod mouse;
pub mod selection;

use self::{
    mouse::{end_drag, mouse_handler, start_drag},
    selection::{SelectionEvent, SelectionState},
};

pub struct GameLogicPlugin;

impl Plugin for GameLogicPlugin {
    fn build(&self, app: &mut App) {
        app
            // States
            .add_state(SelectionState::Unselected)
            // Resources
            .init_resource::<ShowingMovesFor>()
            // Events
            .add_event::<SelectionEvent>()
            // Systems
            .add_system(mouse_handler)
            .add_system(event_handler.at_end())
            .add_system_set(
                SystemSet::on_enter(SelectionState::SelectingDragging(default()))
                    .with_system(on_enter),
            )
            .add_system_set(
                SystemSet::on_enter(SelectionState::Selected(default())).with_system(on_enter),
            )
            .add_system_set(
                SystemSet::on_enter(SelectionState::SelectedDragging(default()))
                    .with_system(on_enter),
            )
            .add_system_set(
                SystemSet::on_enter(SelectionState::DoMove(default(), default()))
                    .with_system(on_enter),
            )
            .add_system_set(
                SystemSet::on_enter(SelectionState::DoUnselect(default())).with_system(on_enter),
            )
            .add_system(start_drag)
            .add_system(end_drag)
            .add_system(hide_highlight)
            .add_system(show_highlight)
            .add_system(hide_hints)
            .add_system(show_hints)
            .add_system(move_piece)
            .add_system(update_piece_square);
    }
}

fn event_handler(
    mut selection_state: ResMut<State<SelectionState>>,
    board_state: Res<BoardState>,
    mut event_reader: EventReader<SelectionEvent>,
) {
    for event in event_reader.iter() {
        match selection_state.current() {
            SelectionState::Unselected => match *event {
                SelectionEvent::MouseDown(square) => {
                    selection_state
                        .set(SelectionState::SelectingDragging(square))
                        .expect("failed to set SelectingDragging");
                }
                SelectionEvent::MouseUp(_) => (),
            },
            &SelectionState::SelectingDragging(selecting_sq) => match *event {
                SelectionEvent::MouseDown(_) => todo!("reset previous drag target"), // TODO
                SelectionEvent::MouseUp(square) => {
                    if board_state.move_is_valid(selecting_sq, square) {
                        selection_state
                            .set(SelectionState::DoMove(selecting_sq, square))
                            .expect("failed to set Move");
                    } else {
                        selection_state
                            .set(SelectionState::Selected(selecting_sq))
                            .expect("failed to set Selected");
                    }
                }
            },
            &SelectionState::Selected(selected_sq) => match *event {
                SelectionEvent::MouseDown(square) => {
                    if square == selected_sq {
                        selection_state
                            .set(SelectionState::SelectedDragging(selected_sq))
                            .expect("failed to set SelectedDragging");
                    } else if board_state.move_is_valid(selected_sq, square) {
                        selection_state
                            .set(SelectionState::DoMove(selected_sq, square))
                            .expect("failed to set Move");
                    } else {
                        selection_state
                            .set(SelectionState::DoUnselect(selected_sq))
                            .expect("failed to set Unselected");
                    }
                }
                SelectionEvent::MouseUp(_) => (),
            },
            &SelectionState::SelectedDragging(selected_sq) => match *event {
                SelectionEvent::MouseDown(_) => todo!("reset previous drag target"), // TODO
                SelectionEvent::MouseUp(square) => {
                    if square == selected_sq {
                        selection_state
                            .set(SelectionState::DoUnselect(selected_sq))
                            .expect("failed to set Unselected");
                    } else if board_state.move_is_valid(selected_sq, square) {
                        selection_state
                            .set(SelectionState::DoMove(selected_sq, square))
                            .expect("failed to set Move");
                    } else {
                        selection_state
                            .set(SelectionState::Selected(selected_sq))
                            .expect("failed to set Selected");
                    }
                }
            },
            SelectionState::DoMove(_, _) => (),
            SelectionState::DoUnselect(_) => (),
        }
    }
}

fn on_enter(
    mut commands: Commands,
    mut selection_state: ResMut<State<SelectionState>>,
    mut board_state: ResMut<BoardState>,
    mut showing_piece_moves: ResMut<ShowingMovesFor>,
) {
    match selection_state.current() {
        SelectionState::Unselected => (),
        SelectionState::SelectingDragging(square) => {
            // Start piece drag
            let piece = board_state.pieces.get(square).expect("failed to get piece entity").entity;
            commands.entity(piece).insert(Dragging::new(*square));
            // Show highlight tile
            let hl_tile =
                *board_state.highlights.get(square).expect("failed to get highlight tile entity");
            commands.entity(hl_tile).insert(ShowHighlight);
            // Show move hints
            if board_state.is_colors_turn_at(*square) {
                **showing_piece_moves = Some(*square);
                board_state.show_piece_move_hints(&mut commands, *square);
            }
        }
        SelectionState::Selected(square) => {
            // Start piece drop
            let piece = board_state.pieces.get(square).expect("failed to get piece entity").entity;
            commands.entity(piece).insert(Dropped);
        }
        SelectionState::SelectedDragging(square) => {
            // Start piece drag
            let piece = board_state.pieces.get(square).expect("failed to get piece entity").entity;
            commands.entity(piece).insert(Dragging::new(*square));
        }
        SelectionState::DoMove(from_sq, to_sq) => {
            // Drop piece & start move
            let piece = board_state.pieces.get(from_sq).expect("failed to get piece entity").entity;
            commands.entity(piece).insert((Dropped, DoMove(*to_sq)));
            // Hide highlight tile
            let hl_tile =
                *board_state.highlights.get(from_sq).expect("failed to get highlight tile entity");
            commands.entity(hl_tile).insert(HideHighlight);
            // Hide move hints
            if showing_piece_moves.is_some() {
                board_state.hide_piece_move_hints(&mut commands);
                **showing_piece_moves = None;
            }
            // Transition to Unselected
            selection_state
                .overwrite_set(SelectionState::Unselected)
                .expect("failed to set Unselected");
        }
        SelectionState::DoUnselect(square) => {
            // Drop piece
            let piece = board_state.pieces.get(square).expect("failed to get piece entity").entity;
            commands.entity(piece).insert(Dropped);
            // Hide highlight tile
            let hl_tile =
                *board_state.highlights.get(square).expect("failed to get highlight tile entity");
            commands.entity(hl_tile).insert(HideHighlight);
            // Hide move hints
            if showing_piece_moves.is_some() {
                board_state.hide_piece_move_hints(&mut commands);
                **showing_piece_moves = None;
            }
            // Transition to Unselected
            selection_state
                .overwrite_set(SelectionState::Unselected)
                .expect("failed to set Unselected");
        }
    }
}

fn show_highlight(
    mut commands: Commands,
    mut q_added_show: Query<(Entity, &mut Visibility), Added<ShowHighlight>>,
) {
    for (entity, mut vis) in &mut q_added_show {
        commands.entity(entity).remove::<ShowHighlight>();
        vis.is_visible = true;
    }
}

fn hide_highlight(
    mut commands: Commands,
    mut q_added_hide: Query<(Entity, &mut Visibility), Added<HideHighlight>>,
) {
    for (entity, mut vis) in &mut q_added_hide {
        commands.entity(entity).remove::<HideHighlight>();
        vis.is_visible = false;
    }
}

fn show_hints(
    mut commands: Commands,
    mut q_show_hints: Query<(Entity, &mut Visibility), Added<ShowHint>>,
) {
    for (entity, mut vis) in &mut q_show_hints {
        commands.entity(entity).remove::<ShowHint>();
        vis.is_visible = true;
    }
}

fn hide_hints(
    mut commands: Commands,
    mut q_hide_hints: Query<(Entity, &mut Visibility), Added<HideHint>>,
) {
    for (entity, mut vis) in &mut q_hide_hints {
        commands.entity(entity).remove::<HideHint>();
        vis.is_visible = false;
    }
}

fn move_piece(
    mut commands: Commands,
    mut board_state: ResMut<BoardState>,
    mut q_move: Query<(Entity, &UiPiece, &UiSquare, &DoMove), Added<DoMove>>,
) {
    for (entity, piece, square, do_move) in &mut q_move {
        let mut cmds = commands.entity(entity);
        cmds.remove::<DoMove>();

        let dest = **do_move;

        if *piece.typ == chess::Piece::King {
            let castle_rights = board_state.move_gen_board.my_castle_rights();
            let back_rank = piece.color.to_my_backrank();
            let kingside_sq = Square::make_square(back_rank, File::G);
            let queenside_sq = Square::make_square(back_rank, File::C);

            // Move king
            board_state.move_piece(**square, dest);
            cmds.insert(DoUpdatePieceSquare(dest));

            // Move rook
            if castle_rights.has_kingside() && dest == kingside_sq {
                let rook = board_state
                    .pieces
                    .get(&Square::make_square(back_rank, File::H))
                    .expect("castle is valid but the kingside rook is not on its starting square");
                commands
                    .entity(rook.entity)
                    .insert(DoUpdatePieceSquare(Square::make_square(back_rank, File::F)));
            } else if castle_rights.has_queenside() && dest == queenside_sq {
                warn!("TODO: move rook"); // TODO
            }
        } else {
            board_state.move_piece(**square, dest);
            cmds.insert(DoUpdatePieceSquare(dest));
        }
    }
}

fn update_piece_square(
    mut commands: Commands,
    mut q_update: Query<(Entity, &mut UiSquare, &DoUpdatePieceSquare), Added<DoUpdatePieceSquare>>,
) {
    for (entity, mut square, update) in &mut q_update {
        commands.entity(entity).remove::<DoUpdatePieceSquare>();
        square.move_to(**update);
    }
}
