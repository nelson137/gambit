use std::ops::Not;

use bevy::prelude::*;
use chess::Piece;

use crate::{assets::PieceColorAndTypeAssetPath, debug_name, game::consts::Z_PIECE};

use super::{location::BoardLocation, BoardState};

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

#[derive(Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
pub struct PieceColor(pub chess::Color);

impl Not for PieceColor {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(self.0.not())
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Display for PieceColor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl PieceColor {
    pub const BLACK: Self = Self(chess::Color::Black);
    pub const WHITE: Self = Self(chess::Color::White);
}

#[derive(Clone, Copy, PartialEq, Eq, Deref, DerefMut)]
pub struct PieceType(pub Piece);

#[cfg(debug_assertions)]
impl std::fmt::Display for PieceType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl PieceType {
    pub const PAWN: Self = Self(chess::Piece::Pawn);
    pub const BISHOP: Self = Self(chess::Piece::Bishop);
    pub const KNIGHT: Self = Self(chess::Piece::Knight);
    pub const ROOK: Self = Self(chess::Piece::Rook);
    pub const QUEEN: Self = Self(chess::Piece::Queen);
}

pub fn spawn_pieces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut board_state: ResMut<BoardState>,
) {
    let pos_top_left = UiRect { top: Val::Px(0.0), left: Val::Px(0.0), ..default() };

    for square in chess::ALL_SQUARES {
        let Some(info) = board_state.get_piece_info_on(square) else { continue };
        let location = BoardLocation::new(square);

        let image_path = info.asset_path();
        let piece_color = PieceColor(info.0);
        let piece_type = PieceType(info.1);

        let piece_entity = commands
            .spawn((
                UiPiece::new(piece_color, piece_type),
                debug_name!("Piece ({piece_color} {piece_type}) ({square})"),
                location,
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
