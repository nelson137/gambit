use bevy::{ecs::system::Command, prelude::*};
use chess::{File, Rank};

use crate::{debug_name, game::consts::Z_END_GAME_ICONS};

use super::{BoardState, PieceColor, Square, Tile};

#[derive(Component)]
pub struct EndGameIcon;

#[derive(Component)]
pub struct WinnerIcon;

#[derive(Component)]
pub struct LoserIconBlack;

#[derive(Component)]
pub struct LoserIconWhite;

#[derive(Component)]
pub struct DrawIconBlack;

#[derive(Component)]
pub struct DrawIconWhite;

pub fn spawn_end_game_icons(
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
            ImageBundle {
                image: asset_server.load("images/checkmate/winner.png").into(),
                visibility: Visibility::Hidden,
                z_index: ZIndex::Global(Z_END_GAME_ICONS),
                ..default()
            },
        ))
        .id();
    commands.entity(board_state.tile(START_SQ)).add_child(winner_icon_entity);

    let black_loser_icon_entity = commands
        .spawn((
            EndGameIcon,
            LoserIconBlack,
            debug_name!("Black Loser Icon"),
            ImageBundle {
                image: asset_server.load("images/checkmate/loser-black.png").into(),
                visibility: Visibility::Hidden,
                z_index: ZIndex::Global(Z_END_GAME_ICONS),
                ..default()
            },
        ))
        .id();
    commands.entity(board_state.tile(START_SQ)).add_child(black_loser_icon_entity);

    let white_loser_icon_entity = commands
        .spawn((
            EndGameIcon,
            LoserIconWhite,
            debug_name!("White Loser Icon"),
            ImageBundle {
                image: asset_server.load("images/checkmate/loser-white.png").into(),
                visibility: Visibility::Hidden,
                z_index: ZIndex::Global(Z_END_GAME_ICONS),
                ..default()
            },
        ))
        .id();
    commands.entity(board_state.tile(START_SQ)).add_child(white_loser_icon_entity);

    let black_draw_entity = commands
        .spawn((
            EndGameIcon,
            DrawIconBlack,
            ImageBundle {
                image: asset_server.load("images/draw/draw-black.png").into(),
                visibility: Visibility::Hidden,
                z_index: ZIndex::Global(Z_END_GAME_ICONS),
                ..default()
            },
        ))
        .id();
    commands.entity(board_state.tile(START_SQ)).add_child(black_draw_entity);

    let white_draw_entity = commands
        .spawn((
            EndGameIcon,
            DrawIconWhite,
            ImageBundle {
                image: asset_server.load("images/draw/draw-white.png").into(),
                visibility: Visibility::Hidden,
                z_index: ZIndex::Global(Z_END_GAME_ICONS),
                ..default()
            },
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
        let loser_tile_entity = board_state.tile(loser_square);

        let winner_color = !loser_color;
        let winner_square = board_state.king_square(winner_color);
        let winner_tile_entity = board_state.tile(winner_square);

        trace!(%winner_square, %loser_square, "Show checkmate icons");

        #[rustfmt::skip]
        match loser_color {
            PieceColor::BLACK => set_end_game_icon::<LoserIconBlack>(world, loser_tile_entity, loser_square),
            PieceColor::WHITE => set_end_game_icon::<LoserIconWhite>(world, loser_tile_entity, loser_square),
        };

        set_end_game_icon::<WinnerIcon>(world, winner_tile_entity, winner_square);
    }
}

pub struct ShowStalemateIcons;

impl Command for ShowStalemateIcons {
    fn apply(self, world: &mut World) {
        let board_state = world.resource::<BoardState>();

        let black_square = board_state.king_square(PieceColor::BLACK);
        let black_tile_entity = board_state.tile(black_square);

        let white_square = board_state.king_square(PieceColor::WHITE);
        let white_tile_entity = board_state.tile(white_square);

        trace!(%white_square, %black_square, "Show stalemate icons");

        set_end_game_icon::<DrawIconBlack>(world, black_tile_entity, black_square);

        set_end_game_icon::<DrawIconWhite>(world, white_tile_entity, white_square);
    }
}

fn set_end_game_icon<IconMarker: Component>(
    world: &mut World,
    tile_entity: Entity,
    square: Square,
) {
    let (icon_entity, icon_parent) =
        world.query_filtered::<(Entity, &Parent), With<IconMarker>>().single(world);
    if icon_parent.get() != tile_entity {
        world.entity_mut(tile_entity).push_children(&[icon_entity]);
    }
    let mut icon = world.entity_mut(icon_entity);
    *icon.get_mut::<Visibility>().unwrap() = Visibility::Visible;
    let mut style = icon.get_mut::<Style>().unwrap();
    if square.get_rank() == Rank::Eighth {
        style.top = Val::Percent(3.0);
    } else {
        style.top = Val::Percent(-14.0);
    }
    if square.get_file() == File::H {
        style.left = Val::Percent(57.0);
    } else {
        style.left = Val::Percent(74.0);
    }
}

pub fn end_game_icon_size(
    q_tiles: Query<&Node, With<Tile>>,
    mut q_end_game_icons: Query<&mut Style, With<EndGameIcon>>,
) {
    let Some(tile_node) = q_tiles.iter().next() else { return };
    let tile_size = tile_node.size().x;
    let icon_size = Val::Px(tile_size * 0.4);
    for mut style in &mut q_end_game_icons {
        style.width = icon_size;
        style.height = icon_size;
    }
}
