use std::{fmt, hash, mem};

use bevy::prelude::*;

use crate::{cli::CliArgs, game::LoadGame};

use super::{GameMenu, GameMenuDimLayer, PopupState};

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

pub(super) fn set_state_to_game_on_load(
    trigger: Trigger<LoadGame>,
    mut next_state: ResMut<NextState<MenuState>>,
) {
    next_state.set(trigger.event().menu_state);
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
    menu_state: Res<State<MenuState>>,
    mut fen_popup_state: ResMut<PopupState>,
    mut game_over_timer: ResMut<GameOverTimer>,
    mut q_menu_components: Query<&mut Style, Or<(With<GameMenuDimLayer>, With<GameMenu>)>>,
) {
    let mut set_menu_display =
        |d| q_menu_components.iter_mut().for_each(|mut style| style.display = d);
    match *menu_state.get() {
        MenuState::FenInput => fen_popup_state.reset(),
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
    mut game_over_timer: ResMut<GameOverTimer>,
) {
    game_over_timer.0.tick(time.delta());
    if game_over_timer.0.just_finished() {
        trace!("Reset game");
        commands.trigger(LoadGame::in_menu(default()));
    }
}
