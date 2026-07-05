use std::{
    collections::{HashMap, hash_map::Entry},
    str::FromStr,
};

use bevy::prelude::*;
use chess::{BitBoard, Board, BoardStatus, CastleRights, ChessMove, EMPTY, MoveGen};

use crate::{cli::CliArgs, game::LoadGame};

use super::{HintDescriptor, PieceColor, PieceMeta, PieceType, PiecesMap, Square};

#[derive(Resource)]
pub struct BoardState {
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
                    (Board::default(), 0, 1)
                }
            },
            None => (Board::default(), 0, 1),
        };

        let mut piece_state_counters = HashMap::with_capacity(300);
        *piece_state_counters.entry(board.get_hash()).or_default() = 1;

        Self { board, half_move_clock, full_move_count, piece_state_counters }
    }
}

pub fn parse_fen(fen: &str) -> Result<(Board, u8, u16), chess::Error> {
    let fen = fen.trim();

    if fen == "default" {
        return Ok((Board::default(), 0, 1));
    }

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

pub(super) fn set_board_on_load_game(load_game: On<LoadGame>, mut board_state: ResMut<BoardState>) {
    board_state.set_board(&load_game.board, load_game.half_move_clock, load_game.full_move_count);
}

//==================================================
// Getters, setters, and delegates
//==================================================

impl BoardState {
    //------------------------------
    // State
    //------------------------------

    #[cfg(feature = "stockfish-player")]
    pub fn fen(&self) -> String {
        use std::fmt::Write;
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

    //------------------------------
    // Board
    //------------------------------

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn set_board(&mut self, board: &Board, half_move_clock: u8, full_move_count: u16) {
        self.board = *board;
        self.half_move_clock = half_move_clock;
        self.full_move_count = full_move_count;
    }
}

//==================================================
// Core game logic
//==================================================

impl BoardState {
    //------------------------------
    // Status
    //------------------------------

    pub fn game_status(&mut self) -> GameStatus {
        match self.board.status() {
            BoardStatus::Checkmate => GameStatus::GameOverCheckmate,
            BoardStatus::Stalemate => GameStatus::GameOverStalemate,
            BoardStatus::Ongoing if self.is_game_over_50_moves() => GameStatus::GameOver50Moves,
            BoardStatus::Ongoing if self.is_game_over_repetition() => {
                GameStatus::GameOverRepetition
            }
            BoardStatus::Ongoing => GameStatus::Ongoing,
        }
    }

    /// Internal helper. **Only to be used by `Self::game_status`**.
    fn is_game_over_50_moves(&self) -> bool {
        self.half_move_clock >= 100
    }

    /// Internal helper. **Only to be used by `Self::game_status`**.
    fn is_game_over_repetition(&self) -> bool {
        self.piece_state_counters.values().copied().any(|count| count >= 3)
    }

    //------------------------------
    // Move
    //------------------------------

    #[must_use]
    pub fn calculate_valid_moves(&self, source: Square) -> Vec<HintDescriptor> {
        let mut move_gen = MoveGen::new_legal(&self.board);
        let mut moves = Vec::with_capacity(move_gen.len());

        let side_to_move_mask = *self.board.color_combined(!self.board.side_to_move());
        move_gen.set_iterator_mask(side_to_move_mask);
        for r#move in &mut move_gen {
            if r#move.get_source() != source {
                continue;
            }
            moves.push(HintDescriptor::capture(r#move.get_dest().into()));
        }

        move_gen.set_iterator_mask(!EMPTY);
        for r#move in &mut move_gen {
            if r#move.get_source() != source {
                continue;
            }
            moves.push(HintDescriptor::r#move(r#move.get_dest().into()));
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

    // TODO: docs
    // `chess::Board::en_passant()` only returns `Some(_)` if the last move was a pawn double move
    // *and* there is a pawn of the opposite color adjacent to the destination square.
    pub fn move_is_en_passant(&self, color: PieceColor, to_sq: Square) -> bool {
        let ep = self.en_passant();
        ep.is_some() && to_sq.backward(color) == ep
    }

    #[must_use]
    pub fn update_piece(
        &mut self,
        pieces_map: &mut PiecesMap,
        color: PieceColor,
        from_sq: Square,
        to_sq: Square,
    ) -> Option<Entity> {
        let piece = pieces_map.remove(from_sq).unwrap_or_else(|| {
            panic!("Failed to move board state piece: no piece found at source square {from_sq}")
        });

        match self.en_passant() {
            // `chess::Board::en_passant` returns an optional square which is that of the piece that
            // can be captured in the en passant move that is currently available on the board.
            // The current move is this en passant if there is an en passant square and the
            // destination of the move is the square ahead of it (from the perspective of the
            // current player).
            Some(ep_sq) if ep_sq.forward(color) == Some(to_sq) => {
                pieces_map.move_en_passant(to_sq, piece, ep_sq)
            }
            _ => match pieces_map.entry(to_sq) {
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

#[derive(Resource, Clone, Copy, Default, Debug, PartialEq, Eq)]
pub enum GameStatus {
    #[default]
    Ongoing,
    GameOverCheckmate,
    GameOverStalemate,
    GameOver50Moves,
    GameOverRepetition,
}

impl GameStatus {
    pub fn is_game_over(self) -> bool {
        matches!(
            self,
            Self::GameOverCheckmate
                | Self::GameOverStalemate
                | Self::GameOverRepetition
                | Self::GameOver50Moves
        )
    }
}

pub trait ChessBoardExts {
    fn get_piece_meta(&self, square: Square) -> Option<PieceMeta>;
}

impl ChessBoardExts for chess::Board {
    fn get_piece_meta(&self, square: Square) -> Option<PieceMeta> {
        let color = self.color_on(square.0).map(PieceColor);
        let typ = self.piece_on(square.0).map(PieceType);
        match (color, typ) {
            (Some(color), Some(typ)) => Some(PieceMeta::new(color, typ)),
            _ => None,
        }
    }
}

#[allow(dead_code)]
pub trait ChessBoardExtsDebug {
    #[cfg(debug_assertions)]
    fn to_pretty_string(&self) -> String;

    #[cfg(debug_assertions)]
    fn log(&self) {
        let pretty = self.to_pretty_string();
        for line in pretty.lines() {
            info!("{line}");
        }
    }
}

#[cfg(debug_assertions)]
impl ChessBoardExtsDebug for chess::BitBoard {
    fn to_pretty_string(&self) -> String {
        let output = String::with_capacity(127);
        chess::ALL_SQUARES.chunks(8).rev().flatten().copied().fold(output, |mut acc, sq| {
            if !acc.is_empty() && sq.to_int().is_multiple_of(8) {
                acc.push('\n');
            }

            acc.push(if self & chess::BitBoard::from_square(sq) == chess::EMPTY {
                '.'
            } else {
                'X'
            });

            if !(sq.to_int() + 1).is_multiple_of(8) {
                acc.push(' ');
            }

            acc
        })
    }
}

#[cfg(debug_assertions)]
impl ChessBoardExtsDebug for chess::Board {
    fn to_pretty_string(&self) -> String {
        Into::<chess::BoardBuilder>::into(self).to_pretty_string()
    }
}

#[cfg(debug_assertions)]
impl ChessBoardExtsDebug for chess::BoardBuilder {
    fn to_pretty_string(&self) -> String {
        let output = String::with_capacity(127);
        chess::ALL_SQUARES.chunks(8).rev().flatten().copied().fold(output, |mut acc, sq| {
            let sq_str = self[sq].map(|(p, c)| p.to_string(c)).unwrap_or_else(|| '.'.into());

            if !acc.is_empty() && sq.to_int().is_multiple_of(8) {
                acc.push('\n');
            }

            acc.push_str(&sq_str);

            if !(sq.to_int() + 1).is_multiple_of(8) {
                acc.push(' ');
            }

            acc
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_board_state(fen: &str) -> BoardState {
        let mut world = World::new();
        world.insert_resource(CliArgs { fen: Some(fen.into()) });
        world.init_resource::<BoardState>();
        world.remove_resource().unwrap()
    }

    #[test]
    fn move_is_en_passant() {
        let mut board_state = build_board_state("k7/3p4/8/2P5/8/8/8/K7 b - - 0 1");
        board_state.make_board_move(Square::D7, Square::D5, None);
        assert_eq!(board_state.en_passant(), Some(Square::D5));
        assert!(board_state.move_is_en_passant(PieceColor::WHITE, Square::D6));
    }

    #[test]
    fn move_isnt_en_passant() {
        let mut board_state = build_board_state("k7/3p4/8/4N3/8/8/8/K7 b - - 0 1");
        board_state.make_board_move(Square::D7, Square::D5, None);
        assert!(!board_state.move_is_en_passant(PieceColor::WHITE, Square::D6));
    }

    #[test]
    fn move_isnt_en_passant_when_there_is_no_ep() {
        let board_state = build_board_state("k7/8/8/8/8/5N2/8/K7 w - - 0 1");
        assert!(!board_state.move_is_en_passant(PieceColor::WHITE, Square::G1));
    }
}
