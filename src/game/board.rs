use std::{
    collections::{hash_map::Entry, HashMap},
    ops::Not,
};

use bevy::{ecs::system::Command, prelude::*};
use chess::{BitBoard, Board, ChessMove, File, MoveGen, Piece, Square, EMPTY};

use super::{
    audio::PlayGameAudio,
    captures::Captured,
    moves::{DoMove, MoveUiPiece},
    utils::GameCommandList,
};

// ======================================================================
// Tile
// ======================================================================

#[derive(Component)]
pub struct Tile;

/// The "black" bord color.
///
/// `#769656`
pub const COLOR_BLACK: Color = Color::rgb(
    0x76 as f32 / u8::MAX as f32,
    0x96 as f32 / u8::MAX as f32,
    0x56 as f32 / u8::MAX as f32,
);

/// The "white" bord color.
///
/// `#eeeed2`
pub const COLOR_WHITE: Color = Color::rgb(
    0xee as f32 / u8::MAX as f32,
    0xee as f32 / u8::MAX as f32,
    0xd2 as f32 / u8::MAX as f32,
);

// ======================================================================
// Hightlight Tile
// ======================================================================

#[derive(Component)]
pub struct HighlightTile;

/// The color used to highlight tiles.
pub const COLOR_HIGHLIGHT: Color = Color::rgba(1.0, 1.0, 0.0, 0.5);

#[derive(Deref, DerefMut)]
pub struct ShowHighlight(pub Entity);

impl Command for ShowHighlight {
    fn write(self, world: &mut World) {
        if let Some(mut vis) = world.entity_mut(*self).get_mut::<Visibility>() {
            vis.is_visible = true;
        }
    }
}

#[derive(Deref, DerefMut)]
pub struct HideHighlight(pub Entity);

impl Command for HideHighlight {
    fn write(self, world: &mut World) {
        if let Some(mut vis) = world.entity_mut(*self).get_mut::<Visibility>() {
            vis.is_visible = false;
        }
    }
}

// ======================================================================
// Move Hint & Capture Hint
// ======================================================================

#[derive(Default)]
pub struct ShowHints(Vec<Entity>);

impl Command for ShowHints {
    fn write(self, world: &mut World) {
        for entity in self.0 {
            if let Some(mut vis) = world.entity_mut(entity).get_mut::<Visibility>() {
                vis.is_visible = true;
            }
        }
    }
}

#[derive(Default)]
pub struct HideHints(Vec<Entity>);

impl Command for HideHints {
    fn write(self, world: &mut World) {
        for entity in self.0 {
            if let Some(mut vis) = world.entity_mut(entity).get_mut::<Visibility>() {
                vis.is_visible = false;
            }
        }
    }
}

// ======================================================================
// Piece
// ======================================================================

#[derive(Component)]
pub struct UiPiece {
    pub color: PieceColor,
    pub typ: PieceType,
}

impl UiPiece {
    pub fn new(color: PieceColor, typ: PieceType) -> Self {
        Self { color, typ }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
pub struct PieceColor(pub chess::Color);

impl Not for PieceColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(self.0.not())
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Display for PieceColor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl PieceColor {
    pub const BLACK: Self = Self(chess::Color::Black);
    pub const WHITE: Self = Self(chess::Color::White);
}

#[derive(Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
pub struct PieceType(pub Piece);

#[cfg(debug_assertions)]
impl std::fmt::Display for PieceType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl PieceType {
    pub const PAWN: Self = Self(chess::Piece::Pawn);
    pub const BISHOP: Self = Self(chess::Piece::Bishop);
    pub const KNIGHT: Self = Self(chess::Piece::Knight);
    pub const ROOK: Self = Self(chess::Piece::Rook);
    pub const QUEEN: Self = Self(chess::Piece::Queen);
}

// ======================================================================
// Board State
// ======================================================================

#[derive(Clone, Copy)]
pub struct BoardPiece {
    pub entity: Entity,
    pub color: PieceColor,
    pub typ: PieceType,
}

impl BoardPiece {
    pub fn new(entity: Entity, color: PieceColor, typ: PieceType) -> Self {
        Self { entity, color, typ }
    }
}

#[derive(Debug)]
pub struct MoveHints {
    pub move_entity: Entity,
    pub capture_entity: Entity,
}

/// The maximum possible valid moves that any piece could ever have in a game: 27.
///
/// This is the number of valid moves that a queen can make when on one of the four middle squares
/// of the board (d4, e4, d5, or e5), with no other pieces blocking any of the eight rays of
/// possible movement.
///
/// Below is a diagram showing one of such configurations. When a queen is on, e.g., d4, she can
/// move to any of the squares that contain an `x`.
///
/// ```
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
    pieces: HashMap<Square, BoardPiece>,
    highlights: HashMap<Square, Entity>,
    move_hints: HashMap<Square, MoveHints>,
    board: Board,
    showing_hints: Vec<Entity>,
}

impl Default for BoardState {
    fn default() -> Self {
        Self {
            tiles: HashMap::with_capacity(64),
            pieces: HashMap::with_capacity(32),
            highlights: HashMap::with_capacity(64),
            move_hints: HashMap::with_capacity(64),
            board: Board::default(),
            showing_hints: Vec::with_capacity(MAX_POSSIBLE_MOVES),
        }
    }
}

impl BoardState {
    pub fn is_colors_turn_at(&self, square: Square) -> bool {
        self.board.side_to_move() == *self.piece(square).color
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

    pub fn piece(&self, square: Square) -> BoardPiece {
        self.get_piece(square).unwrap_or_else(|| panic!("no piece at {square}"))
    }

    pub fn get_piece(&self, square: Square) -> Option<BoardPiece> {
        self.pieces.get(&square).copied()
    }

    pub fn set_piece(&mut self, square: Square, piece: BoardPiece) {
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

        let mut was_castle = false;

        if *piece.typ == chess::Piece::King {
            let castle_rights = self.board.my_castle_rights();
            let back_rank = piece.color.to_my_backrank();
            let kingside_sq = Square::make_square(back_rank, File::G);
            let queenside_sq = Square::make_square(back_rank, File::C);

            // Move UI rook
            if castle_rights.has_kingside() && to_sq == kingside_sq {
                let piece = self.piece(Square::make_square(back_rank, File::H));
                let to_sq = Square::make_square(back_rank, File::F);
                cmd_list.add(MoveUiPiece { piece, to_sq });
                was_castle = true;
            } else if castle_rights.has_queenside() && to_sq == queenside_sq {
                let piece = self.piece(Square::make_square(back_rank, File::A));
                let to_sq = Square::make_square(back_rank, File::D);
                cmd_list.add(MoveUiPiece { piece, to_sq });
                was_castle = true;
            }
        }

        // Move UI piece
        cmd_list.add(MoveUiPiece { piece, to_sq });

        // Make move on board
        self.board = self.board.make_move_new(ChessMove::new(from_sq, to_sq, None));

        // Update pieces map
        let (_old_square, piece) = self
            .pieces
            .remove_entry(&from_sq)
            .expect("Failed to move board state piece: no piece found at source square");
        let captured_piece = match self.pieces.entry(to_sq) {
            // Move is a capture
            Entry::Occupied(mut entry) => {
                let value = entry.get_mut();
                let old_piece = *value;
                *value = piece;
                Some(old_piece)
            }
            // Move is just a move
            Entry::Vacant(entry) => {
                entry.insert(piece);
                None
            }
        };

        // Play audio
        if let Some(piece) = captured_piece {
            cmd_list.add(Captured(piece));
            cmd_list.add(PlayGameAudio::Capture);
        } else if was_castle {
            cmd_list.add(PlayGameAudio::Castle);
        } else {
            cmd_list.add(match *piece.color {
                chess::Color::Black => PlayGameAudio::MoveOpponent,
                chess::Color::White => PlayGameAudio::MoveSelf,
            });
        }

        cmd_list
    }
}
