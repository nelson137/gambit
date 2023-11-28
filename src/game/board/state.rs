use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Write,
    str::FromStr,
};

use bevy::prelude::*;
use chess::{BitBoard, Board, BoardStatus, CastleRights, ChessMove, MoveGen, EMPTY};

use crate::cli::CliArgs;

use super::{PieceColor, PieceMeta, PieceType, Square, TileHints};

#[derive(Resource)]
pub struct BoardState {
    status: GameStatus,
    tiles: HashMap<Square, Entity>,
    pieces: HashMap<Square, Entity>,
    highlights: HashMap<Square, Entity>,
    tile_hints: HashMap<Square, TileHints>,
    board: Board,
    half_move_clock: u8,
    full_move_count: u16,
    piece_state_counters: HashMap<u64, u8>,
}

impl FromWorld for BoardState {
    fn from_world(world: &mut World) -> Self {
        let fen = world.get_resource::<CliArgs>().and_then(|cli| cli.fen.as_deref());
        let (board, half_move_clock, full_move_count) = match fen {
            Some(fen) => match parse_fen(fen) {
                Ok(data) => data,
                Err(err) => {
                    warn!("{err}");
                    warn!("Using default board");
                    (Board::default(), 0, 0)
                }
            },
            _ => (Board::default(), 0, 0),
        };

        let mut piece_state_counters = HashMap::with_capacity(300);
        *piece_state_counters.entry(board.get_hash()).or_default() = 1;

        Self {
            status: GameStatus::Ongoing,
            tiles: HashMap::with_capacity(64),
            pieces: HashMap::with_capacity(32),
            highlights: HashMap::with_capacity(64),
            tile_hints: HashMap::with_capacity(64),
            board,
            half_move_clock,
            full_move_count,
            piece_state_counters,
        }
    }
}

fn parse_fen(fen: &str) -> Result<(Board, u8, u16), chess::Error> {
    let board = Board::from_str(fen)?;
    let invalid_fen = || chess::Error::InvalidFen { fen: fen.to_string() };

    let mut fen_iter = fen.split(' ').skip(4);
    match (fen_iter.next(), fen_iter.next()) {
        (Some(half_move_raw), Some(full_move_raw)) => Ok((
            board,
            half_move_raw.parse().map_err(|_| invalid_fen())?,
            full_move_raw.parse().map_err(|_| invalid_fen())?,
        )),
        _ => Err(invalid_fen()),
    }
}

//==================================================
// Getters, setters, and delegates
//==================================================

impl BoardState {
    //------------------------------
    // State
    //------------------------------

    pub fn status(&self) -> GameStatus {
        self.status
    }

    pub fn is_game_over(&self) -> bool {
        matches!(
            self.status,
            GameStatus::GameOverCheckmate
                | GameStatus::GameOverStalemate
                | GameStatus::GameOverRepetition
                | GameStatus::GameOver50Moves
        )
    }

    pub fn fen(&self) -> String {
        let mut fen = self.board.to_string();
        fen.truncate(fen.len() - 3);
        let half_move = self.half_move_clock;
        let full_move = self.full_move_count;
        write!(&mut fen, "{half_move} {full_move}")
            .expect("Write halfmove clock & fullmove count to FEN");
        fen
    }

    pub fn side_to_move(&self) -> PieceColor {
        self.board.side_to_move().into()
    }

    pub fn get_piece_meta(&self, square: Square) -> Option<PieceMeta> {
        let color = self.board.color_on(square.0).map(PieceColor);
        let typ = self.board.piece_on(square.0).map(PieceType);
        match (color, typ) {
            (Some(color), Some(typ)) => Some(PieceMeta::new(color, typ)),
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

    pub fn tile_hints(&self, square: Square) -> &TileHints {
        self.tile_hints.get(&square).unwrap_or_else(|| panic!("no move hints at {square}"))
    }

    pub fn set_tile_hints(&mut self, square: Square, hints: TileHints) {
        match self.tile_hints.entry(square) {
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
    // Status
    //------------------------------

    pub fn sync_status(&mut self) {
        self.status = match self.board.status() {
            BoardStatus::Checkmate => GameStatus::GameOverCheckmate,
            BoardStatus::Stalemate => GameStatus::GameOverStalemate,
            BoardStatus::Ongoing if self.is_game_over_50_moves() => GameStatus::GameOver50Moves,
            BoardStatus::Ongoing if self.is_game_over_repetition() => {
                GameStatus::GameOverRepetition
            }
            BoardStatus::Ongoing => GameStatus::Ongoing,
        };
    }

    /// Internal helper. **Only to be used by `Self::sync_status`**.
    fn is_game_over_50_moves(&self) -> bool {
        self.half_move_clock >= 100
    }

    /// Internal helper. **Only to be used by `Self::sync_status`**.
    fn is_game_over_repetition(&self) -> bool {
        self.piece_state_counters.values().copied().any(|count| count >= 3)
    }

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
            moves.push(self.tile_hints(r#move.get_dest().into()).capture_entity);
        }

        move_gen.set_iterator_mask(!EMPTY);
        for r#move in &mut move_gen {
            if r#move.get_source() != source {
                continue;
            }
            moves.push(self.tile_hints(r#move.get_dest().into()).move_entity);
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

        let hash = self.board.get_hash();
        *self.piece_state_counters.entry(hash).or_default() += 1;
    }

    pub fn inc_half_move_clock(&mut self) {
        self.half_move_clock += 1;
    }

    pub fn reset_half_move_clock(&mut self) {
        self.half_move_clock = 0;
    }

    pub fn inc_full_move_count(&mut self) {
        self.full_move_count += 1;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameStatus {
    Ongoing,
    GameOverCheckmate,
    GameOverStalemate,
    GameOver50Moves,
    GameOverRepetition,
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
