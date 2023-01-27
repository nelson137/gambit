use std::{
    collections::{hash_map::Entry, HashMap},
    str::FromStr,
};

use bevy::prelude::*;
use chess::{BitBoard, Board, BoardStatus, ChessMove, File, MoveGen, Square, EMPTY};

use crate::{
    cli::CliArgs,
    game::{
        audio::PlayGameAudio,
        captures::Captured,
        game_over::GameOver,
        moves::{DoMove, MoveUiPiece},
        utils::GameCommandList,
    },
};

use super::{HideHighlight, HideHints, MoveHints, ShowHighlight, ShowHints};

/// The maximum possible valid moves that any piece could ever have in a game: 27.
///
/// This is the number of valid moves that a queen can make when on one of the four middle squares
/// of the board (d4, e4, d5, or e5), with no other pieces blocking any of the eight rays of
/// possible movement.
///
/// Below is a diagram showing one of such configurations. When a queen is on, e.g., d4, she can
/// move to any of the squares that contain an `x`.
///
/// ```text
///   ┌───┬───┬───┬───┬───┬───┬───┬───┐
/// 8 │   │   │   │ x │   │   │   │ x │
///   ├───┼───┼───┼───┼───┼───┼───┼───┤
/// 7 │ x │   │   │ x │   │   │ x │   │
///   ├───┼───┼───┼───┼───┼───┼───┼───┤
/// 6 │   │ x │   │ x │   │ x │   │   │
///   ├───┼───┼───┼───┼───┼───┼───┼───┤
/// 5 │   │   │ x │ x │ x │   │   │   │
///   ├───┼───┼───┼───┼───┼───┼───┼───┤
/// 4 │ x │ x │ x │ Q │ x │ x │ x │ x │
///   ├───┼───┼───┼───┼───┼───┼───┼───┤
/// 3 │   │   │ x │ x │ x │   │   │   │
///   ├───┼───┼───┼───┼───┼───┼───┼───┤
/// 2 │   │ x │   │ x │   │ x │   │   │
///   ├───┼───┼───┼───┼───┼───┼───┼───┤
/// 1 │ x │   │   │ x │   │   │ x │   │
///   └───┴───┴───┴───┴───┴───┴───┴───┘
///     a   b   c   d   e   f   g   h
/// ```
///
const MAX_POSSIBLE_MOVES: usize = 27;

#[derive(Resource)]
pub struct BoardState {
    tiles: HashMap<Square, Entity>,
    pieces: HashMap<Square, Entity>,
    highlights: HashMap<Square, Entity>,
    move_hints: HashMap<Square, MoveHints>,
    board: Board,
    current_highlight: Option<Entity>,
    showing_hints: Vec<Entity>,
}

impl FromWorld for BoardState {
    fn from_world(world: &mut World) -> Self {
        let board = match &world.resource::<CliArgs>().fen {
            Some(fen) => Board::from_str(fen).unwrap(),
            _ => Board::default(),
        };

        Self {
            tiles: HashMap::with_capacity(64),
            pieces: HashMap::with_capacity(32),
            highlights: HashMap::with_capacity(64),
            move_hints: HashMap::with_capacity(64),
            board,
            current_highlight: None,
            showing_hints: Vec::with_capacity(MAX_POSSIBLE_MOVES),
        }
    }
}

impl BoardState {
    pub fn reset(&mut self) {
        self.clear_pieces();
        self.clear_showing_hints();
        self.board = Board::default();
    }

    pub fn clear_showing_hints(&mut self) {
        self.showing_hints.clear();
    }

    pub fn clear_pieces(&mut self) {
        self.pieces.clear();
    }

    pub fn get_piece_info_on(&self, square: Square) -> Option<(chess::Color, chess::Piece)> {
        self.board
            .color_on(square)
            .map(|c| (c, self.board.piece_on(square).expect("invalid board")))
    }

    fn color_on(&self, square: Square) -> chess::Color {
        self.board.color_on(square).unwrap_or_else(|| panic!("no piece at {square}"))
    }

    fn piece_on(&self, square: Square) -> chess::Piece {
        self.board.piece_on(square).unwrap_or_else(|| panic!("no piece at {square}"))
    }

    pub fn is_colors_turn_at(&self, square: Square) -> bool {
        self.color_on(square) == self.board.side_to_move()
    }

    pub fn tile(&self, square: Square) -> Entity {
        self.tiles.get(&square).copied().unwrap_or_else(|| panic!("no tile at {square}"))
    }

    pub fn set_tile(&mut self, square: Square, entity: Entity) {
        match self.tiles.entry(square) {
            Entry::Occupied(_) => panic!("tile already in the state at {square}"),
            Entry::Vacant(e) => e.insert(entity),
        };
    }

    pub fn has_piece_at(&self, square: Square) -> bool {
        self.pieces.contains_key(&square)
    }

    pub fn piece(&self, square: Square) -> Entity {
        self.get_piece(square).unwrap_or_else(|| panic!("no piece at {square}"))
    }

    pub fn get_piece(&self, square: Square) -> Option<Entity> {
        self.pieces.get(&square).copied()
    }

    pub fn set_piece(&mut self, square: Square, piece: Entity) {
        match self.pieces.entry(square) {
            Entry::Occupied(_) => panic!("piece already in the state at {square}"),
            Entry::Vacant(e) => e.insert(piece),
        };
    }

    pub fn highlight(&self, square: Square) -> Entity {
        self.highlights.get(&square).copied().unwrap_or_else(|| panic!("no highlight at {square}"))
    }

    pub fn set_highlight(&mut self, square: Square, entity: Entity) {
        match self.highlights.entry(square) {
            Entry::Occupied(_) => panic!("highlight already in the state at {square}"),
            Entry::Vacant(e) => e.insert(entity),
        };
    }

    pub fn move_hints(&self, square: Square) -> &MoveHints {
        self.move_hints.get(&square).unwrap_or_else(|| panic!("no move hints at {square}"))
    }

    pub fn set_move_hints(&mut self, square: Square, hints: MoveHints) {
        match self.move_hints.entry(square) {
            Entry::Occupied(_) => panic!("move hints already in the state at {square}"),
            Entry::Vacant(e) => e.insert(hints),
        };
    }

    pub fn board(&self) -> Board {
        self.board
    }

    pub fn set_board(&mut self, board: &Board) {
        self.board = *board;
    }

    #[must_use]
    pub fn show_highlight_tile(&mut self, square: Square) -> ShowHighlight {
        let entity = self.highlight(square);
        self.current_highlight = Some(entity);
        ShowHighlight(entity)
    }

    #[must_use]
    pub fn hide_highlight_tile(&mut self) -> HideHighlight {
        HideHighlight(
            self.current_highlight
                .take()
                .unwrap_or_else(|| panic!("Failed to hide highlight tile, none are shown")),
        )
    }

    #[must_use]
    pub fn show_move_hints_for(&mut self, source: Square) -> ShowHints {
        if !self.is_colors_turn_at(source) {
            return Default::default();
        }

        let mut move_gen = MoveGen::new_legal(&self.board);
        let mut moves = Vec::with_capacity(move_gen.len());

        let side_to_move_mask = *self.board.color_combined(!self.board.side_to_move());
        move_gen.set_iterator_mask(side_to_move_mask);
        for r#move in &mut move_gen {
            if r#move.get_source() != source {
                continue;
            }
            moves.push(self.move_hints(r#move.get_dest()).capture_entity);
        }

        move_gen.set_iterator_mask(!EMPTY);
        for r#move in &mut move_gen {
            if r#move.get_source() != source {
                continue;
            }
            moves.push(self.move_hints(r#move.get_dest()).move_entity);
        }

        if !moves.is_empty() {
            self.showing_hints.extend(&moves);
        }
        ShowHints(moves)
    }

    #[must_use]
    pub fn hide_move_hints(&mut self) -> HideHints {
        HideHints(if self.showing_hints.is_empty() {
            Vec::new()
        } else {
            self.showing_hints.drain(..).collect()
        })
    }

    pub fn move_is_valid(&self, source: Square, dest: Square) -> bool {
        let mut move_gen = MoveGen::new_legal(&self.board);
        // Mask the generator to only gen moves (by any piece) to the destination.
        move_gen.set_iterator_mask(BitBoard::from_square(dest));
        // Return whether any of the generated moves are from the source.
        move_gen.any(|m| m.get_source() == source)
    }

    #[must_use]
    pub fn move_piece(&mut self, DoMove { piece, from_sq, to_sq }: DoMove) -> GameCommandList {
        let mut cmd_list = GameCommandList::default();

        let (color, typ) = match (self.board.color_on(from_sq), self.board.piece_on(from_sq)) {
            (Some(c), Some(t)) => (c, t),
            _ => return cmd_list,
        };

        // Move UI piece
        cmd_list.add(MoveUiPiece { piece, to_sq });

        let mut is_castle = false;
        if typ == chess::Piece::King {
            let castle_rights = self.board.my_castle_rights();
            let back_rank = color.to_my_backrank();
            let kingside_sq = Square::make_square(back_rank, File::G);
            let queenside_sq = Square::make_square(back_rank, File::C);

            // Move UI rook
            if castle_rights.has_kingside() && to_sq == kingside_sq {
                let piece = self.piece(Square::make_square(back_rank, File::H));
                let to_sq = Square::make_square(back_rank, File::F);
                cmd_list.add(MoveUiPiece { piece, to_sq });
                is_castle = true;
            } else if castle_rights.has_queenside() && to_sq == queenside_sq {
                let piece = self.piece(Square::make_square(back_rank, File::A));
                let to_sq = Square::make_square(back_rank, File::D);
                cmd_list.add(MoveUiPiece { piece, to_sq });
                is_castle = true;
            }
        }

        // Update pieces map
        let (_old_square, piece) = self
            .pieces
            .remove_entry(&from_sq)
            .expect("Failed to move board state piece: no piece found at source square");
        let captured_piece = match self.board.en_passant() {
            // `chess::Board::en_passant` returns an optional square which is that of the piece that
            // can be captured in the en passant move that is currently available on the board.
            // The current move is this en passant if there is an en passant square and the
            // destination of the move is the square behind it (from the perspective of the *other*
            // player, hence the `!color`).
            Some(ep_sq) if ep_sq.backward(!color).map(|sq| sq == to_sq).unwrap_or(false) => {
                self.pieces.insert(to_sq, piece);
                self.pieces
                    .remove(&ep_sq)
                    .map(|entity| (entity, self.color_on(ep_sq), self.piece_on(ep_sq)))
            }
            _ => match self.pieces.entry(to_sq) {
                // Capture
                Entry::Occupied(mut entry) => {
                    let value = entry.get_mut();
                    let old_piece = *value;
                    *value = piece;
                    Some((old_piece, self.color_on(to_sq), self.piece_on(to_sq)))
                }
                // Move
                Entry::Vacant(entry) => {
                    entry.insert(piece);
                    None
                }
            },
        };

        // Make move on board
        self.board = self.board.make_move_new(ChessMove::new(from_sq, to_sq, None));

        // Play audio
        if let Some((cap_entity, cap_color, cap_typ)) = captured_piece {
            cmd_list.add(Captured::new(cap_entity, cap_color, cap_typ));
            cmd_list.add(PlayGameAudio::Capture);
        } else if is_castle {
            cmd_list.add(PlayGameAudio::Castle);
        } else {
            cmd_list.add(match color {
                chess::Color::Black => PlayGameAudio::MoveOpponent,
                chess::Color::White => PlayGameAudio::MoveSelf,
            });
        }

        if let BoardStatus::Checkmate | BoardStatus::Stalemate = self.board.status() {
            cmd_list.add(GameOver);
        }

        cmd_list
    }
}
