use std::{
    ops::{Index, IndexMut},
    sync::Arc,
};

use bevy::{ecs::system::Command, prelude::*};

use crate::data::{BoardPiece, PieceColor, PieceType};

#[derive(Resource)]
pub struct CaptureState {
    pub image_handles: Arc<PlayerCaptures<Vec<Handle<Image>>>>,
    pub image_entities: Arc<PlayerCaptures<Entity>>,
    pub counts: PlayerCaptures<u8>,
}

const DEFAULT_ENTITY: Entity = Entity::from_raw(u32::MAX);

impl Default for CaptureState {
    fn default() -> Self {
        Self {
            image_handles: Default::default(),
            image_entities: Arc::new(PlayerCaptures::with(DEFAULT_ENTITY)),
            counts: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Default, Deref, DerefMut)]
pub struct PlayerCaptures<C>(pub [PieceCaptures<C>; 2]);

impl<C: Copy> PlayerCaptures<C> {
    const fn with(c: C) -> Self {
        Self([PieceCaptures::new(c); 2])
    }
}

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

impl<C: Copy> PieceCaptures<C> {
    const fn new(c: C) -> Self {
        Self([c; 5])
    }
}

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

impl CaptureState {
    pub fn update_count(&mut self, BoardPiece { color, typ, .. }: BoardPiece) -> u8 {
        let count = &mut self.counts[color][typ];
        *count += 1;
        *count
    }
}

#[derive(Clone, Copy, Deref, DerefMut)]
pub struct Captured(pub BoardPiece);

impl Command for Captured {
    fn write(self, world: &mut World) {
        let BoardPiece { entity, mut color, typ } = *self;
        // The count and capture image need to be updated for the player who performed the capture,
        // i.e. the one whose color is the opposite of that of the captured piece.
        color = !color;

        // Hide piece
        if let Some(mut vis) = world.entity_mut(entity).get_mut::<Visibility>() {
            vis.is_visible = false;
        }

        let mut capture_state = world.resource_mut::<CaptureState>();

        // Update count
        let count = capture_state.update_count(*self);

        // Get the handle to the correct image for the updated count
        let image_handles = &capture_state.image_handles[color][typ];
        let index = image_handles.len() - count as usize;
        let handle = image_handles[index].clone();

        // Get the image entity
        let image_entity = capture_state.image_entities[color][typ];
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
