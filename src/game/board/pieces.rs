use std::{fmt, ops::Not};

use bevy::prelude::*;
use chess::{Piece, Rank};

use crate::{debug_name_f, game::consts::Z_PIECE};

use super::{square::Square, BoardState};

macro_rules! asset_path {
    ($color:literal, $type:literal) => {
        concat!("images/pieces/", $color, "-", $type, ".png")
    };
}

#[derive(Clone, Copy, Component)]
pub struct PieceMeta {
    pub color: PieceColor,
    pub typ: PieceType,
}

impl PieceMeta {
    pub fn new(color: PieceColor, typ: PieceType) -> Self {
        Self { color, typ }
    }

    pub fn asset_path(self) -> &'static str {
        match (self.color, self.typ) {
            (PieceColor::BLACK, PieceType::BISHOP) => asset_path!("black", "bishop"),
            (PieceColor::BLACK, PieceType::KING) => asset_path!("black", "king"),
            (PieceColor::BLACK, PieceType::KNIGHT) => asset_path!("black", "knight"),
            (PieceColor::BLACK, PieceType::PAWN) => asset_path!("black", "pawn"),
            (PieceColor::BLACK, PieceType::QUEEN) => asset_path!("black", "queen"),
            (PieceColor::BLACK, PieceType::ROOK) => asset_path!("black", "rook"),
            (PieceColor::WHITE, PieceType::BISHOP) => asset_path!("white", "bishop"),
            (PieceColor::WHITE, PieceType::KING) => asset_path!("white", "king"),
            (PieceColor::WHITE, PieceType::KNIGHT) => asset_path!("white", "knight"),
            (PieceColor::WHITE, PieceType::PAWN) => asset_path!("white", "pawn"),
            (PieceColor::WHITE, PieceType::QUEEN) => asset_path!("white", "queen"),
            (PieceColor::WHITE, PieceType::ROOK) => asset_path!("white", "rook"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PieceColor(pub chess::Color);

impl From<chess::Color> for PieceColor {
    fn from(color: chess::Color) -> Self {
        Self(color)
    }
}

impl From<PieceColor> for chess::Color {
    fn from(color: PieceColor) -> Self {
        color.0
    }
}

impl Not for PieceColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(self.0.not())
    }
}

impl PartialEq<chess::Color> for PieceColor {
    fn eq(&self, other: &chess::Color) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<PieceColor> for chess::Color {
    fn eq(&self, other: &PieceColor) -> bool {
        self.eq(&other.0)
    }
}

impl fmt::Debug for PieceColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for PieceColor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl PieceColor {
    pub const BLACK: Self = Self(chess::Color::Black);
    pub const WHITE: Self = Self(chess::Color::White);

    pub fn to_my_backrank(self) -> Rank {
        self.0.to_my_backrank()
    }

    pub fn to_their_backrank(self) -> Rank {
        self.0.to_their_backrank()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PieceType(pub Piece);

impl From<chess::Piece> for PieceType {
    fn from(typ: chess::Piece) -> Self {
        Self(typ)
    }
}

impl From<PieceType> for chess::Piece {
    fn from(typ: PieceType) -> Self {
        typ.0
    }
}

impl PartialEq<chess::Piece> for PieceType {
    fn eq(&self, other: &chess::Piece) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<PieceType> for chess::Piece {
    fn eq(&self, other: &PieceType) -> bool {
        self.eq(&other.0)
    }
}

impl fmt::Debug for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl PieceType {
    pub const PAWN: Self = Self(chess::Piece::Pawn);
    pub const BISHOP: Self = Self(chess::Piece::Bishop);
    pub const KNIGHT: Self = Self(chess::Piece::Knight);
    pub const ROOK: Self = Self(chess::Piece::Rook);
    pub const QUEEN: Self = Self(chess::Piece::Queen);
    pub const KING: Self = Self(chess::Piece::King);

    pub fn num_pieces(self) -> u8 {
        match self {
            Self::PAWN => 8,
            Self::KNIGHT | Self::BISHOP | Self::ROOK => 2,
            Self::QUEEN | Self::KING => 1,
        }
    }

    pub fn value(self) -> u8 {
        match self {
            Self::PAWN => 1,
            Self::KNIGHT | Self::BISHOP => 3,
            Self::ROOK => 5,
            Self::QUEEN => 9,
            Self::KING => panic!("King has no value as it cannot be captured"),
        }
    }
}

pub fn spawn_pieces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut board_state: ResMut<BoardState>,
) {
    for square in chess::ALL_SQUARES.map(Square::new) {
        let Some(info) = board_state.get_piece_meta(square) else { continue };
        let image_path = info.asset_path();

        let piece_entity = commands
            .spawn((
                info,
                debug_name_f!("Piece ({} {}) ({square})", info.color, info.typ),
                square,
                ImageBundle {
                    image: UiImage::new(asset_server.load(image_path)),
                    style: Style {
                        position_type: PositionType::Absolute,
                        top: Val::Px(0.0),
                        left: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    z_index: ZIndex::Global(Z_PIECE),
                    ..default()
                },
            ))
            .id();

        commands.entity(board_state.tile(square)).add_child(piece_entity);
        board_state.set_piece(square, piece_entity);
    }
}
