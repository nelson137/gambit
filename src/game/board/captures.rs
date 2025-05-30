use std::ops::{Index, IndexMut};

use bevy::prelude::*;

use crate::{
    game::{
        LoadGame,
        board::{PieceColor, PieceType},
        panels::MaterialAdvantageLabel,
    },
    utils::NoopExts,
};

use super::{BoardPlugin, PieceMeta};

#[derive(Debug)]
pub struct CapturePlugin;

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<BoardPlugin>() {
            panic!("Attempted to add plugin without required dependency: {BoardPlugin:?}");
        }

        app.noop()
            // Resources
            .init_resource::<CaptureState>()
            // Observers
            .add_observer(load_capture_state)
            .add_observer(captures)
            .noop();
    }
}

const CAPTURABLE_PIECES: [PieceType; chess::NUM_PIECES - 1] =
    [PieceType::PAWN, PieceType::KNIGHT, PieceType::BISHOP, PieceType::ROOK, PieceType::QUEEN];

#[derive(Deref, DerefMut, Resource)]
pub struct CaptureState([ColorCaptures; 2]);

impl FromWorld for CaptureState {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let mut state = CaptureState::new();

        state[PieceColor::BLACK][PieceType::PAWN].image_handles.extend([
            default(),
            asset_server.load("images/captures/white-pawns-1.png"),
            asset_server.load("images/captures/white-pawns-2.png"),
            asset_server.load("images/captures/white-pawns-3.png"),
            asset_server.load("images/captures/white-pawns-4.png"),
            asset_server.load("images/captures/white-pawns-5.png"),
            asset_server.load("images/captures/white-pawns-6.png"),
            asset_server.load("images/captures/white-pawns-7.png"),
            asset_server.load("images/captures/white-pawns-8.png"),
        ]);
        state[PieceColor::BLACK][PieceType::BISHOP].image_handles.extend([
            default(),
            asset_server.load("images/captures/white-bishops-1.png"),
            asset_server.load("images/captures/white-bishops-2.png"),
        ]);
        state[PieceColor::BLACK][PieceType::KNIGHT].image_handles.extend([
            default(),
            asset_server.load("images/captures/white-knights-1.png"),
            asset_server.load("images/captures/white-knights-2.png"),
        ]);
        state[PieceColor::BLACK][PieceType::ROOK].image_handles.extend([
            default(),
            asset_server.load("images/captures/white-rooks-1.png"),
            asset_server.load("images/captures/white-rooks-2.png"),
        ]);
        state[PieceColor::BLACK][PieceType::QUEEN]
            .image_handles
            .extend([default(), asset_server.load("images/captures/white-queen.png")]);

        state[PieceColor::WHITE][PieceType::PAWN].image_handles.extend([
            default(),
            asset_server.load("images/captures/black-pawns-1.png"),
            asset_server.load("images/captures/black-pawns-2.png"),
            asset_server.load("images/captures/black-pawns-3.png"),
            asset_server.load("images/captures/black-pawns-4.png"),
            asset_server.load("images/captures/black-pawns-5.png"),
            asset_server.load("images/captures/black-pawns-6.png"),
            asset_server.load("images/captures/black-pawns-7.png"),
            asset_server.load("images/captures/black-pawns-8.png"),
        ]);
        state[PieceColor::WHITE][PieceType::BISHOP].image_handles.extend([
            default(),
            asset_server.load("images/captures/black-bishops-1.png"),
            asset_server.load("images/captures/black-bishops-2.png"),
        ]);
        state[PieceColor::WHITE][PieceType::KNIGHT].image_handles.extend([
            default(),
            asset_server.load("images/captures/black-knights-1.png"),
            asset_server.load("images/captures/black-knights-2.png"),
        ]);
        state[PieceColor::WHITE][PieceType::ROOK].image_handles.extend([
            default(),
            asset_server.load("images/captures/black-rooks-1.png"),
            asset_server.load("images/captures/black-rooks-2.png"),
        ]);
        state[PieceColor::WHITE][PieceType::QUEEN]
            .image_handles
            .extend([default(), asset_server.load("images/captures/black-queen.png")]);

        state
    }
}

impl CaptureState {
    pub fn new() -> Self {
        Self(default())
    }

    pub fn get_advantage(&self) -> Option<(PieceColor, u8)> {
        match (self[PieceColor::BLACK].score, self[PieceColor::WHITE].score) {
            (black_score, white_score) if black_score > white_score => {
                Some((PieceColor::BLACK, black_score - white_score))
            }
            (black_score, white_score) if white_score > black_score => {
                Some((PieceColor::WHITE, white_score - black_score))
            }
            _ => None,
        }
    }

    pub fn patch(&mut self, update: CapStateUpdate) -> &CapState {
        let color_caps = &mut self[update.color];
        color_caps[update.typ].patch(update.diff);
        color_caps.score =
            CAPTURABLE_PIECES.into_iter().map(|typ| typ.value() * color_caps[typ].count).sum();
        &color_caps[update.typ]
    }

    #[cfg(debug_assertions)]
    #[allow(dead_code)]
    pub fn log_counts(&self) {
        fn log(color: &str, caps: &ColorCaptures) {
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
        log("White", &self[PieceColor::WHITE]);
        log("Black", &self[PieceColor::BLACK]);
    }
}

impl Index<PieceColor> for CaptureState {
    type Output = ColorCaptures;

    fn index(&self, index: PieceColor) -> &Self::Output {
        self.0.index(index.0 as usize)
    }
}

impl IndexMut<PieceColor> for CaptureState {
    fn index_mut(&mut self, index: PieceColor) -> &mut Self::Output {
        self.0.index_mut(index.0 as usize)
    }
}

#[derive(Clone, Default, Deref, DerefMut)]
pub struct ColorCaptures {
    #[deref]
    piece_captures: [CapState; 5],
    score: u8,
}

impl Index<usize> for ColorCaptures {
    type Output = CapState;

    fn index(&self, index: usize) -> &Self::Output {
        self.piece_captures.index(index)
    }
}

impl IndexMut<usize> for ColorCaptures {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.piece_captures.index_mut(index)
    }
}

impl Index<PieceType> for ColorCaptures {
    type Output = CapState;

    fn index(&self, index: PieceType) -> &Self::Output {
        self.piece_captures.index(index.0 as usize)
    }
}

impl IndexMut<PieceType> for ColorCaptures {
    fn index_mut(&mut self, index: PieceType) -> &mut Self::Output {
        self.piece_captures.index_mut(index.0 as usize)
    }
}

#[derive(Clone)]
pub struct CapState {
    pub image_handles: Vec<Handle<Image>>,
    pub image_entity: Entity,
    pub count: u8,
}

impl Default for CapState {
    fn default() -> Self {
        Self { image_handles: Vec::new(), image_entity: Entity::PLACEHOLDER, count: 0 }
    }
}

impl CapState {
    /// Return the image handle for the current capture count.
    fn handle(&self) -> Handle<Image> {
        self.image_handles[self.count as usize].clone()
    }

    /// Apply the capture state diff. Return the image handle for the new
    /// capture count.
    fn patch(&mut self, diff: CapStateDiff) {
        match diff {
            CapStateDiff::Increment => {
                self.count = (self.count + 1) % self.image_handles.len() as u8
            }
            CapStateDiff::Set(count) => self.count = count,
        }
    }
}

fn load_capture_state(trigger: Trigger<LoadGame>, mut commands: Commands) {
    let board = &trigger.event().board;
    let kings_bb = board.pieces(chess::Piece::King);

    for color in chess::ALL_COLORS {
        let opponent_pieces_bb = *board.color_combined(!color) & !kings_bb;
        let color = PieceColor(color);

        for typ in CAPTURABLE_PIECES {
            let opponent_pieces_of_type = opponent_pieces_bb & *board.pieces(typ.into());
            let captured_count = typ.num_pieces() - opponent_pieces_of_type.popcnt() as u8;
            let diff = CapStateDiff::Set(captured_count);
            commands.queue(CapStateUpdate::new(color, typ, diff));
        }
    }
}

#[derive(Clone, Copy, Debug)]
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
        trace!(side = ?self.color, typ = ?self.typ, action = ?self.diff, "Update capture state");

        let mut state = world.resource_mut::<CaptureState>();
        let cap = state.patch(self);
        let count = cap.count;
        let image_entity = cap.image_entity;
        let handle = cap.handle();

        if let Ok(mut image_entity) = world.get_entity_mut(image_entity) {
            if let Some(mut node) = image_entity.get_mut::<Node>() {
                node.display = match count {
                    0 => Display::None,
                    _ => Display::Flex,
                };
            }
            if let Some(mut image) = image_entity.get_mut::<ImageNode>() {
                image.image = handle;
            }
        }

        if let Some((color_with_adv, adv)) = world.resource::<CaptureState>().get_advantage() {
            let mut q = world.query::<(&MaterialAdvantageLabel, &mut Visibility, &mut Text)>();
            for (label, mut vis, mut text) in q.iter_mut(world) {
                if **label == color_with_adv {
                    *vis = Visibility::Visible;
                    text.0 = format!("+{adv}");
                } else {
                    *vis = Visibility::Hidden;
                }
            }
        } else {
            world
                .query_filtered::<&mut Visibility, With<MaterialAdvantageLabel>>()
                .iter_mut(world)
                .for_each(|mut vis| *vis = Visibility::Hidden);
        }
    }
}

#[derive(Event)]
pub struct Captured;

pub fn captures(
    trigger: Trigger<Captured>,
    mut commands: Commands,
    mut q_data: Query<(&PieceMeta, &mut Visibility)>,
) {
    let Ok((&PieceMeta { mut color, typ }, mut vis)) = q_data.get_mut(trigger.target()) else {
        return;
    };
    trace!(?color, ?typ, "Capture piece");

    *vis = Visibility::Hidden;

    // The count and capture image need to be updated for the player who performed the capture,
    // i.e. the one whose color is the opposite of that of the captured piece.
    color = !color;

    commands.queue(CapStateUpdate::new(color, typ, CapStateDiff::Increment));
}

pub struct ResetCapturesUi;

impl Command for ResetCapturesUi {
    fn apply(self, world: &mut World) {
        let mut capture_state = world.resource_mut::<CaptureState>();

        let image_entities: Vec<Entity> = capture_state
            .iter_mut()
            .flat_map(|player_caps| player_caps.iter_mut())
            .map(|player_caps| {
                player_caps.count = 0;
                player_caps.image_entity
            })
            .collect();

        for img_entity in image_entities {
            if let Some(mut node) = world.entity_mut(img_entity).get_mut::<Node>() {
                node.display = Display::None;
            }
        }

        world
            .query_filtered::<&mut Visibility, With<MaterialAdvantageLabel>>()
            .iter_mut(world)
            .for_each(|mut vis| *vis = Visibility::Hidden);
    }
}
