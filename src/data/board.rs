use std::collections::{hash_map::Entry, HashMap};

use bevy::prelude::*;
use chess::{BitBoard, Board, ChessMove, MoveGen, Piece, Square, EMPTY};

#[derive(Component)]
pub struct Ui;

#[derive(Component)]
pub struct UiBoard;

/// The color used to highlight tiles.
pub const COLOR_HIGHLIGHT: Color = Color::rgba(1.0, 1.0, 0.0, 0.5);

#[derive(Component)]
pub struct Tile;

#[derive(Component)]
pub struct HighlightTile;

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

pub const BOARD_TEXT_FONT_SIZE: f32 = 20.0;

#[derive(Default, Deref, DerefMut, Resource)]
pub struct ShowingMovesFor(pub Option<Square>);

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct ShowHint;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct HideHint;

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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deref, DerefMut)]
pub struct PieceColor(pub chess::Color);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deref, DerefMut)]
pub struct PieceType(pub Piece);

#[derive(Clone, Copy, Debug)]
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

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct ShowHighlight;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct HideHighlight;

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
    last_shown_hints: Vec<Entity>,
}

impl Default for BoardState {
    fn default() -> Self {
        Self {
            tiles: HashMap::with_capacity(64),
            pieces: HashMap::with_capacity(32),
            highlights: HashMap::with_capacity(64),
            move_hints: HashMap::with_capacity(64),
            board: Board::default(),
            last_shown_hints: Vec::with_capacity(MAX_POSSIBLE_MOVES),
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

    pub fn show_piece_move_hints(&mut self, commands: &mut Commands, source: Square) {
        self.last_shown_hints.clear();

        let mut moves = MoveGen::new_legal(&self.board);

        let side_to_move_mask = self.board.color_combined(!self.board.side_to_move());
        moves.set_iterator_mask(*side_to_move_mask);
        for r#move in &mut moves {
            if r#move.get_source() != source {
                continue;
            }
            let entity = self.move_hints(r#move.get_dest()).capture_entity;
            commands.entity(entity).insert(ShowHint);
            self.last_shown_hints.push(entity);
        }

        moves.set_iterator_mask(!EMPTY);
        for r#move in &mut moves {
            if r#move.get_source() != source {
                continue;
            }
            let entity = self.move_hints(r#move.get_dest()).move_entity;
            commands.entity(entity).insert(ShowHint);
            self.last_shown_hints.push(entity);
        }
    }

    pub fn hide_piece_move_hints(&mut self, commands: &mut Commands) {
        for entity in self.last_shown_hints.drain(..) {
            commands.entity(entity).insert(HideHint);
        }
    }

    pub fn move_is_valid(&self, source: Square, dest: Square) -> bool {
        let mut move_gen = MoveGen::new_legal(&self.board);
        // Mask the generator to only gen moves (by any piece) to the destination.
        move_gen.set_iterator_mask(BitBoard::from_square(dest));
        // Return whether any of the generated moves are from the source.
        move_gen.any(|m| m.get_source() == source)
    }

    pub fn move_piece(&mut self, from: Square, to: Square) -> Option<BoardPiece> {
        self.board = self.board.make_move_new(ChessMove::new(from, to, None));
        let (_old_square, piece) = self
            .pieces
            .remove_entry(&from)
            .expect("Failed to move board state piece: no piece found at source square");
        match self.pieces.entry(to) {
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
        }
    }
}
