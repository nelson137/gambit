use std::ops::{Index, IndexMut};

use bevy::{
    ecs::system::{Command, SystemState},
    prelude::*,
};

use crate::game::board::{PieceColor, PieceType};

use super::board::UiPiece;

#[derive(Debug)]
pub struct CapturePlugin;

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CaptureState>().add_systems(PostUpdate, captures);
    }
}

#[derive(Deref, DerefMut, Resource)]
pub struct CaptureState(GameCaptures<CapState>);

impl FromWorld for CaptureState {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let mut captures = GameCaptures::<CapState>::default();

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

        Self(captures)
    }
}

impl CaptureState {
    #[cfg(debug_assertions)]
    #[allow(dead_code)]
    pub fn log_counts(&self) {
        fn log(color: &str, caps: &ColorCaptures<CapState>) {
            info!(
                pawns = caps[PieceType::PAWN].count,
                bishops = caps[PieceType::BISHOP].count,
                knights = caps[PieceType::KNIGHT].count,
                rooks = caps[PieceType::ROOK].count,
                queens = caps[PieceType::QUEEN].count,
                "{color} :"
            );
        }
        info!("");
        info!("================== Capture Counts ==================");
        log("White", &self.0[PieceColor::WHITE]);
        log("Black", &self.0[PieceColor::BLACK]);
    }
}

#[derive(Clone, Copy, Default, Deref, DerefMut)]
pub struct GameCaptures<C>(pub [ColorCaptures<C>; 2]);

impl<C> Index<PieceColor> for GameCaptures<C> {
    type Output = ColorCaptures<C>;

    fn index(&self, index: PieceColor) -> &Self::Output {
        self.0.index(index.0 as usize)
    }
}

impl<C> IndexMut<PieceColor> for GameCaptures<C> {
    fn index_mut(&mut self, index: PieceColor) -> &mut Self::Output {
        self.0.index_mut(index.0 as usize)
    }
}

#[derive(Clone, Copy, Default, Deref, DerefMut)]
pub struct ColorCaptures<C>(pub [C; 5]);

impl<C> Index<usize> for ColorCaptures<C> {
    type Output = C;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<C> IndexMut<usize> for ColorCaptures<C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

impl<C> Index<PieceType> for ColorCaptures<C> {
    type Output = C;

    fn index(&self, index: PieceType) -> &Self::Output {
        self.0.index(index.0 as usize)
    }
}

impl<C> IndexMut<PieceType> for ColorCaptures<C> {
    fn index_mut(&mut self, index: PieceType) -> &mut Self::Output {
        self.0.index_mut(index.0 as usize)
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

impl CapState {
    fn patch(&mut self, diff: CapStateDiff) {
        match diff {
            CapStateDiff::Increment => self.count += 1,
            CapStateDiff::Set(count) => self.count = count,
        }
    }
}

pub struct CapStateUpdate {
    color: PieceColor,
    typ: PieceType,
    diff: CapStateDiff,
}

impl CapStateUpdate {
    pub fn new(color: PieceColor, typ: PieceType, diff: CapStateDiff) -> Self {
        Self { color, typ, diff }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CapStateDiff {
    Increment,
    Set(u8),
}

impl Command for CapStateUpdate {
    fn apply(self, world: &mut World) {
        let CapStateUpdate { color, typ, diff } = self;
        trace!(side = ?self.color, typ = ?self.typ, action = ?self.diff, "Update capture state");

        let cap = &mut world.resource_mut::<CaptureState>()[color][typ];

        // Update the capture count
        cap.patch(diff);
        let count = cap.count;

        if count == 0 {
            let image_entity = cap.image_entity;
            let mut image_entity = world.entity_mut(image_entity);
            if let Some(mut style) = image_entity.get_mut::<Style>() {
                style.display = Display::None;
            }
        } else {
            // Images handles are ordered from most pieces to least. So for a count of 2 the index
            // of the image handle is `len - 2`.
            let index = cap.image_handles.len() - count as usize;
            let handle = cap.image_handles[index].clone();

            // Get the image entity
            let image_entity = cap.image_entity;
            if let Some(mut image_entity) = world.get_entity_mut(image_entity) {
                // Update image handle
                if let Some(mut image) = image_entity.get_mut::<UiImage>() {
                    image.texture = handle;
                }

                // Set display to not-none if the count was previously 0
                if let Some(mut style) = image_entity.get_mut::<Style>() {
                    style.display = Display::Flex;
                }
            }
        }
    }
}

#[derive(Component)]
pub struct Captured;

pub fn captures(
    mut commands: Commands,
    mut q_added: Query<(Entity, &UiPiece, &mut Visibility), Added<Captured>>,
) {
    for (entity, &UiPiece { mut color, typ }, mut vis) in &mut q_added {
        trace!(?color, ?typ, "Capture piece");

        commands.entity(entity).remove::<Captured>();

        *vis = Visibility::Hidden;

        // The count and capture image need to be updated for the player who performed the capture,
        // i.e. the one whose color is the opposite of that of the captured piece.
        color = !color;

        commands.add(CapStateUpdate::new(color, typ, CapStateDiff::Increment));
    }
}

pub struct ResetCapturesUi;

impl Command for ResetCapturesUi {
    fn apply(self, world: &mut World) {
        let mut state = SystemState::<ResMut<CaptureState>>::new(world);
        let mut capture_state = state.get_mut(world);

        let image_entities: Vec<Entity> = capture_state
            .iter_mut()
            .flat_map(|player_caps| player_caps.iter_mut())
            .map(|player_caps| {
                player_caps.count = 0;
                player_caps.image_entity
            })
            .collect();

        for img_entity in image_entities {
            if let Some(mut style) = world.entity_mut(img_entity).get_mut::<Style>() {
                style.display = Display::None;
            }
        }

        state.apply(world);
    }
}
