use core::fmt;
use std::{
    collections::{hash_map::Entry, HashMap},
    hash::{Hash, Hasher},
};

use bevy::prelude::*;

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
pub struct ShowHint;

#[derive(Component)]
pub struct HideHint;

#[derive(Component)]
pub struct Piece;

#[derive(Clone, Copy, Component, PartialEq, Eq, Debug)]
pub enum PieceColor {
    Black,
    White,
}

#[derive(Clone, Copy, Component, Debug)]
pub enum PieceType {
    Bishop,
    King,
    Knight,
    Pawn,
    Queen,
    Rook,
}

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

    pub const fn new(file: u8, rank: u8, z: f32) -> Self {
        Self { file, rank, z, snap: true }
    }

    pub fn with_rank(mut self, rank: u8) -> Self {
        self.rank = rank;
        self
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

    pub fn try_offset(&self, file_offset: i8, rank_offset: i8) -> Option<Location> {
        let file = self.file as i8 + file_offset;
        let rank = self.rank as i8 + rank_offset;
        if 0 <= file && file <= 7 && 0 <= rank && rank <= 7 {
            Some(Self { file: file as u8, rank: rank as u8, z: self.z, snap: self.snap })
        } else {
            None
        }
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

#[derive(Debug)]
pub struct BoardPiece {
    pub color: PieceColor,
    pub typ: PieceType,
}

#[derive(Debug)]
pub struct MoveHints {
    pub entity_move: Entity,
    pub entity_capture: Entity,
}

pub enum PieceMoveType {
    Move,
    Capture,
}

pub struct BoardState {
    pub pieces: HashMap<Location, BoardPiece>,
    pub move_hints: HashMap<Location, MoveHints>,
    piece_moves_cache: HashMap<Location, Vec<(Location, PieceMoveType)>>,
}

impl Default for BoardState {
    fn default() -> Self {
        Self {
            pieces: HashMap::with_capacity(32),
            move_hints: HashMap::with_capacity(64),
            piece_moves_cache: HashMap::with_capacity(64),
        }
    }
}

impl BoardState {
    fn get_hints(&self, location: &Location) -> &MoveHints {
        self.move_hints.get(location).expect("Failed to get hints: none at location")
    }

    fn possible_piece_moves(&self, location: Location) -> Vec<(Location, PieceMoveType)> {
        let &BoardPiece { color, typ } = self.pieces.get(&location).expect(
            "Failed to calculate possible moves for piece: no such piece exists at this location",
        );

        let mut moves = Vec::new();

        // Return whether a directional piece (pawn, rook, bishop, queen) could move past loc
        let mut push_directional_move = |loc: Location| match self.pieces.get(&loc) {
            Some(piece) if piece.color != color => {
                moves.push((loc, PieceMoveType::Capture));
                false
            }
            None => {
                moves.push((loc, PieceMoveType::Move));
                true
            }
            _ => false,
        };
        let mut push_directional_offset = |file_o, rank_o| {
            location.try_offset(file_o, rank_o).map(|l| push_directional_move(l)).unwrap_or(false)
        };

        match typ {
            PieceType::Pawn => {
                let (offset, start_rank): (i8, u8) = match color {
                    PieceColor::Black => (-1, 6),
                    PieceColor::White => (1, 1),
                };

                if let Some(loc1) = location.try_offset(0, offset as i8) {
                    if !self.pieces.contains_key(&loc1) {
                        moves.push((loc1, PieceMoveType::Move));
                        if location.rank == start_rank {
                            let loc2 = loc1.with_rank((loc1.rank as i8 + offset) as u8);
                            if !self.pieces.contains_key(&loc2) {
                                moves.push((loc2, PieceMoveType::Move));
                            }
                        }
                    }
                }

                let loc_capturable =
                    |loc| self.pieces.get(&loc).map(|p| p.color != color).unwrap_or(false);
                if let Some(loc) = location.try_offset(1, offset) {
                    if loc_capturable(loc) {
                        moves.push((loc, PieceMoveType::Capture));
                    }
                }
                if let Some(loc) = location.try_offset(-1, offset) {
                    if loc_capturable(loc) {
                        moves.push((loc, PieceMoveType::Capture));
                    }
                }
            }

            PieceType::Rook => {
                let (mut n, mut e, mut s, mut w) = (true, true, true, true);
                for i in 1..8 {
                    n = n && push_directional_offset(0, i);
                    e = e && push_directional_offset(i, 0);
                    s = s && push_directional_offset(0, -i);
                    w = w && push_directional_offset(-i, 0);
                }
            }

            PieceType::Knight => {
                const KNIGHT_MOVES: [(i8, i8); 8] =
                    [(1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1), (-2, 1), (-1, 2)];
                for (file_o, rank_o) in KNIGHT_MOVES {
                    push_directional_offset(file_o, rank_o);
                }
            }

            PieceType::Bishop => {
                let (mut ne, mut se, mut sw, mut nw) = (true, true, true, true);
                for o in 1..8 {
                    ne = ne && push_directional_offset(o, o);
                    se = se && push_directional_offset(o, -o);
                    sw = sw && push_directional_offset(-o, -o);
                    nw = nw && push_directional_offset(-o, o);
                }
            }

            PieceType::Queen => {
                let (mut n, mut ne, mut e, mut se, mut s, mut sw, mut w, mut nw) =
                    (true, true, true, true, true, true, true, true);
                for o in 1..8 {
                    n = n && push_directional_offset(0, o);
                    ne = ne && push_directional_offset(o, o);
                    e = e && push_directional_offset(o, 0);
                    se = se && push_directional_offset(o, -o);
                    s = s && push_directional_offset(0, -o);
                    sw = sw && push_directional_offset(-o, -o);
                    w = w && push_directional_offset(-o, 0);
                    nw = nw && push_directional_offset(-o, o);
                }
            }

            PieceType::King => {
                const KING_MOVES: [(i8, i8); 8] =
                    [(0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0), (-1, 1)];
                for (file_o, rank_o) in KING_MOVES {
                    push_directional_offset(file_o, rank_o);
                }
            }
        }

        moves
    }

    fn calculate_and_cache_piece_moves(&mut self, location: Location) {
        self.piece_moves_cache.insert(location, self.possible_piece_moves(location));
    }

    pub fn get_piece_moves(&self, location: &Location) -> &Vec<(Location, PieceMoveType)> {
        self.piece_moves_cache.get(location).expect("Failed to get possible moves: not cached")
    }

    pub fn show_piece_move_hints(&mut self, commands: &mut Commands, location: Location) {
        self.calculate_and_cache_piece_moves(location);
        for (loc, typ) in self.get_piece_moves(&location) {
            let loc_hints = self.get_hints(loc);
            let entity = match typ {
                PieceMoveType::Move => loc_hints.entity_move,
                PieceMoveType::Capture => loc_hints.entity_capture,
            };
            commands.entity(entity).insert(ShowHint);
        }
    }

    pub fn hide_piece_move_hints(&self, commands: &mut Commands, location: &Location) {
        for (loc, typ) in self.get_piece_moves(location) {
            let loc_hints = self.get_hints(loc);
            let entity = match typ {
                PieceMoveType::Capture => loc_hints.entity_capture,
                PieceMoveType::Move => loc_hints.entity_move,
            };
            commands.entity(entity).insert(HideHint);
        }
    }

    pub fn move_piece(&mut self, from: Location, to: Location) {
        let (_old_loc, piece) = self
            .pieces
            .remove_entry(&from)
            .expect("Failed to move board state piece: no piece found at source location");
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
    }
}
