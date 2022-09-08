use core::fmt;
use std::{
    collections::{hash_map::Entry, HashMap},
    hash::{Hash, Hasher},
};

use bevy::prelude::*;

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

#[derive(Component)]
pub struct Piece;

#[derive(Component, Clone, Copy, Debug)]
pub struct Location {
    file: u8,
    rank: u8,
    pub z: f32,
    file_char: char,
    rank_char: char,
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
        Self {
            file,
            rank,
            z,
            file_char: Self::file_to_char(file),
            rank_char: Self::rank_to_char(rank),
            snap: true,
        }
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
        self.file_char
    }

    pub fn rank_char(&self) -> char {
        self.rank_char
    }

    pub fn move_to(&mut self, location: Location) {
        self.file = location.file;
        self.rank = location.rank;
        self.file_char = Self::file_to_char(location.file);
        self.rank_char = Self::rank_to_char(location.rank);
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
        self.file_char.hash(state);
        self.rank_char.hash(state);
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{}{}", self.file_char, self.rank_char))
    }
}

#[derive(Debug)]
pub struct BoardPiece;

#[derive(Default)]
pub struct BoardState {
    pub pieces: HashMap<Location, BoardPiece>,
}

impl BoardState {
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
