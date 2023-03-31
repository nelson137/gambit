use std::{
    collections::{hash_map::Entry, HashMap},
    str::FromStr,
};

use bevy::{ecs::system::Command, prelude::*};
use chess::{BitBoard, Board, BoardStatus, ChessMove, File, MoveGen, EMPTY};

use crate::{
    cli::CliArgs,
    game::{
        audio::PlayGameAudio, board::StartPromotion, captures::Captured, game_over::GameOver,
        moves::MoveUiPiece, utils::GameCommandList,
    },
};

use super::{
    HideHighlight, HideHints, MoveHints, PieceColor, PieceType, ShowHighlight, ShowHints, Square,
};

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
    last_move_highlights: Option<(Entity, Entity)>,
    current_highlight: Option<Entity>,
    showing_hints: Vec<Entity>,
}

impl FromWorld for BoardState {
    fn from_world(world: &mut World) -> Self {
        let board = match &world.resource::<CliArgs>().fen {
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
            last_move_highlights: None,
            current_highlight: None,
            showing_hints: Vec::with_capacity(MAX_POSSIBLE_MOVES),
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

    pub fn side_to_move(&self) -> PieceColor {
        self.board.side_to_move().into()
    }

    pub fn is_colors_turn_at(&self, square: Square) -> bool {
        self.color_on(square) == self.board.side_to_move()
    }

    fn color_on(&self, square: Square) -> PieceColor {
        PieceColor(self.board.color_on(square.0).unwrap_or_else(|| panic!("no piece at {square}")))
    }

    fn piece_on(&self, square: Square) -> PieceType {
        PieceType(self.board.piece_on(square.0).unwrap_or_else(|| panic!("no piece at {square}")))
    }

    pub fn get_piece_info_on(&self, square: Square) -> Option<(PieceColor, PieceType)> {
        match (self.board.color_on(square.0), self.board.piece_on(square.0)) {
            (Some(color), Some(typ)) => Some((PieceColor(color), PieceType(typ))),
            _ => None,
        }
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
        self.clear_showing_hints();
        self.board = Board::default();
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

    pub fn move_hints(&self, square: Square) -> &MoveHints {
        self.move_hints.get(&square).unwrap_or_else(|| panic!("no move hints at {square}"))
    }

    pub fn set_move_hints(&mut self, square: Square, hints: MoveHints) {
        match self.move_hints.entry(square) {
            Entry::Occupied(_) => panic!("move hints already in the state at {square}"),
            Entry::Vacant(e) => e.insert(hints),
        };
    }

    pub fn clear_showing_hints(&mut self) {
        self.showing_hints.clear();
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
    // Select & unselect
    //------------------------------

    #[must_use]
    pub fn select_square(&mut self, square: Square) -> impl Command {
        let mut cmd_list = GameCommandList::default();
        cmd_list.add(self.show_highlight_tile(square));
        cmd_list.add(self.show_move_hints_for(square));
        cmd_list
    }

    #[must_use]
    pub fn unselect_square(&mut self) -> impl Command {
        let mut cmd_list = GameCommandList::default();
        match (self.last_move_highlights, self.current_highlight) {
            (Some((last_src, last_dest)), Some(current))
                if current == last_src || current == last_dest => {}
            _ => cmd_list.add(self.hide_highlight_tile()),
        }
        cmd_list.add(self.hide_move_hints());
        cmd_list
    }

    #[must_use]
    fn show_highlight_tile(&mut self, square: Square) -> impl Command {
        let entity = self.highlight(square);
        self.current_highlight = Some(entity);
        ShowHighlight(entity)
    }

    #[must_use]
    fn hide_highlight_tile(&mut self) -> impl Command {
        HideHighlight(self.current_highlight.take())
    }

    #[must_use]
    fn show_move_hints_for(&mut self, source: Square) -> impl Command {
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
            moves.push(self.move_hints(r#move.get_dest().into()).capture_entity);
        }

        move_gen.set_iterator_mask(!EMPTY);
        for r#move in &mut move_gen {
            if r#move.get_source() != source {
                continue;
            }
            moves.push(self.move_hints(r#move.get_dest().into()).move_entity);
        }

        if !moves.is_empty() {
            self.showing_hints.extend(&moves);
        }
        ShowHints(moves)
    }

    #[must_use]
    fn hide_move_hints(&mut self) -> impl Command {
        HideHints(if self.showing_hints.is_empty() {
            Vec::new()
        } else {
            self.showing_hints.drain(..).collect()
        })
    }

    pub fn hide_last_move_highlights(&mut self) -> impl Command {
        let mut cmd_list = GameCommandList::default();
        let last_mv_hl = self.last_move_highlights.take();
        cmd_list.add(HideHighlight(last_mv_hl.map(|(entity, _)| entity)));
        cmd_list.add(HideHighlight(last_mv_hl.map(|(_, entity)| entity)));
        cmd_list
    }

    //------------------------------
    // Move
    //------------------------------

    #[must_use]
    pub fn update_piece(
        &mut self,
        color: PieceColor,
        from_sq: Square,
        to_sq: Square,
    ) -> Option<impl Command> {
        let (_old_square, piece) = self.pieces.remove_entry(&from_sq).unwrap_or_else(|| {
            panic!("Failed to move board state piece: no piece found at source square {from_sq}")
        });

        let captured_piece = match self.en_passant() {
            // `chess::Board::en_passant` returns an optional square which is that of the piece that
            // can be captured in the en passant move that is currently available on the board.
            // The current move is this en passant if there is an en passant square and the
            // destination of the move is the square behind it (from the perspective of the *other*
            // player, hence the `!color`).
            Some(ep_sq) if ep_sq.backward(!color).map(|sq| sq == to_sq).unwrap_or(false) => {
                self.pieces.insert(to_sq, piece);
                self.pieces.remove(&ep_sq).map(|entity| (entity, ep_sq))
            }
            _ => match self.pieces.entry(to_sq) {
                // Capture
                Entry::Occupied(mut entry) => {
                    let value = entry.get_mut();
                    let old_piece = *value;
                    *value = piece;
                    Some((old_piece, to_sq))
                }
                // Move
                Entry::Vacant(entry) => {
                    entry.insert(piece);
                    None
                }
            },
        };

        captured_piece
            .map(|(entity, sq)| Captured::new(entity, self.color_on(sq), self.piece_on(sq)))
    }

    #[must_use]
    pub fn move_piece(&mut self, from_sq: Square, to_sq: Square) -> impl Command {
        let entity = self.piece(from_sq);
        let color = self.color_on(from_sq);
        let typ = self.piece_on(from_sq);

        if typ == PieceType::PAWN && to_sq.get_rank() == color.to_their_backrank() {
            self.start_promotion(entity, color, from_sq, to_sq)
        } else {
            self.move_piece_inner(entity, color, from_sq, to_sq, None)
        }
    }

    #[must_use]
    fn start_promotion(
        &mut self,
        entity: Entity,
        color: PieceColor,
        from_sq: Square,
        to_sq: Square,
    ) -> GameCommandList {
        let mut cmd_list = GameCommandList::default();
        cmd_list.add(self.unselect_square());
        cmd_list.add(StartPromotion { entity, color, from_sq, to_sq });
        cmd_list
    }

    #[must_use]
    pub fn move_piece_inner(
        &mut self,
        entity: Entity,
        color: PieceColor,
        from_sq: Square,
        to_sq: Square,
        promotion: Option<PieceType>,
    ) -> GameCommandList {
        let mut cmd_list = GameCommandList::default();

        let typ = self.piece_on(from_sq);

        cmd_list.add(self.unselect_square());

        // Move UI piece
        cmd_list.add(MoveUiPiece { entity, color, from_sq, to_sq });

        let mut is_castle = false;
        if typ == PieceType::KING {
            let castle_rights = self.board.my_castle_rights();
            let back_rank = color.to_my_backrank();
            let kingside_sq = Square::from_coords(back_rank, File::G);
            let queenside_sq = Square::from_coords(back_rank, File::C);

            // Move UI rook
            if castle_rights.has_kingside() && to_sq == kingside_sq {
                let from_sq = Square::from_coords(back_rank, File::H);
                let to_sq = Square::from_coords(back_rank, File::F);
                let entity = self.piece(from_sq);
                cmd_list.add(MoveUiPiece { entity, color, from_sq, to_sq });
                is_castle = true;
            } else if castle_rights.has_queenside() && to_sq == queenside_sq {
                let from_sq = Square::from_coords(back_rank, File::A);
                let to_sq = Square::from_coords(back_rank, File::D);
                let entity = self.piece(from_sq);
                cmd_list.add(MoveUiPiece { entity, color, from_sq, to_sq });
                is_castle = true;
            }
        }

        let is_capture = match self.en_passant() {
            // `chess::Board::en_passant` returns an optional square which is that of the piece that
            // can be captured in the en passant move that is currently available on the board.
            // The current move is this en passant if there is an en passant square and the
            // destination of the move is the square behind it (from the perspective of the *other*
            // player, hence the `!color`).
            Some(ep_sq) if ep_sq.backward(!color).map(|sq| sq == to_sq).unwrap_or(false) => true,
            _ => self.pieces.contains_key(&to_sq),
        };

        // Make move on board
        let r#move = ChessMove::new(from_sq.0, to_sq.0, promotion.map(|p| p.0));
        self.board = self.board.make_move_new(r#move);

        // Play audio
        if is_capture {
            cmd_list.add(PlayGameAudio::Capture);
        } else if is_castle {
            cmd_list.add(PlayGameAudio::Castle);
        } else {
            cmd_list.add(match color {
                PieceColor::BLACK => PlayGameAudio::MoveOpponent,
                PieceColor::WHITE => PlayGameAudio::MoveSelf,
            });
        }

        let hl_1 = self.highlight(from_sq);
        let hl_2 = self.highlight(to_sq);
        if let Some((prev_hl_1, prev_hl_2)) = self.last_move_highlights.replace((hl_1, hl_2)) {
            cmd_list.add(HideHighlight(Some(prev_hl_1)));
            cmd_list.add(HideHighlight(Some(prev_hl_2)));
        }
        cmd_list.add(ShowHighlight(hl_1));
        cmd_list.add(ShowHighlight(hl_2));

        if let BoardStatus::Checkmate | BoardStatus::Stalemate = self.board.status() {
            cmd_list.add(GameOver);
        }

        cmd_list
    }
}
