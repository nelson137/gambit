use core::fmt;
use std::{
    collections::{hash_map::Entry, HashMap},
    hash::{Hash, Hasher},
};

use bevy::prelude::*;
use chess::{ChessMove, File, MoveGen, Piece, Rank, Square, EMPTY};

pub const Z_PIECE_SELECTED: f32 = 1.5;

pub const Z_PIECE: f32 = 1.0;

pub const Z_HIGHLIGHT_TILE: f32 = 0.3;

pub const Z_MOVE_HINT: f32 = 0.2;

pub const Z_NOTATION_TEXT: f32 = 0.1;

pub const Z_TILE: f32 = 0.0;

#[derive(Component)]
pub struct Board;

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

#[derive(Default)]
pub struct ShowingMovesFor(pub Option<Location>);

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct ShowHint;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct HideHint;

#[derive(Component)]
pub struct UiPiece;

#[derive(Clone, Copy, Component, PartialEq, Eq, Debug)]
pub enum PieceColor {
    Black,
    White,
}

#[allow(clippy::from_over_into)]
impl Into<chess::Color> for PieceColor {
    fn into(self) -> chess::Color {
        match self {
            PieceColor::Black => chess::Color::Black,
            PieceColor::White => chess::Color::White,
        }
    }
}

#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
pub struct PieceType(pub Piece);

#[derive(Component, Clone, Copy, Debug)]
pub struct Location {
    file: u8,
    rank: u8,
    pub z: f32,
    pub snap: bool,
}

impl Location {
    pub const fn file_to_char(file: u8) -> char {
        (b'a' + file) as char
    }

    pub const fn rank_to_char(rank: u8) -> char {
        (b'0' + rank + 1) as char
    }

    pub const fn new_with_z(file: u8, rank: u8, z: f32) -> Self {
        Self { file, rank, z, snap: true }
    }

    pub const fn new(file: u8, rank: u8) -> Self {
        Self::new_with_z(file, rank, 0.0)
    }

    pub fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    pub fn file(&self) -> u8 {
        self.file
    }

    pub fn rank(&self) -> u8 {
        self.rank
    }

    pub fn file_char(&self) -> char {
        Self::file_to_char(self.file)
    }

    pub fn rank_char(&self) -> char {
        Self::rank_to_char(self.rank)
    }

    pub fn move_to(&mut self, location: Location) {
        self.file = location.file;
        self.rank = location.rank;
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        self.file == other.file && self.rank == other.rank
    }
}

impl Eq for Location {}

impl Hash for Location {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Self::file_to_char(self.file).hash(state);
        Self::rank_to_char(self.rank).hash(state);
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}{}",
            Self::file_to_char(self.file),
            Self::rank_to_char(self.rank)
        ))
    }
}

pub trait LocationToSquare {
    fn to_square(self) -> Square;
}
impl LocationToSquare for Location {
    fn to_square(self) -> Square {
        let rank = Rank::from_index(self.rank as usize);
        let file = File::from_index(self.file as usize);
        Square::make_square(rank, file)
    }
}

pub trait SquareToLocation {
    fn to_loc(self) -> Location;
}
impl SquareToLocation for Square {
    fn to_loc(self) -> Location {
        Location::new(self.get_file().to_index() as u8, self.get_rank().to_index() as u8)
    }
}

#[derive(Debug)]
pub struct BoardPiece {
    pub color: PieceColor,
    pub typ: PieceType,
    pub did_move: bool,
}

impl BoardPiece {
    pub fn new(color: PieceColor, typ: PieceType) -> Self {
        Self { color, typ, did_move: false }
    }
}

#[derive(Debug)]
pub struct MoveHints {
    pub entity_move: Entity,
    pub entity_capture: Entity,
}

pub struct BoardState {
    pub move_count: u32,
    pub pieces: HashMap<Location, BoardPiece>,
    pub move_hints: HashMap<Location, MoveHints>,
    pub move_gen_board: chess::Board,
    last_shown_hints: Vec<Entity>,
}

impl Default for BoardState {
    fn default() -> Self {
        Self {
            move_count: 0,
            pieces: HashMap::with_capacity(32),
            move_hints: HashMap::with_capacity(64),
            move_gen_board: chess::Board::default(),
            last_shown_hints: Vec::with_capacity(27),
        }
    }
}

impl BoardState {
    pub fn is_colors_turn_at(&self, location: Location) -> bool {
        let color = self.pieces.get(&location).expect("TODO").color;
        match color {
            PieceColor::Black => self.move_count % 2 == 1,
            PieceColor::White => self.move_count % 2 == 0,
        }
    }

    fn get_hints(&self, location: Location) -> &MoveHints {
        self.move_hints.get(&location).expect("Failed to get hints: none at location")
    }

    pub fn show_piece_move_hints(&mut self, commands: &mut Commands, source: Location) {
        self.last_shown_hints.clear();

        let source = source.to_square();

        let mut moves = MoveGen::new_legal(&self.move_gen_board);

        let side_to_move_mask =
            self.move_gen_board.color_combined(!self.move_gen_board.side_to_move());
        moves.set_iterator_mask(*side_to_move_mask);
        for r#move in &mut moves {
            if r#move.get_source() != source {
                continue;
            }
            let dest = r#move.get_dest();
            let entity = self.get_hints(dest.to_loc()).entity_capture;
            commands.entity(entity).insert(ShowHint);
            self.last_shown_hints.push(entity);
        }

        moves.set_iterator_mask(!EMPTY);
        for r#move in &mut moves {
            if r#move.get_source() != source {
                continue;
            }
            let dest = r#move.get_dest();
            let entity = self.get_hints(dest.to_loc()).entity_move;
            commands.entity(entity).insert(ShowHint);
            self.last_shown_hints.push(entity);
        }
    }

    pub fn hide_piece_move_hints(&mut self, commands: &mut Commands) {
        for entity in self.last_shown_hints.drain(..) {
            commands.entity(entity).insert(HideHint);
        }
    }

    pub fn move_piece(&mut self, from: Location, to: Location) {
        let (_old_loc, mut piece) = self
            .pieces
            .remove_entry(&from)
            .expect("Failed to move board state piece: no piece found at source location");
        piece.did_move = true;
        match self.pieces.entry(to) {
            Entry::Occupied(entry) => {
                panic!(
                    "Failed to move board state piece: piece already at destination location {}",
                    entry.key()
                )
            }
            Entry::Vacant(entry) => {
                entry.insert(piece);
            }
        }
        self.move_gen_board = self.move_gen_board.make_move_new(ChessMove::new(
            from.to_square(),
            to.to_square(),
            None,
        ));
    }
}
