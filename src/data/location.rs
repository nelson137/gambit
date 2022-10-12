use core::{
    fmt,
    hash::{Hash, Hasher},
};

use bevy::prelude::*;
use chess::{File, Rank, Square};

#[derive(Component, Clone, Copy, Debug)]
pub struct Location {
    square: Square,
    pub z: f32,
    pub snap: bool,
}

impl Location {
    pub fn file_to_char(file: File) -> char {
        (b'a' + file.to_index() as u8) as char
    }

    pub fn rank_to_char(rank: Rank) -> char {
        (b'0' + rank.to_index() as u8 + 1) as char
    }

    pub fn new_with_z(square: Square, z: f32) -> Self {
        Self { square, z, snap: true }
    }

    pub fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    pub fn square(&self) -> Square {
        self.square
    }

    pub fn file(&self) -> File {
        self.square.get_file()
    }

    pub fn file_char(&self) -> char {
        Self::file_to_char(self.file())
    }

    pub fn file_index(&self) -> usize {
        self.file().to_index()
    }

    pub fn rank(&self) -> Rank {
        self.square.get_rank()
    }

    pub fn rank_char(&self) -> char {
        Self::rank_to_char(self.rank())
    }

    pub fn rank_index(&self) -> usize {
        self.rank().to_index()
    }

    pub fn move_to(&mut self, square: Square) {
        self.square = square;
    }
}

impl PartialEq for Location {
    fn eq(&self, other: &Self) -> bool {
        self.square.eq(&other.square)
    }
}

impl Eq for Location {}

impl Hash for Location {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.square.hash(state);
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.square))
    }
}

impl From<Square> for Location {
    fn from(square: Square) -> Self {
        Self { square, z: 0.0, snap: true }
    }
}
