use bevy::prelude::*;
use chess::{File, Rank};

use crate::{
    debug_name,
    game::{LoadGame, consts::Z_END_GAME_ICONS},
};

use super::{BoardState, PieceColor, Square, Tile};

#[derive(Component)]
pub(super) struct EndGameIcon;

#[derive(Component)]
struct WinnerIcon;

#[derive(Component)]
struct LoserIconBlack;

#[derive(Component)]
struct LoserIconWhite;

#[derive(Component)]
struct DrawIconBlack;

#[derive(Component)]
struct DrawIconWhite;

pub(super) fn spawn_end_game_icons(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    board_state: Res<BoardState>,
) {
    const START_SQ: Square = Square::DEFAULT;

    let winner_icon_entity = commands
        .spawn((
            EndGameIcon,
            WinnerIcon,
            debug_name!("Winner Icon"),
            ImageNode::new(asset_server.load("images/checkmate/winner.png")),
            Visibility::Hidden,
            GlobalZIndex(Z_END_GAME_ICONS),
        ))
        .id();
    commands.entity(board_state.tile(START_SQ)).add_child(winner_icon_entity);

    let black_loser_icon_entity = commands
        .spawn((
            EndGameIcon,
            LoserIconBlack,
            debug_name!("Black Loser Icon"),
            ImageNode::new(asset_server.load("images/checkmate/loser-black.png")),
            Visibility::Hidden,
            GlobalZIndex(Z_END_GAME_ICONS),
        ))
        .id();
    commands.entity(board_state.tile(START_SQ)).add_child(black_loser_icon_entity);

    let white_loser_icon_entity = commands
        .spawn((
            EndGameIcon,
            LoserIconWhite,
            debug_name!("White Loser Icon"),
            ImageNode::new(asset_server.load("images/checkmate/loser-white.png")),
            Visibility::Hidden,
            GlobalZIndex(Z_END_GAME_ICONS),
        ))
        .id();
    commands.entity(board_state.tile(START_SQ)).add_child(white_loser_icon_entity);

    let black_draw_entity = commands
        .spawn((
            EndGameIcon,
            DrawIconBlack,
            ImageNode::new(asset_server.load("images/draw/draw-black.png")),
            Visibility::Hidden,
            GlobalZIndex(Z_END_GAME_ICONS),
        ))
        .id();
    commands.entity(board_state.tile(START_SQ)).add_child(black_draw_entity);

    let white_draw_entity = commands
        .spawn((
            EndGameIcon,
            DrawIconWhite,
            ImageNode::new(asset_server.load("images/draw/draw-white.png")),
            Visibility::Hidden,
            GlobalZIndex(Z_END_GAME_ICONS),
        ))
        .id();
    commands.entity(board_state.tile(START_SQ)).add_child(white_draw_entity);
}

#[derive(Debug)]
pub struct ShowCheckmateIcons;

impl Command for ShowCheckmateIcons {
    fn apply(self, world: &mut World) {
        let board_state = world.resource::<BoardState>();

        let loser_color = board_state.side_to_move();
        let loser_square = board_state.king_square(loser_color);

        let winner_color = !loser_color;
        let winner_square = board_state.king_square(winner_color);

        trace!(%winner_square, %loser_square, "Show checkmate icons");

        match loser_color {
            PieceColor::BLACK => set_end_game_icon::<LoserIconBlack>(world, loser_square),
            PieceColor::WHITE => set_end_game_icon::<LoserIconWhite>(world, loser_square),
        };

        set_end_game_icon::<WinnerIcon>(world, winner_square);
    }
}

pub struct ShowDrawIcons;

impl Command for ShowDrawIcons {
    fn apply(self, world: &mut World) {
        let board_state = world.resource::<BoardState>();

        let black_square = board_state.king_square(PieceColor::BLACK);
        let white_square = board_state.king_square(PieceColor::WHITE);

        trace!(%white_square, %black_square, "Show draw icons");

        set_end_game_icon::<DrawIconBlack>(world, black_square);
        set_end_game_icon::<DrawIconWhite>(world, white_square);
    }
}

fn set_end_game_icon<IconMarker: Component>(world: &mut World, square: Square) {
    let tile_entity = world.resource::<BoardState>().tile(square);
    let icon_entity = world.query_filtered::<Entity, With<IconMarker>>().single(world).unwrap();
    world.entity_mut(icon_entity).insert(ChildOf(tile_entity));
    let mut icon = world.entity_mut(icon_entity);
    *icon.get_mut::<Visibility>().unwrap() = Visibility::Visible;
    let mut node = icon.get_mut::<Node>().unwrap();
    if square.get_rank() == Rank::Eighth {
        node.top = Val::Percent(3.0);
    } else {
        node.top = Val::Percent(-14.0);
    }
    if square.get_file() == File::H {
        node.left = Val::Percent(57.0);
    } else {
        node.left = Val::Percent(74.0);
    }
}

pub(super) fn end_game_icon_size(
    q_tiles: Query<&ComputedNode, With<Tile>>,
    mut q_end_game_icons: Query<&mut Node, With<EndGameIcon>>,
) {
    let Some(tile_computed_node) = q_tiles.iter().next() else { return };
    let tile_size = tile_computed_node.size().x * tile_computed_node.inverse_scale_factor();
    let icon_size = Val::Px(tile_size * 0.4);
    for mut node in &mut q_end_game_icons {
        node.width = icon_size;
        node.height = icon_size;
    }
}

pub(super) fn hide_end_game_icons_on_load_game(
    _trigger: Trigger<LoadGame>,
    mut q_end_game_icons: Query<&mut Visibility, With<EndGameIcon>>,
) {
    q_end_game_icons.iter_mut().for_each(|mut vis| *vis = Visibility::Hidden);
}
