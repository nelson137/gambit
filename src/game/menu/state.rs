use std::{fmt, hash, mem};

use bevy::{ecs::system::QueryLens, prelude::*};

use crate::{cli::CliArgs, game::LoadGame};

use super::{FenInputMenu, FenInputMenuState, GameMenu, GameMenuDimLayer};

#[derive(Clone, Copy, Debug, Default, Eq, States)]
pub enum MenuState {
    #[default]
    Main,
    FenInput,
    Game,
}

pub(super) fn init_menu_state_from_cli(
    cli_args: Res<CliArgs>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
) {
    next_menu_state.set(match cli_args.fen {
        Some(_) => MenuState::Game,
        _ => MenuState::Main,
    });
}

pub(super) fn set_state_to_game_on_load(
    load_game: On<LoadGame>,
    mut next_state: ResMut<NextState<MenuState>>,
) {
    next_state.set(load_game.menu_state);
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

pub(super) struct OnEnterMenuState;

impl OnEnterMenuState {
    pub(super) fn main(
        mut q: ParamSet<(
            Query<&mut Node, With<GameMenuDimLayer>>,
            Query<&mut Node, With<GameMenu>>,
            Query<&mut Node, With<FenInputMenu>>,
        )>,
    ) {
        set_menu_display(q.p0().transmute_lens(), Display::Flex);
        set_menu_display(q.p1().transmute_lens(), Display::Flex);
        set_menu_display(q.p2().transmute_lens(), Display::None);
    }

    pub(super) fn fen_input(
        mut fen_popup_state: ResMut<FenInputMenuState>,
        mut q: ParamSet<(
            Query<&mut Node, With<GameMenuDimLayer>>,
            Query<&mut Node, With<GameMenu>>,
            Query<&mut Node, With<FenInputMenu>>,
        )>,
    ) {
        fen_popup_state.reset();
        set_menu_display(q.p0().transmute_lens(), Display::Flex);
        set_menu_display(q.p1().transmute_lens(), Display::None);
        set_menu_display(q.p2().transmute_lens(), Display::Flex);
    }

    pub(super) fn game(mut q_menu: Query<&mut Node, With<GameMenuDimLayer>>) {
        set_menu_display(q_menu.transmute_lens(), Display::None);
    }
}

fn set_menu_display(mut q_lens_menu: QueryLens<&mut Node>, display: Display) {
    q_lens_menu.query().iter_mut().for_each(|mut node| node.display = display);
}
