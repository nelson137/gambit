use bevy::prelude::*;
use chess::{File, Rank};

use crate::game::{LoadGame, consts::z};

use super::{BoardState, PieceColor, Square, Tile, TilesMap};

#[derive(Component, Clone, Copy, Default)]
#[require(Visibility::Hidden, ZIndex(z::END_GAME_ICONS))]
pub(super) enum EndGameIcon {
    #[default]
    Winner,
    LoserBlack,
    LoserWhite,
    DrawBlack,
    DrawWhite,
}

trait PieceColorToLoserIcon {
    fn to_loser_icon(self) -> EndGameIcon;
}

impl PieceColorToLoserIcon for PieceColor {
    fn to_loser_icon(self) -> EndGameIcon {
        match self {
            Self::BLACK => EndGameIcon::LoserBlack,
            Self::WHITE => EndGameIcon::LoserWhite,
        }
    }
}

#[derive(Resource)]
pub(super) struct EndGameIcons {
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

pub(super) fn end_game_icons() -> impl SceneList {
    fn image(path: &'static str) -> impl Scene {
        template(move |ctx| {
            let asset_server = ctx.resource::<AssetServer>();
            Ok(ImageNode::new(asset_server.load(path)))
        })
    }

    let scenes: Vec<Box<dyn Scene>> = vec![
        Box::new(bsn! {
            #WinnerIcon
            template_value(EndGameIcon::Winner)
            image("images/checkmate/winner.png")
        }),
        Box::new(bsn! {
            #BlackLoserIcon
            template_value(EndGameIcon::LoserBlack)
            image("images/checkmate/loser-black.png")
        }),
        Box::new(bsn! {
            #WhiteLoserIcon
            template_value(EndGameIcon::LoserWhite)
            image("images/checkmate/loser-white.png")
        }),
        Box::new(bsn! {
            #BlackDrawIcon
            template_value(EndGameIcon::DrawBlack)
            image("images/draw/draw-black.png")
        }),
        Box::new(bsn! {
            #WhiteDrawIcon
            template_value(EndGameIcon::DrawWhite)
            image("images/draw/draw-white.png")
        }),
    ];

    scenes
}

pub(super) fn load_end_game_icons(mut commands: Commands, q_icons: Query<(Entity, &EndGameIcon)>) {
    let mut winner = Entity::PLACEHOLDER;
    let mut loser_black = Entity::PLACEHOLDER;
    let mut loser_white = Entity::PLACEHOLDER;
    let mut draw_black = Entity::PLACEHOLDER;
    let mut draw_white = Entity::PLACEHOLDER;

    for (entity, icon) in &q_icons {
        match *icon {
            EndGameIcon::Winner => winner = entity,
            EndGameIcon::LoserBlack => loser_black = entity,
            EndGameIcon::LoserWhite => loser_white = entity,
            EndGameIcon::DrawBlack => draw_black = entity,
            EndGameIcon::DrawWhite => draw_white = entity,
        }
    }

    commands.insert_resource(EndGameIcons {
        winner,
        loser_black,
        loser_white,
        draw_black,
        draw_white,
    });
}

#[derive(Event, Debug)]
pub struct ShowCheckmateIcons;

impl ShowCheckmateIcons {
    pub(super) fn handle(
        _show_checkmate_icons: On<Self>,
        mut commands: Commands,
        board_state: Res<BoardState>,
    ) {
        let loser_color = board_state.side_to_move();
        let loser_square = board_state.king_square(loser_color);

        let winner_color = !loser_color;
        let winner_square = board_state.king_square(winner_color);

        trace!(%winner_square, %loser_square, "Show checkmate icons");

        commands.trigger(ShowEndGameIcon::new(loser_color.to_loser_icon(), loser_square));
        commands.trigger(ShowEndGameIcon::new(EndGameIcon::Winner, winner_square));
    }
}

#[derive(Event)]
pub struct ShowDrawIcons;

impl ShowDrawIcons {
    pub(super) fn handle(
        _show_draw_icons: On<Self>,
        mut commands: Commands,
        board_state: Res<BoardState>,
    ) {
        let black_square = board_state.king_square(PieceColor::BLACK);
        let white_square = board_state.king_square(PieceColor::WHITE);

        trace!(%white_square, %black_square, "Show draw icons");

        commands.trigger(ShowEndGameIcon::new(EndGameIcon::DrawBlack, black_square));
        commands.trigger(ShowEndGameIcon::new(EndGameIcon::DrawWhite, white_square));
    }
}

#[derive(Event, Clone, Copy)]
pub(super) struct ShowEndGameIcon {
    icon: EndGameIcon,
    square: Square,
}

impl ShowEndGameIcon {
    fn new(icon: EndGameIcon, square: Square) -> Self {
        Self { icon, square }
    }

    pub(super) fn handle(
        show_end_game_icons: On<Self>,
        mut commands: Commands,
        tiles_map: Res<TilesMap>,
        end_game_icons: Res<EndGameIcons>,
        mut q_icon: Query<(&mut Visibility, &mut Node)>,
    ) {
        let Self { icon, square } = *show_end_game_icons;
        let tile_entity = tiles_map.at(square);
        let icon_entity = end_game_icons.entity(icon);

        commands.entity(icon_entity).insert(ChildOf(tile_entity));

        let Ok((mut vis, mut node)) = q_icon.get_mut(icon_entity) else { return };

        *vis = Visibility::Visible;

        node.top = percent(if square.get_rank() == Rank::Eighth { 3.0 } else { -14.0 });
        node.left = percent(if square.get_file() == File::H { 57.0 } else { 74.0 });
    }
}

pub(super) fn end_game_icon_size(
    q_tiles: Query<&ComputedNode, With<Tile>>,
    mut q_end_game_icons: Query<&mut Node, With<EndGameIcon>>,
) {
    let Some(tile_computed_node) = q_tiles.iter().next() else { return };
    let tile_size = tile_computed_node.size().x * tile_computed_node.inverse_scale_factor();
    let icon_size = px(tile_size * 0.4);
    for mut node in &mut q_end_game_icons {
        node.width = icon_size;
        node.height = icon_size;
    }
}

pub(super) fn hide_end_game_icons_on_load_game(
    _load_game: On<LoadGame>,
    mut q_end_game_icons: Query<&mut Visibility, With<EndGameIcon>>,
) {
    q_end_game_icons.iter_mut().for_each(|mut vis| *vis = Visibility::Hidden);
}
