use std::{fmt, hash::Hash};

use bevy::prelude::*;
use chess::{File, Rank};

use super::PieceColor;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Square(pub chess::Square);

impl Square {
    pub const DEFAULT: Self = Self(chess::Square::A1);

    pub fn file_to_char(file: File) -> char {
        (b'a' + file.to_index() as u8) as char
    }

    pub fn rank_to_char(rank: Rank) -> char {
        (b'0' + rank.to_index() as u8 + 1) as char
    }

    pub const fn new(square: chess::Square) -> Self {
        Self(square)
    }

    pub fn from_coords(rank: Rank, file: File) -> Self {
        Self::new(chess::Square::make_square(rank, file))
    }

    pub fn get_file(self) -> File {
        self.0.get_file()
    }

    pub fn file_char(self) -> char {
        Self::file_to_char(self.get_file())
    }

    pub fn get_rank(self) -> Rank {
        self.0.get_rank()
    }

    pub fn rank_char(self) -> char {
        Self::rank_to_char(self.get_rank())
    }

    pub fn backward(self, color: PieceColor) -> Option<Self> {
        self.0.backward(color.0).map(Self::new)
    }

    pub fn move_to(&mut self, other: Self) {
        self.0 = other.0;
    }
}

impl From<chess::Square> for Square {
    fn from(square: chess::Square) -> Self {
        Self(square)
    }
}

impl PartialEq<chess::Square> for Square {
    fn eq(&self, other: &chess::Square) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<Square> for chess::Square {
    fn eq(&self, other: &Square) -> bool {
        self.eq(&other.0)
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
