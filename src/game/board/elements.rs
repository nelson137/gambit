use std::ops::Not;

use bevy::{ecs::system::Command, prelude::*};
use chess::{File, Piece, Rank, Square};

use crate::{
    assets::PieceColorAndTypeAssetPath,
    debug_name,
    game::{
        board::{BoardLocation, MoveHints},
        consts::{Z_CHECKMATE_ICONS, Z_HIGHLIGHT_TILE, Z_MOVE_HINT, Z_NOTATION_TEXT, Z_PIECE},
    },
};

use crate::game::consts::Z_TILE;

use super::{BoardState, UiBoard};

// ======================================================================
// Icons
// ======================================================================

#[derive(Component)]
pub struct WinnerIcon;

#[derive(Component)]
pub struct LoserIconBlack;

#[derive(Component)]
pub struct LoserIconWhite;

pub fn spawn_checkmate_icons(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    board_state: Res<BoardState>,
) {
    let winner_icon_entity = commands
        .spawn((
            WinnerIcon,
            debug_name!("Winner Icon"),
            ImageBundle {
                image: asset_server.load("images/checkmate/winner.png").into(),
                visibility: Visibility::INVISIBLE,
                z_index: ZIndex::Global(Z_CHECKMATE_ICONS),
                ..default()
            },
        ))
        .id();
    commands.entity(board_state.tile(Square::A1)).add_child(winner_icon_entity);

    let black_loser_icon_entity = commands
        .spawn((
            LoserIconBlack,
            debug_name!("Black Loser Icon"),
            ImageBundle {
                image: asset_server.load("images/checkmate/loser-black.png").into(),
                visibility: Visibility::INVISIBLE,
                z_index: ZIndex::Global(Z_CHECKMATE_ICONS),
                ..default()
            },
        ))
        .id();
    commands.entity(board_state.tile(Square::A2)).add_child(black_loser_icon_entity);

    let white_loser_icon_entity = commands
        .spawn((
            LoserIconWhite,
            debug_name!("White Loser Icon"),
            ImageBundle {
                image: asset_server.load("images/checkmate/loser-white.png").into(),
                visibility: Visibility::INVISIBLE,
                z_index: ZIndex::Global(Z_CHECKMATE_ICONS),
                ..default()
            },
        ))
        .id();
    commands.entity(board_state.tile(Square::A3)).add_child(white_loser_icon_entity);
}

#[derive(Debug)]
pub struct ShowCheckmateIcons;

impl Command for ShowCheckmateIcons {
    fn write(self, world: &mut World) {
        let board_state = world.resource::<BoardState>();
        let board = board_state.board();

        let loser_color = board.side_to_move();
        let loser_square = board.king_square(loser_color);
        let loser_tile_entity = board_state.tile(loser_square);

        let winner_color = !loser_color;
        let winner_square = board.king_square(winner_color);
        let winner_tile_entity = board_state.tile(winner_square);

        #[rustfmt::skip]
        match loser_color {
            chess::Color::Black => set_checkmate_icon::<LoserIconBlack>(world, loser_tile_entity, loser_square),
            chess::Color::White => set_checkmate_icon::<LoserIconWhite>(world, loser_tile_entity, loser_square),
        };

        set_checkmate_icon::<LoserIconWhite>(world, winner_tile_entity, winner_square);
    }
}

fn set_checkmate_icon<IconMarker: Component>(
    world: &mut World,
    tile_entity: Entity,
    square: Square,
) {
    let icon_entity = world.query_filtered::<Entity, With<IconMarker>>().single(world);
    world.entity_mut(tile_entity).push_children(&[icon_entity]);
    let mut icon = world.entity_mut(icon_entity);
    icon.get_mut::<Visibility>().unwrap().is_visible = true;
    let mut style = icon.get_mut::<Style>().unwrap();
    if square.get_rank() == Rank::Eighth {
        style.position.top = Val::Percent(3.0);
    } else {
        style.position.top = Val::Percent(-14.0);
    }
    if square.get_file() == File::H {
        style.position.left = Val::Percent(57.0);
    } else {
        style.position.left = Val::Percent(74.0);
    }
}

pub fn checkmate_icon_size(
    q_tiles: Query<&Node, With<Tile>>,
    mut q_checkmate_icons: Query<
        &mut Style,
        Or<(With<WinnerIcon>, With<LoserIconBlack>, With<LoserIconWhite>)>,
    >,
) {
    let icon_size = {
        let tile_size = q_tiles.iter().next().unwrap().size().x;
        let size = Val::Px(tile_size * 0.4);
        Size::new(size, size)
    };
    for mut style in &mut q_checkmate_icons {
        style.size = icon_size;
    }
}

// ======================================================================
// Piece
// ======================================================================

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

// ======================================================================
// Move Hint & Capture Hint
// ======================================================================

#[derive(Default)]
pub struct ShowHints(pub Vec<Entity>);

impl Command for ShowHints {
    fn write(self, world: &mut World) {
        for entity in self.0 {
            if let Some(mut vis) = world.entity_mut(entity).get_mut::<Visibility>() {
                vis.is_visible = true;
            }
        }
    }
}

#[derive(Default)]
pub struct HideHints(pub Vec<Entity>);

impl Command for HideHints {
    fn write(self, world: &mut World) {
        for entity in self.0 {
            if let Some(mut vis) = world.entity_mut(entity).get_mut::<Visibility>() {
                vis.is_visible = false;
            }
        }
    }
}

// ======================================================================
// Hightlight Tile
// ======================================================================

#[derive(Component)]
pub struct HighlightTile;

/// The color used to highlight tiles.
pub const COLOR_HIGHLIGHT: Color = Color::rgba(1.0, 1.0, 0.0, 0.5);

#[derive(Deref, DerefMut)]
pub struct ShowHighlight(pub Entity);

impl Command for ShowHighlight {
    fn write(self, world: &mut World) {
        if let Some(mut vis) = world.entity_mut(*self).get_mut::<Visibility>() {
            vis.is_visible = true;
        }
    }
}

#[derive(Deref, DerefMut)]
pub struct HideHighlight(pub Entity);

impl Command for HideHighlight {
    fn write(self, world: &mut World) {
        if let Some(mut vis) = world.entity_mut(*self).get_mut::<Visibility>() {
            vis.is_visible = false;
        }
    }
}

// ======================================================================
// Tile
// ======================================================================

#[derive(Component)]
pub struct Tile;

/// The "black" bord color.
///
/// `#769656`
pub const COLOR_BLACK: Color = Color::rgb(
    0x76 as f32 / u8::MAX as f32,
    0x96 as f32 / u8::MAX as f32,
    0x56 as f32 / u8::MAX as f32,
);

/// The "white" bord color.
///
/// `#eeeed2`
pub const COLOR_WHITE: Color = Color::rgb(
    0xee as f32 / u8::MAX as f32,
    0xee as f32 / u8::MAX as f32,
    0xd2 as f32 / u8::MAX as f32,
);

// ======================================================================
// Spawn
// ======================================================================

pub fn spawn_board_elements(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut board_state: ResMut<BoardState>,
    q_board: Query<Entity, With<UiBoard>>,
) {
    let board = q_board.single();
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    let move_hint_texture = asset_server.load("images/hints/move.png");
    let capture_hint_texture = asset_server.load("images/hints/capture.png");
    let pos_top_left = UiRect { top: Val::Px(0.0), left: Val::Px(0.0), ..default() };
    let file_label_margins =
        UiRect { bottom: Val::Percent(3.5), right: Val::Percent(8.0), ..default() };
    let rank_label_margins =
        UiRect { top: Val::Percent(1.0), left: Val::Percent(4.5), ..default() };

    for square in chess::ALL_SQUARES {
        let file = square.get_file();
        let rank = square.get_rank();
        let location = BoardLocation::new(square);

        // Tile
        let file_rank_sum = rank.to_index() + file.to_index();
        let color = if file_rank_sum % 2 == 0 { COLOR_BLACK } else { COLOR_WHITE };
        let tile_entity = commands
            .spawn((
                Tile,
                debug_name!("Tile ({square})"),
                location,
                NodeBundle {
                    background_color: color.into(),
                    style: Style {
                        position_type: PositionType::Relative,
                        size: Size::new(Val::Percent(100.0 / 8.0), Val::Percent(100.0 / 8.0)),
                        ..default()
                    },
                    z_index: ZIndex::Local(Z_TILE),
                    ..default()
                },
            ))
            .with_children(|cmds| {
                pub const BOARD_TEXT_FONT_SIZE: f32 = 20.0;

                // File markers
                if square.get_rank() == Rank::First {
                    let text_style = TextStyle {
                        color: if file.to_index() % 2 == 0 { COLOR_WHITE } else { COLOR_BLACK },
                        font_size: BOARD_TEXT_FONT_SIZE,
                        font: font.clone(),
                    };
                    cmds.spawn(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: pos_top_left,
                            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            justify_content: JustifyContent::FlexEnd,
                            align_items: AlignItems::FlexEnd,
                            ..default()
                        },
                        z_index: ZIndex::Local(Z_NOTATION_TEXT),
                        ..default()
                    })
                    .with_children(|cmds| {
                        cmds.spawn(TextBundle {
                            text: Text::from_section(location.file_char(), text_style),
                            style: Style { margin: file_label_margins, ..default() },
                            ..default()
                        });
                    });
                }

                // Rank markers
                if square.get_file() == File::A {
                    let text_style = TextStyle {
                        color: if rank.to_index() % 2 == 0 { COLOR_WHITE } else { COLOR_BLACK },
                        font_size: BOARD_TEXT_FONT_SIZE,
                        font: font.clone(),
                    };
                    cmds.spawn(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: pos_top_left,
                            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::FlexStart,
                            ..default()
                        },
                        z_index: ZIndex::Local(Z_NOTATION_TEXT),
                        ..default()
                    })
                    .with_children(|cmds| {
                        cmds.spawn(TextBundle {
                            text: Text::from_section(location.rank_char(), text_style),
                            style: Style { margin: rank_label_margins, ..default() },
                            ..default()
                        });
                    });
                }

                // Highlight tile
                let hl_tile_entity = cmds
                    .spawn((
                        HighlightTile,
                        debug_name!("Highlight Tile ({square})"),
                        location,
                        NodeBundle {
                            background_color: COLOR_HIGHLIGHT.into(),
                            style: Style {
                                position_type: PositionType::Absolute,
                                position: pos_top_left,
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                ..default()
                            },
                            visibility: Visibility::INVISIBLE,
                            z_index: ZIndex::Local(Z_HIGHLIGHT_TILE),
                            ..default()
                        },
                    ))
                    .id();
                board_state.set_highlight(square, hl_tile_entity);

                // Move hint
                let move_entity = cmds
                    .spawn((
                        debug_name!("Move Hint ({square})"),
                        location,
                        NodeBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                position: pos_top_left,
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            visibility: Visibility::INVISIBLE,
                            z_index: ZIndex::Local(Z_MOVE_HINT),
                            ..default()
                        },
                    ))
                    .with_children(|cmds| {
                        cmds.spawn(ImageBundle {
                            image: UiImage(move_hint_texture.clone()),
                            style: Style {
                                size: Size::new(
                                    Val::Percent(100.0 / 3.0),
                                    Val::Percent(100.0 / 3.0),
                                ),
                                ..default()
                            },
                            ..default()
                        });
                    })
                    .id();

                // Capture hint
                let capture_entity = cmds
                    .spawn((
                        debug_name!("Capture Hint ({square})"),
                        location,
                        ImageBundle {
                            image: UiImage(capture_hint_texture.clone()),
                            style: Style {
                                position_type: PositionType::Absolute,
                                position: pos_top_left,
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                ..default()
                            },
                            visibility: Visibility::INVISIBLE,
                            z_index: ZIndex::Local(Z_MOVE_HINT),
                            ..default()
                        },
                    ))
                    .id();

                board_state.set_move_hints(square, MoveHints { capture_entity, move_entity });
            })
            .id();
        commands.entity(board).add_child(tile_entity);
        board_state.set_tile(square, tile_entity);
    }
}

pub fn spawn_board_pieces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut board_state: ResMut<BoardState>,
) {
    let pos_top_left = UiRect { top: Val::Px(0.0), left: Val::Px(0.0), ..default() };

    for square in chess::ALL_SQUARES {
        let location = BoardLocation::new(square);

        if let Some(info) = board_state.get_piece_info_on(square) {
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
}
