use bevy::prelude::*;
use chess::{File, Rank};

use crate::{
    debug_name,
    game::{LoadGame, consts::Z_END_GAME_ICONS},
};

use super::{BoardState, PieceColor, Square, Tile};

#[derive(Component)]
pub(super) enum EndGameIcon {
    Winner,
    LoserBlack,
    LoserWhite,
    DrawBlack,
    DrawWhite,
}

#[derive(Resource)]
struct EndGameIcons {
    winner: Entity,
    loser_black: Entity,
    loser_white: Entity,
    draw_black: Entity,
    draw_white: Entity,
}

impl EndGameIcons {
    fn entity(&self, icon: EndGameIcon) -> Entity {
        match icon {
            EndGameIcon::Winner => self.winner,
            EndGameIcon::LoserBlack => self.loser_black,
            EndGameIcon::LoserWhite => self.loser_white,
            EndGameIcon::DrawBlack => self.draw_black,
            EndGameIcon::DrawWhite => self.draw_white,
        }
    }
}

pub(super) fn spawn_end_game_icons(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    board_state: Res<BoardState>,
) {
    let parent = board_state.tile(Square::DEFAULT);

    let winner = commands
        .spawn((
            EndGameIcon::Winner,
            debug_name!("Winner Icon"),
            ImageNode::new(asset_server.load("images/checkmate/winner.png")),
            Visibility::Hidden,
            GlobalZIndex(Z_END_GAME_ICONS),
            ChildOf(parent),
        ))
        .id();

    let loser_black = commands
        .spawn((
            EndGameIcon::LoserBlack,
            debug_name!("Black Loser Icon"),
            ImageNode::new(asset_server.load("images/checkmate/loser-black.png")),
            Visibility::Hidden,
            GlobalZIndex(Z_END_GAME_ICONS),
            ChildOf(parent),
        ))
        .id();

    let loser_white = commands
        .spawn((
            EndGameIcon::LoserWhite,
            debug_name!("White Loser Icon"),
            ImageNode::new(asset_server.load("images/checkmate/loser-white.png")),
            Visibility::Hidden,
            GlobalZIndex(Z_END_GAME_ICONS),
            ChildOf(parent),
        ))
        .id();

    let draw_black = commands
        .spawn((
            EndGameIcon::DrawBlack,
            ImageNode::new(asset_server.load("images/draw/draw-black.png")),
            Visibility::Hidden,
            GlobalZIndex(Z_END_GAME_ICONS),
            ChildOf(parent),
        ))
        .id();

    let draw_white = commands
        .spawn((
            EndGameIcon::DrawWhite,
            ImageNode::new(asset_server.load("images/draw/draw-white.png")),
            Visibility::Hidden,
            GlobalZIndex(Z_END_GAME_ICONS),
            ChildOf(parent),
        ))
        .id();

    commands.insert_resource(EndGameIcons {
        winner,
        loser_black,
        loser_white,
        draw_black,
        draw_white,
    })
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
            PieceColor::BLACK => set_end_game_icon(world, EndGameIcon::LoserBlack, loser_square),
            PieceColor::WHITE => set_end_game_icon(world, EndGameIcon::LoserWhite, loser_square),
        };

        set_end_game_icon(world, EndGameIcon::Winner, winner_square);
    }
}

pub struct ShowDrawIcons;

impl Command for ShowDrawIcons {
    fn apply(self, world: &mut World) {
        let board_state = world.resource::<BoardState>();

        let black_square = board_state.king_square(PieceColor::BLACK);
        let white_square = board_state.king_square(PieceColor::WHITE);

        trace!(%white_square, %black_square, "Show draw icons");

        set_end_game_icon(world, EndGameIcon::DrawBlack, black_square);
        set_end_game_icon(world, EndGameIcon::DrawWhite, white_square);
    }
}

fn set_end_game_icon(world: &mut World, icon: EndGameIcon, square: Square) {
    let tile_entity = world.resource::<BoardState>().tile(square);
    let icon_entity = world.resource::<EndGameIcons>().entity(icon);
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
