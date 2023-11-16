use std::{fmt, hash, mem};

use bevy::prelude::*;

use crate::{
    cli::CliArgs,
    game::{
        board::{spawn_pieces, BoardState, EndGameIcon, ResetCapturesUi, SelectionEvent},
        load::DespawnPieces,
    },
};

use super::{FenPopupData, GameMenu, GameMenuDimLayer};

#[derive(Clone, Copy, Debug, Default, Eq, States)]
pub enum MenuState {
    FenInput,
    #[default]
    Menu,
    Game,
    DoGameOver,
}

pub(super) fn init_menu_state_from_cli(
    cli_args: Res<CliArgs>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
) {
    next_menu_state.set(match cli_args.fen {
        Some(_) => MenuState::Game,
        _ => MenuState::Menu,
    });
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
    match *menu_state.get() {
        MenuState::FenInput => fen_popup_data.reset(),
        MenuState::Menu => set_menu_display(Display::Flex),
        MenuState::Game => set_menu_display(Display::None),
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
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut board_state: ResMut<BoardState>,
    mut selection_events: EventWriter<SelectionEvent>,
    mut q_end_game_icons: Query<&mut Visibility, With<EndGameIcon>>,
) {
    game_over_timer.0.tick(time.delta());
    if game_over_timer.0.just_finished() {
        trace!("Reset game");

        q_end_game_icons.for_each_mut(|mut vis| *vis = Visibility::Hidden);

        selection_events.send(SelectionEvent::UnsetAll);
        board_state.reset();

        commands.add(ResetCapturesUi);

        commands.add(DespawnPieces);
        spawn_pieces(commands, asset_server, board_state);

        next_menu_state.set(MenuState::Menu);
    }
}
