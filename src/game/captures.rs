use std::{
    ops::{Index, IndexMut},
    sync::Arc,
};

use bevy::{ecs::system::Command, prelude::*};

use crate::game::board::{PieceColor, PieceType};

#[derive(Deref, DerefMut, Resource)]
pub struct CaptureState(Arc<PlayerCaptures<CapState>>);

impl FromWorld for CaptureState {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let mut captures = PlayerCaptures::<CapState>::default();

        captures[PieceColor::BLACK][PieceType::PAWN].image_handles.extend([
            asset_server.load("images/captures/white-pawns-8.png"),
            asset_server.load("images/captures/white-pawns-7.png"),
            asset_server.load("images/captures/white-pawns-6.png"),
            asset_server.load("images/captures/white-pawns-5.png"),
            asset_server.load("images/captures/white-pawns-4.png"),
            asset_server.load("images/captures/white-pawns-3.png"),
            asset_server.load("images/captures/white-pawns-2.png"),
            asset_server.load("images/captures/white-pawns-1.png"),
        ]);
        captures[PieceColor::BLACK][PieceType::BISHOP].image_handles.extend([
            asset_server.load("images/captures/white-bishops-2.png"),
            asset_server.load("images/captures/white-bishops-1.png"),
        ]);
        captures[PieceColor::BLACK][PieceType::KNIGHT].image_handles.extend([
            asset_server.load("images/captures/white-knights-2.png"),
            asset_server.load("images/captures/white-knights-1.png"),
        ]);
        captures[PieceColor::BLACK][PieceType::ROOK].image_handles.extend([
            asset_server.load("images/captures/white-rooks-2.png"),
            asset_server.load("images/captures/white-rooks-1.png"),
        ]);
        captures[PieceColor::BLACK][PieceType::QUEEN]
            .image_handles
            .push(asset_server.load("images/captures/white-queen.png"));

        captures[PieceColor::WHITE][PieceType::PAWN].image_handles.extend([
            asset_server.load("images/captures/black-pawns-8.png"),
            asset_server.load("images/captures/black-pawns-7.png"),
            asset_server.load("images/captures/black-pawns-6.png"),
            asset_server.load("images/captures/black-pawns-5.png"),
            asset_server.load("images/captures/black-pawns-4.png"),
            asset_server.load("images/captures/black-pawns-3.png"),
            asset_server.load("images/captures/black-pawns-2.png"),
            asset_server.load("images/captures/black-pawns-1.png"),
        ]);
        captures[PieceColor::WHITE][PieceType::BISHOP].image_handles.extend([
            asset_server.load("images/captures/black-bishops-2.png"),
            asset_server.load("images/captures/black-bishops-1.png"),
        ]);
        captures[PieceColor::WHITE][PieceType::KNIGHT].image_handles.extend([
            asset_server.load("images/captures/black-knights-2.png"),
            asset_server.load("images/captures/black-knights-1.png"),
        ]);
        captures[PieceColor::WHITE][PieceType::ROOK].image_handles.extend([
            asset_server.load("images/captures/black-rooks-2.png"),
            asset_server.load("images/captures/black-rooks-1.png"),
        ]);
        captures[PieceColor::WHITE][PieceType::QUEEN]
            .image_handles
            .push(asset_server.load("images/captures/black-queen.png"));

        Self(Arc::new(captures))
    }
}

impl CaptureState {
    #[cfg(debug_assertions)]
    #[allow(dead_code)]
    pub fn log_counts(&self) {
        fn log(color: &str, caps: &PieceCaptures<CapState>) {
            info!(
                pawns = caps[PieceType(chess::Piece::Pawn)].count,
                bishops = caps[PieceType(chess::Piece::Bishop)].count,
                knights = caps[PieceType(chess::Piece::Knight)].count,
                rooks = caps[PieceType(chess::Piece::Rook)].count,
                queens = caps[PieceType(chess::Piece::Queen)].count,
                "{color}"
            );
        }
        info!("");
        info!("================== Capture Counts ==================");
        log("White :", &self.0[PieceColor(chess::Color::White)]);
        log("Black :", &self.0[PieceColor(chess::Color::Black)]);
    }
}

pub struct CapState {
    pub image_handles: Vec<Handle<Image>>,
    pub image_entity: Entity,
    pub count: u8,
}

impl Default for CapState {
    fn default() -> Self {
        Self { image_handles: Vec::new(), image_entity: Entity::from_raw(u32::MAX), count: 0 }
    }
}

#[derive(Clone, Copy, Default, Deref, DerefMut)]
pub struct PlayerCaptures<C>(pub [PieceCaptures<C>; 2]);

impl<C> Index<PieceColor> for PlayerCaptures<C> {
    type Output = PieceCaptures<C>;

    fn index(&self, index: PieceColor) -> &Self::Output {
        self.0.index(*index as usize)
    }
}

impl<C> IndexMut<PieceColor> for PlayerCaptures<C> {
    fn index_mut(&mut self, index: PieceColor) -> &mut Self::Output {
        self.0.index_mut(*index as usize)
    }
}

#[derive(Clone, Copy, Default, Deref, DerefMut)]
pub struct PieceCaptures<C>(pub [C; 5]);

impl<C> Index<usize> for PieceCaptures<C> {
    type Output = C;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<C> IndexMut<usize> for PieceCaptures<C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<C> Index<PieceType> for PieceCaptures<C> {
    type Output = C;

    fn index(&self, index: PieceType) -> &Self::Output {
        self.0.index(*index as usize)
    }
}

impl<C> IndexMut<PieceType> for PieceCaptures<C> {
    fn index_mut(&mut self, index: PieceType) -> &mut Self::Output {
        self.0.index_mut(*index as usize)
    }
}

#[derive(Clone, Copy)]
pub struct Captured {
    entity: Entity,
    color: chess::Color,
    typ: chess::Piece,
}

impl Captured {
    pub fn new(entity: Entity, color: chess::Color, typ: chess::Piece) -> Self {
        Self { entity, color, typ }
    }
}

impl Command for Captured {
    fn write(self, world: &mut World) {
        // The count and capture image need to be updated for the player who performed the capture,
        // i.e. the one whose color is the opposite of that of the captured piece.
        let color = PieceColor(!self.color);
        let typ = PieceType(self.typ);

        // Hide piece
        if let Some(mut vis) = world.entity_mut(self.entity).get_mut::<Visibility>() {
            vis.is_visible = false;
        }

        let mut capture_state = world.resource_mut::<CaptureState>();
        let capture_state = Arc::get_mut(&mut capture_state).unwrap();
        let cap_state = &mut capture_state[color][typ];

        // Update count
        cap_state.count += 1;
        let count = cap_state.count;

        // Get the handle to the correct image for the updated count
        let index = cap_state.image_handles.len() - count as usize;
        let handle = cap_state.image_handles[index].clone();

        // Get the image entity
        let image_entity = cap_state.image_entity;
        let mut image_entity = world.entity_mut(image_entity);

        // Set display to not-none if the count was previously 0
        if count == 1 {
            if let Some(mut style) = image_entity.get_mut::<Style>() {
                style.display = Display::Flex;
            }
        }

        // Update image handle
        if let Some(mut image) = image_entity.get_mut::<UiImage>() {
            image.0 = handle;
        }
    }
}
