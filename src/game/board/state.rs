use std::{
    collections::{hash_map::Entry, HashMap},
    str::FromStr,
};

use bevy::prelude::*;
use chess::{BitBoard, Board, BoardStatus, CastleRights, ChessMove, MoveGen, EMPTY};

use crate::cli::CliArgs;

use super::{PieceColor, PieceType, Square, TileMoveHints};

#[derive(Resource)]
pub struct BoardState {
    tiles: HashMap<Square, Entity>,
    pieces: HashMap<Square, Entity>,
    highlights: HashMap<Square, Entity>,
    move_hints: HashMap<Square, TileMoveHints>,
    board: Board,
}

impl FromWorld for BoardState {
    fn from_world(world: &mut World) -> Self {
        let board = match &world.get_resource::<CliArgs>().and_then(|cli| cli.fen.as_deref()) {
            Some(fen) => match Board::from_str(fen) {
                Ok(board) => board,
                Err(err) => {
                    warn!("{err}");
                    warn!("Using default board");
                    Board::default()
                }
            },
            _ => Board::default(),
        };

        Self {
            tiles: HashMap::with_capacity(64),
            pieces: HashMap::with_capacity(32),
            highlights: HashMap::with_capacity(64),
            move_hints: HashMap::with_capacity(64),
            board,
        }
    }
}

//==================================================
// Getters, setters, and delegates
//==================================================

impl BoardState {
    //------------------------------
    // State
    //------------------------------

    pub fn is_game_over(&self) -> bool {
        matches!(self.board.status(), BoardStatus::Checkmate | BoardStatus::Stalemate)
    }

    pub fn side_to_move(&self) -> PieceColor {
        self.board.side_to_move().into()
    }

    pub fn get_piece_info_on(&self, square: Square) -> Option<(PieceColor, PieceType)> {
        match (self.board.color_on(square.0), self.board.piece_on(square.0)) {
            (Some(color), Some(typ)) => Some((PieceColor(color), PieceType(typ))),
            _ => None,
        }
    }

    pub fn my_castle_rights(&self) -> CastleRights {
        self.board.my_castle_rights()
    }

    pub fn king_square(&self, color: PieceColor) -> Square {
        self.board.king_square(color.0).into()
    }

    fn en_passant(&self) -> Option<Square> {
        self.board.en_passant().map(Square::new)
    }

    pub fn move_is_valid(&self, source: Square, dest: Square) -> bool {
        let mut move_gen = MoveGen::new_legal(&self.board);
        // Mask the generator to only gen moves (by any piece) to the destination.
        move_gen.set_iterator_mask(BitBoard::from_square(dest.0));
        // Return whether any of the generated moves are from the source.
        move_gen.any(|m| m.get_source() == source)
    }

    pub fn reset(&mut self) {
        self.clear_pieces();
        self.board = Board::default();
    }

    #[allow(dead_code)]
    pub fn log(&self) {
        let pretty_board = self.board().to_pretty_string();
        let pretty_state = {
            let mut bb = chess::BoardBuilder::new();
            for &Square(sq) in self.pieces.keys() {
                let typ = self.board.piece_on(sq);
                let color = self.board.color_on(sq);
                if let (Some(typ), Some(color)) = (typ, color) {
                    bb.piece(sq, typ, color);
                } else {
                    warn!(square = %sq, ?color, ?typ, "Failed to get piece info");
                }
            }
            bb.to_pretty_string()
        };
        const SEP: &str = "    ";
        info!("{:15}{SEP}{:15}", "chess::Board:", "Piece State:");
        pretty_board.lines().zip(pretty_state.lines()).for_each(|(a, b)| info!("{a}{SEP}{b}"));
    }

    //------------------------------
    // Board
    //------------------------------

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn set_board(&mut self, board: &Board) {
        self.board = *board;
    }

    //------------------------------
    // Tiles
    //------------------------------

    pub fn tile(&self, square: Square) -> Entity {
        self.tiles.get(&square).copied().unwrap_or_else(|| panic!("no tile at {square}"))
    }

    pub fn set_tile(&mut self, square: Square, entity: Entity) {
        match self.tiles.entry(square) {
            Entry::Occupied(_) => panic!("tile already in the state at {square}"),
            Entry::Vacant(e) => e.insert(entity),
        };
    }

    //------------------------------
    // Highlight tiles
    //------------------------------

    pub fn highlight(&self, square: Square) -> Entity {
        self.highlights.get(&square).copied().unwrap_or_else(|| panic!("no highlight at {square}"))
    }

    pub fn set_highlight(&mut self, square: Square, entity: Entity) {
        match self.highlights.entry(square) {
            Entry::Occupied(_) => panic!("highlight already in the state at {square}"),
            Entry::Vacant(e) => e.insert(entity),
        };
    }

    //------------------------------
    // Hints
    //------------------------------

    pub fn move_hints(&self, square: Square) -> &TileMoveHints {
        self.move_hints.get(&square).unwrap_or_else(|| panic!("no move hints at {square}"))
    }

    pub fn set_move_hints(&mut self, square: Square, hints: TileMoveHints) {
        match self.move_hints.entry(square) {
            Entry::Occupied(_) => panic!("move hints already in the state at {square}"),
            Entry::Vacant(e) => e.insert(hints),
        };
    }

    //------------------------------
    // Pieces
    //------------------------------

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

    pub fn clear_pieces(&mut self) {
        self.pieces.clear();
    }
}

//==================================================
// Core game logic
//==================================================

impl BoardState {
    //------------------------------
    // Move
    //------------------------------

    #[must_use]
    pub fn calculate_valid_moves(&self, source: Square) -> Vec<Entity> {
        let mut move_gen = MoveGen::new_legal(&self.board);
        let mut moves = Vec::with_capacity(move_gen.len());

        let side_to_move_mask = *self.board.color_combined(!self.board.side_to_move());
        move_gen.set_iterator_mask(side_to_move_mask);
        for r#move in &mut move_gen {
            if r#move.get_source() != source {
                continue;
            }
            moves.push(self.move_hints(r#move.get_dest().into()).capture_entity);
        }

        move_gen.set_iterator_mask(!EMPTY);
        for r#move in &mut move_gen {
            if r#move.get_source() != source {
                continue;
            }
            moves.push(self.move_hints(r#move.get_dest().into()).move_entity);
        }

        moves
    }

    pub fn make_board_move(
        &mut self,
        from_sq: Square,
        to_sq: Square,
        promotion: Option<PieceType>,
    ) {
        let r#move = ChessMove::new(from_sq.0, to_sq.0, promotion.map(|p| p.0));
        self.board = self.board.make_move_new(r#move);
    }

    #[must_use]
    pub fn update_piece(
        &mut self,
        color: PieceColor,
        from_sq: Square,
        to_sq: Square,
    ) -> Option<Entity> {
        let (_old_square, piece) = self.pieces.remove_entry(&from_sq).unwrap_or_else(|| {
            panic!("Failed to move board state piece: no piece found at source square {from_sq}")
        });

        match self.en_passant() {
            // `chess::Board::en_passant` returns an optional square which is that of the piece that
            // can be captured in the en passant move that is currently available on the board.
            // The current move is this en passant if there is an en passant square and the
            // destination of the move is the square behind it (from the perspective of the
            // capturer).
            Some(ep_sq) if ep_sq.forward(color).map(|sq| sq == to_sq).unwrap_or(false) => {
                self.pieces.insert(to_sq, piece);
                self.pieces.remove(&ep_sq)
            }
            _ => match self.pieces.entry(to_sq) {
                // Capture
                Entry::Occupied(mut entry) => {
                    let value = entry.get_mut();
                    let old_piece = *value;
                    *value = piece;
                    Some(old_piece)
                }
                // Move
                Entry::Vacant(entry) => {
                    entry.insert(piece);
                    None
                }
            },
        }
    }
}

pub trait ChessBoardExts {
    fn log(&self) {
        self.to_pretty_string().lines().for_each(|l| info!("{l}"));
    }

    fn to_pretty_string(&self) -> String;
}

impl ChessBoardExts for chess::Board {
    fn to_pretty_string(&self) -> String {
        Into::<chess::BoardBuilder>::into(self).to_pretty_string()
    }
}

impl ChessBoardExts for chess::BoardBuilder {
    fn to_pretty_string(&self) -> String {
        let output = String::with_capacity(127);
        chess::ALL_SQUARES.chunks(8).rev().flatten().copied().fold(output, |mut acc, sq| {
            let sq_str = self[sq].map(|(p, c)| p.to_string(c)).unwrap_or_else(|| '.'.into());

            if !acc.is_empty() && sq.to_int() % 8 == 0 {
                acc.push('\n');
            }

            acc.push_str(&sq_str);

            if (sq.to_int() + 1) % 8 > 0 {
                acc.push(' ');
            }

            acc
        })
    }
}
