use std::{fmt, hash::Hash};

use bevy::prelude::*;
use chess::{File, Rank, Square};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct BoardLocation(pub Square);

impl BoardLocation {
    pub fn file_to_char(file: File) -> char {
        (b'a' + file.to_index() as u8) as char
    }

    pub fn rank_to_char(rank: Rank) -> char {
        (b'0' + rank.to_index() as u8 + 1) as char
    }

    pub fn new(square: Square) -> Self {
        Self(square)
    }

    pub fn file(&self) -> File {
        self.0.get_file()
    }

    pub fn file_char(&self) -> char {
        Self::file_to_char(self.file())
    }

    pub fn rank(&self) -> Rank {
        self.0.get_rank()
    }

    pub fn rank_char(&self) -> char {
        Self::rank_to_char(self.rank())
    }

    pub fn move_to(&mut self, square: Square) {
        self.0 = square;
    }
}

impl fmt::Display for BoardLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{}", **self))
    }
}
