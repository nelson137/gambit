use std::{fmt, ops::Not};

use bevy::{ecs::system::Command, prelude::*};
use chess::{Piece, Rank};

use crate::{assets::PieceColorAndTypeAssetPath, debug_name_f, game::consts::Z_PIECE};

use super::{square::Square, BoardState};

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
}

pub fn spawn_pieces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut board_state: ResMut<BoardState>,
) {
    let pos_top_left = UiRect { top: Val::Px(0.0), left: Val::Px(0.0), ..default() };

    for square in chess::ALL_SQUARES.map(Square::new) {
        let Some(info) = board_state.get_piece_info_on(square) else { continue };
        let image_path = info.asset_path();
        let (piece_color, piece_type) = info;

        let piece_entity = commands
            .spawn((
                UiPiece::new(piece_color, piece_type),
                debug_name_f!("Piece ({piece_color} {piece_type}) ({square})"),
                square,
                ImageBundle {
                    image: UiImage(asset_server.load(image_path)),
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: pos_top_left,
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        ..default()
                    },
                    z_index: ZIndex::Local(Z_PIECE),
                    ..default()
                },
            ))
            .id();

        commands.entity(board_state.tile(square)).add_child(piece_entity);
        board_state.set_piece(square, piece_entity);
    }
}

pub struct PromoteUiPiece {
    entity: Entity,
    color: PieceColor,
    typ: PieceType,
}

impl PromoteUiPiece {
    pub fn new(entity: Entity, color: PieceColor, typ: PieceType) -> Self {
        Self { entity, color, typ }
    }
}

impl Command for PromoteUiPiece {
    fn write(self, world: &mut World) {
        let new_asset_path = (self.color, self.typ).asset_path();
        let new_asset = world.resource_mut::<AssetServer>().load(new_asset_path);

        let mut e = world.entity_mut(self.entity);
        if let Some(mut image) = e.get_mut::<UiImage>() {
            image.0 = new_asset;
        }
    }
}
