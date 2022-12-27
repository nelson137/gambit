use std::collections::{hash_map::Entry, HashMap};

use bevy::prelude::*;
use chess::{BitBoard, Board, ChessMove, MoveGen, Piece, Square, EMPTY};

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

pub const BOARD_TEXT_FONT_SIZE: f32 = 28.0;

const _BOARD_LOCATION_TEXT_OFFSET: f32 = 60.0;

pub const BOARD_FILE_TEXT_OFFSET_X: f32 = _BOARD_LOCATION_TEXT_OFFSET;
pub const BOARD_FILE_TEXT_OFFSET_Y: f32 = -BOARD_FILE_TEXT_OFFSET_X;

pub const BOARD_RANK_TEXT_OFFSET_X: f32 = -BOARD_RANK_TEXT_OFFSET_Y;
pub const BOARD_RANK_TEXT_OFFSET_Y: f32 = _BOARD_LOCATION_TEXT_OFFSET;

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

#[derive(Resource)]
pub struct BoardState {
    pub pieces: HashMap<Square, BoardPiece>,
    pub highlights: HashMap<Square, Entity>,
    pub move_hints: HashMap<Square, MoveHints>,
    pub move_gen_board: Board,
    last_shown_hints: Vec<Entity>,
}

impl Default for BoardState {
    fn default() -> Self {
        Self {
            pieces: HashMap::with_capacity(32),
            highlights: HashMap::with_capacity(64),
            move_hints: HashMap::with_capacity(64),
            move_gen_board: Board::default(),
            last_shown_hints: Vec::with_capacity(27),
        }
    }
}

impl BoardState {
    pub fn is_colors_turn_at(&self, square: Square) -> bool {
        let color = self.pieces.get(&square).expect("TODO").color;
        self.move_gen_board.side_to_move() == *color
    }

    fn get_hints(&self, square: Square) -> &MoveHints {
        self.move_hints.get(&square).expect("Failed to get hints: none at square")
    }

    pub fn show_piece_move_hints(&mut self, commands: &mut Commands, source: Square) {
        self.last_shown_hints.clear();

        let mut moves = MoveGen::new_legal(&self.move_gen_board);

        let side_to_move_mask =
            self.move_gen_board.color_combined(!self.move_gen_board.side_to_move());
        moves.set_iterator_mask(*side_to_move_mask);
        for r#move in &mut moves {
            if r#move.get_source() != source {
                continue;
            }
            let entity = self.get_hints(r#move.get_dest()).capture_entity;
            commands.entity(entity).insert(ShowHint);
            self.last_shown_hints.push(entity);
        }

        moves.set_iterator_mask(!EMPTY);
        for r#move in &mut moves {
            if r#move.get_source() != source {
                continue;
            }
            let entity = self.get_hints(r#move.get_dest()).move_entity;
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
        let mut move_gen = MoveGen::new_legal(&self.move_gen_board);
        // Mask the generator to only gen moves (by any piece) to the destination.
        move_gen.set_iterator_mask(BitBoard::from_square(dest));
        // Return whether any of the generated moves are from the source.
        move_gen.any(|m| m.get_source() == source)
    }

    pub fn move_piece(&mut self, from: Square, to: Square) -> Option<BoardPiece> {
        self.move_gen_board = self.move_gen_board.make_move_new(ChessMove::new(from, to, None));
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
