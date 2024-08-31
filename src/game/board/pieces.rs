use std::{fmt, ops::Not};

use bevy::{
    ecs::{
        component::{ComponentHooks, StorageType},
        world::Command,
    },
    prelude::*,
};
use chess::{Piece, Rank};

use crate::{
    debug_name_f,
    game::{
        consts::{Z_PIECE, Z_PIECE_SELECTED},
        LoadGame,
    },
    utils::{hook, NoopExts},
};

use super::{square::Square, BoardState, ChessBoardExts};

macro_rules! asset_path {
    ($color:literal, $type:literal) => {
        concat!("images/pieces/", $color, "-", $type, ".png")
    };
}

#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
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

pub(super) fn spawn_pieces_on_load_game(
    trigger: Trigger<LoadGame>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut board_state: ResMut<BoardState>,
    q_pieces: Query<Entity, With<PieceMeta>>,
) {
    // Despawn all pieces
    board_state.clear_pieces();
    q_pieces.iter().for_each(|e| commands.entity(e).despawn_recursive());

    for square in chess::ALL_SQUARES.map(Square::new) {
        let Some(info) = trigger.event().board.get_piece_meta(square) else { continue };
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

pub struct PieceAnimationPlugin;

impl Plugin for PieceAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            .init_resource::<IsPieceAnimationPluginAdded>()
            .add_systems(Startup, spawn_animation_layer)
            .add_systems(Update, animate_pieces)
            .noop();
    }
}

#[derive(Default, Resource)]
struct IsPieceAnimationPluginAdded;

#[derive(Component)]
struct AnimationLayer;

fn spawn_animation_layer(mut commands: Commands) {
    commands.spawn((
        Name::new("Animation Layer"),
        AnimationLayer,
        NodeBundle {
            style: Style {
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            z_index: ZIndex::Global(Z_PIECE_SELECTED),
            ..default()
        },
    ));
}

/// Setup `entity` (the piece) for animation.
///
/// If the `PieceAnimationPlugin` is not added to the app then this is a noop.
pub struct AnimatePiece {
    pub entity: Entity,
    pub from: Square,
    pub to: Square,
}

impl AnimatePiece {
    pub fn new(entity: Entity, from: Square, to: Square) -> Self {
        Self { entity, from, to }
    }
}

impl Command for AnimatePiece {
    fn apply(self, world: &mut World) {
        let Self { entity, from, to } = self;

        if world.get_resource::<IsPieceAnimationPluginAdded>().is_none() {
            warn!(%from, %to, ?entity, "PieceAnimationPlugin is not loaded, skipping animation");
            return;
        }

        let to_entity = world.resource::<BoardState>().tile(self.to);
        let to_transl = if let Ok(transf) = world.query::<&GlobalTransform>().get(world, to_entity)
        {
            transf.translation().truncate()
        } else {
            return;
        };

        let animation_layer = world.query_filtered::<Entity, With<AnimationLayer>>().single(world);

        let mut entity = world.entity_mut(self.entity);

        entity.set_parent(animation_layer);

        let size = entity.get::<Node>().unwrap().size();
        let ui_world_offset = size / 2.0;

        let transl = entity.get::<GlobalTransform>().unwrap().translation();
        let from_pos = transl.truncate() - ui_world_offset;
        let to_pos = to_transl - ui_world_offset;

        let mut style = entity.get_mut::<Style>().unwrap();
        style.left = Val::Px(from_pos.x);
        style.top = Val::Px(from_pos.y);
        style.width = Val::Px(size.x);
        style.height = Val::Px(size.y);

        entity.insert(Animating::new(from_pos, to_pos, to_entity));
    }
}

#[derive(Clone)]
struct Animating {
    timer: Timer,
    /// The cubic bezier function that drives the animation.
    ///
    /// See [the CSS easing function specification][1] for more info.
    ///
    /// [1]: https://www.w3.org/TR/css-easing-1/#cubic-bezier-easing-functions
    interpolater: CubicSegment<Vec2>,
    from: Vec2,
    to: Vec2,
    to_entity: Entity,
}

impl Component for Animating {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.noop().on_remove(hook!(Animating => on_remove_animating)).noop();
    }
}

impl Animating {
    fn new(from: Vec2, to: Vec2, to_entity: Entity) -> Self {
        Self {
            timer: Timer::from_seconds(0.07, TimerMode::Once),
            interpolater: CubicSegment::new_bezier((0.42, 0.0), (0.58, 1.0)),
            from,
            to,
            to_entity,
        }
    }
}

fn animate_pieces(
    mut commands: Commands,
    time: Res<Time>,
    mut q_animating: Query<(Entity, &mut Animating, &mut Style)>,
) {
    for (entity, mut animating, mut style) in &mut q_animating {
        if animating.timer.finished() {
            continue;
        }

        animating.timer.tick(time.delta());

        if animating.timer.just_finished() {
            commands.entity(entity).remove::<Animating>();
        }

        let t = animating.interpolater.ease(animating.timer.fraction());
        let Vec2 { x, y } = animating.from.lerp(animating.to, t);
        style.left = Val::Px(x);
        style.top = Val::Px(y);
    }
}

fn on_remove_animating(
    In((entity, animating)): In<(Entity, Animating)>,
    mut commands: Commands,
    mut q_style: Query<&mut Style>,
) {
    commands.entity(entity).set_parent(animating.to_entity);

    let mut style = q_style.get_mut(entity).unwrap();
    style.left = Val::Px(0.0);
    style.top = Val::Px(0.0);
    style.width = Val::Percent(100.0);
    style.height = Val::Percent(100.0);
}
