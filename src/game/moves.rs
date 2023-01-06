use bevy::{ecs::system::Command, prelude::*};
use chess::{File, Square};

use crate::{
    data::{BoardPiece, BoardState, UiSquare},
    game::audio::PlayGameAudio,
};

use super::captures::Captured;

pub struct DoMove {
    pub piece: BoardPiece,
    pub from_sq: Square,
    pub to_sq: Square,
}

pub struct MoveUiPiece {
    pub piece: BoardPiece,
    pub to_sq: Square,
}

impl Command for MoveUiPiece {
    fn write(self, world: &mut World) {
        let mut entity = world.entity_mut(self.piece.entity);
        if let Some(mut square) = entity.get_mut::<UiSquare>() {
            square.move_to(self.to_sq);
        }

        let board_state = world.resource::<BoardState>();
        let to_tile = board_state.tile(self.to_sq);
        world.entity_mut(to_tile).push_children(&[self.piece.entity]);
    }
}

pub fn move_piece(
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
