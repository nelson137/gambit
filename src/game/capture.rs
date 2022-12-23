use std::ops::{Index, IndexMut};

use bevy::prelude::*;

use crate::data::{BoardPiece, PieceColor, PieceType};

#[derive(Default, Resource)]
pub struct CaptureState(Captures);

type Captures = [CaptureSet; 2];

type CaptureSet = [u8; 5];

impl Index<PieceColor> for Captures {
    type Output = CaptureSet;

    fn index(&self, index: PieceColor) -> &Self::Output {
        self.index(*index as usize)
    }
}

impl IndexMut<PieceColor> for Captures {
    fn index_mut(&mut self, index: PieceColor) -> &mut Self::Output {
        self.index_mut(*index as usize)
    }
}

impl Index<PieceType> for CaptureSet {
    type Output = u8;

    fn index(&self, index: PieceType) -> &Self::Output {
        self.index(*index as usize)
    }
}

impl IndexMut<PieceType> for CaptureSet {
    fn index_mut(&mut self, index: PieceType) -> &mut Self::Output {
        self.index_mut(*index as usize)
    }
}

impl CaptureState {
    pub fn update(&mut self, BoardPiece { color, typ, .. }: BoardPiece) {
        self.0[color][typ] += 1;
    }
}

#[derive(Clone, Copy, Component, Deref, DerefMut)]
pub struct Captured(pub BoardPiece);

pub fn capture_piece(
    mut captures_state: ResMut<CaptureState>,
    mut q_captured: Query<(&mut Visibility, &Captured), Added<Captured>>,
) {
    if captures_state.is_changed() {
        warn!("TODO: update captured pieces display"); // TODO
    }

    for (mut vis, capture) in &mut q_captured {
        vis.is_visible = false;
        captures_state.update(**capture);
    }
}
