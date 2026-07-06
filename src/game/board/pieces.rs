use std::{
    collections::{HashMap, hash_map::Entry},
    fmt,
    ops::Not,
};

use bevy::prelude::*;
use chess::{Piece, Rank};

use crate::{
    debug_name, debug_name_f,
    game::{LoadGame, consts::z},
    utils::NoopExts,
};

use super::{ChessBoardExts, TilesMap, square::Square};

macro_rules! asset_path {
    ($color:literal, $type:literal) => {
        concat!("images/pieces/", $color, "-", $type, ".png")
    };
}

#[derive(Clone, Copy, Component, Debug, PartialEq, Eq)]
#[require(ZIndex(z::PIECE))]
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
    load_game: On<LoadGame>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    tiles_map: Res<TilesMap>,
    mut pieces_map: ResMut<PiecesMap>,
    q_pieces: Query<Entity, With<PieceMeta>>,
) {
    // Despawn all pieces
    pieces_map.clear();
    q_pieces.iter().for_each(|e| commands.entity(e).despawn());

    for square in chess::ALL_SQUARES.map(Square::new) {
        let Some(info) = load_game.board.get_piece_meta(square) else { continue };
        let image_path = info.asset_path();

        let piece_entity = commands
            .spawn((
                debug_name_f!("Piece ({} {}) ({square})", info.color, info.typ),
                info,
                square,
                ImageNode::new(asset_server.load(image_path)),
                Node {
                    position_type: PositionType::Absolute,
                    top: px(0.0),
                    left: px(0.0),
                    width: percent(100.0),
                    height: percent(100.0),
                    ..default()
                },
            ))
            .observe(super::move_piece)
            .observe(super::PromotingPiece::on_add)
            .observe(super::PromotingPiece::on_remove)
            .id();

        commands.entity(tiles_map.at(square)).add_child(piece_entity);
        pieces_map.set(square, piece_entity);
    }
}

#[derive(Resource)]
pub struct PiecesMap(HashMap<Square, Entity>);

impl Default for PiecesMap {
    fn default() -> Self {
        Self(HashMap::with_capacity(32))
    }
}

impl PiecesMap {
    fn set(&mut self, square: Square, entity: Entity) {
        match self.0.entry(square) {
            Entry::Occupied(_) => panic!("piece already in the state at {square}"),
            Entry::Vacant(e) => e.insert(entity),
        };
    }

    pub fn has(&self, square: Square) -> bool {
        self.0.contains_key(&square)
    }

    pub fn at(&self, square: Square) -> Entity {
        self.0.get(&square).copied().unwrap_or_else(|| panic!("no piece at {square}"))
    }

    pub fn remove(&mut self, square: Square) -> Option<Entity> {
        self.0.remove(&square)
    }

    pub fn move_en_passant(
        &mut self,
        to_sq: Square,
        piece: Entity,
        ep_sq: Square,
    ) -> Option<Entity> {
        self.0.insert(to_sq, piece);
        self.0.remove(&ep_sq)
    }

    pub fn entry(&mut self, square: Square) -> Entry<'_, Square, Entity> {
        self.0.entry(square)
    }

    pub fn clear(&mut self) {
        self.0.clear()
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
#[require(GlobalZIndex(z::G_PIECE_MOVE))]
struct AnimationLayer;

fn spawn_animation_layer(mut commands: Commands) {
    commands.spawn((
        debug_name!("Animation Layer"),
        AnimationLayer,
        Node {
            top: px(0.0),
            left: px(0.0),
            width: percent(100.0),
            height: percent(100.0),
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
    type Out = ();

    fn apply(self, world: &mut World) -> Self::Out {
        let Self { entity, from, to } = self;

        let to_entity = world.resource::<TilesMap>().at(self.to);

        if world.get_resource::<IsPieceAnimationPluginAdded>().is_none() {
            warn!(%from, %to, ?entity, "PieceAnimationPlugin is not loaded, skipping animation");
            world.entity_mut(entity).insert(ChildOf(to_entity));
            return;
        }

        let to_transl = if let Ok((transf, computed_node)) =
            world.query::<(&UiGlobalTransform, &ComputedNode)>().get(world, to_entity)
        {
            transf.translation.trunc() * computed_node.inverse_scale_factor()
        } else {
            return;
        };

        let animation_layer =
            world.query_filtered::<Entity, With<AnimationLayer>>().single(world).unwrap();

        let mut entity = world.entity_mut(self.entity);

        entity.insert(ChildOf(animation_layer));

        let computed_node = entity.get::<ComputedNode>().unwrap();
        let inverse_scale_factor = computed_node.inverse_scale_factor();
        let size = computed_node.size() * inverse_scale_factor;
        let ui_world_offset = size / 2.0;

        let transl =
            entity.get::<UiGlobalTransform>().unwrap().translation.trunc() * inverse_scale_factor;
        let from_pos = transl - ui_world_offset;
        let to_pos = to_transl - ui_world_offset;

        let mut node = entity.get_mut::<Node>().unwrap();
        node.left = px(from_pos.x);
        node.top = px(from_pos.y);
        node.width = px(size.x);
        node.height = px(size.y);

        entity.insert(Animating::new(from_pos, to_pos, to_entity));
    }
}

#[derive(Component, Clone)]
#[component(storage = "SparseSet")]
pub(super) struct Animating {
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

impl Animating {
    fn new(from: Vec2, to: Vec2, to_entity: Entity) -> Self {
        Self {
            timer: Timer::from_seconds(0.07, TimerMode::Once),
            interpolater: CubicSegment::new_bezier_easing((0.42, 0.0), (0.58, 1.0)),
            from,
            to,
            to_entity,
        }
    }
}

fn animate_pieces(
    mut commands: Commands,
    time: Res<Time>,
    mut q_animating: Query<(Entity, &mut Animating, &mut Node)>,
) {
    for (entity, mut animating, mut node) in &mut q_animating {
        if animating.timer.is_finished() {
            continue;
        }

        animating.timer.tick(time.delta());

        if animating.timer.just_finished() {
            commands.entity(entity).remove::<Animating>();
        }

        let t = animating.interpolater.ease(animating.timer.fraction());
        let Vec2 { x, y } = animating.from.lerp(animating.to, t);
        node.left = px(x);
        node.top = px(y);
    }
}

pub(super) fn on_remove_animating(
    remove: On<Remove, Animating>,
    mut commands: Commands,
    q_animating: Query<&Animating>,
    mut q_node: Query<&mut Node>,
) {
    let Ok(animating) = q_animating.get(remove.entity) else { return };

    commands.entity(remove.entity).insert(ChildOf(animating.to_entity));

    let mut node = q_node.get_mut(remove.entity).unwrap();
    node.left = px(0.0);
    node.top = px(0.0);
    node.width = percent(100.0);
    node.height = percent(100.0);
}
