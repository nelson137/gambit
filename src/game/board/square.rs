use std::{fmt, hash::Hash, str::FromStr};

use bevy::prelude::*;
use chess::{File, Rank};

use super::PieceColor;

#[derive(Component, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Square(pub chess::Square);

macro_rules! define_square_consts {
    ($($labels:ident),+ $(,)?) => {
        $(pub const $labels: Self = Self(chess::Square::$labels);)+
    }
}

impl Square {
    define_square_consts! {
        A1, B1, C1, D1, E1, F1, G1, H1,
        A2, B2, C2, D2, E2, F2, G2, H2,
        A3, B3, C3, D3, E3, F3, G3, H3,
        A4, B4, C4, D4, E4, F4, G4, H4,
        A5, B5, C5, D5, E5, F5, G5, H5,
        A6, B6, C6, D6, E6, F6, G6, H6,
        A7, B7, C7, D7, E7, F7, G7, H7,
        A8, B8, C8, D8, E8, F8, G8, H8,
    }
}

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

    pub fn forward(self, color: PieceColor) -> Option<Self> {
        self.0.forward(color.0).map(Self)
    }

    pub fn backward(self, color: PieceColor) -> Option<Self> {
        self.0.backward(color.0).map(Self)
    }
}

impl From<chess::Square> for Square {
    fn from(square: chess::Square) -> Self {
        Self(square)
    }
}

impl FromStr for Square {
    type Err = chess::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(chess::Square::from_str(s)?))
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

impl fmt::Debug for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Square({})", self.0)
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
