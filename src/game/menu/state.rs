use std::{fmt, hash, mem};

use bevy::prelude::*;

use crate::{
    cli::CliArgs,
    game::{
        board::{spawn_pieces, BoardState, EndGameIcon, PieceColor, Square},
        captures::ResetCapturesUi,
        load::DespawnPieces,
    },
    utils::StateExts,
};

use super::{FenPopupData, GameMenu, GameMenuDimLayer};

#[derive(Clone, Copy, Debug, Eq)]
pub enum MenuState {
    FenInput,
    Menu,
    Game,
    GamePromotion { entity: Entity, color: PieceColor, from_sq: Square, to_sq: Square },
    DoGameOver,
}

impl FromWorld for MenuState {
    fn from_world(world: &mut World) -> Self {
        match world.resource::<CliArgs>().fen {
            Some(_) => Self::Game,
            _ => Self::Menu,
        }
    }
}

impl PartialEq for MenuState {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }
}

impl hash::Hash for MenuState {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        mem::discriminant(self).hash(state);
    }
}

impl MenuState {
    pub const GAME_PROMOTION: Self = Self::GamePromotion {
        entity: Entity::from_raw(0),
        color: PieceColor::WHITE,
        from_sq: Square::DEFAULT,
        to_sq: Square::DEFAULT,
    };
}

impl fmt::Display for MenuState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub(super) fn on_enter_menu_state(
    menu_state: ResMut<State<MenuState>>,
    mut fen_popup_data: ResMut<FenPopupData>,
    mut game_over_timer: ResMut<GameOverTimer>,
    mut q_menu_components: Query<&mut Style, Or<(With<GameMenuDimLayer>, With<GameMenu>)>>,
) {
    let mut set_menu_display =
        |d| q_menu_components.iter_mut().for_each(|mut style| style.display = d);
    match menu_state.current() {
        MenuState::FenInput => fen_popup_data.reset(),
        MenuState::Menu => set_menu_display(Display::Flex),
        MenuState::Game => set_menu_display(Display::None),
        MenuState::GamePromotion { .. } => {}
        MenuState::DoGameOver => *game_over_timer = default(),
    }
}

#[derive(Resource)]
pub(super) struct GameOverTimer(Timer);

impl Default for GameOverTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Once))
    }
}

pub(super) fn game_over(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut game_over_timer: ResMut<GameOverTimer>,
    mut menu_state: ResMut<State<MenuState>>,
    mut board_state: ResMut<BoardState>,
    mut q_end_game_icons: Query<&mut Visibility, With<EndGameIcon>>,
) {
    game_over_timer.0.tick(time.delta());
    if game_over_timer.0.just_finished() {
        q_end_game_icons.for_each_mut(|mut vis| vis.is_visible = false);

        commands.add(board_state.unselect_square());
        commands.add(board_state.hide_last_move_highlights());
        board_state.reset();

        commands.add(ResetCapturesUi);

        commands.add(DespawnPieces);
        spawn_pieces(commands, asset_server, board_state);

        menu_state.transition(MenuState::Menu);
    }
}
