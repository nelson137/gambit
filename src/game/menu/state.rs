use std::fmt;

use bevy::prelude::*;

use super::{GameMenu, GameMenuDimLayer};

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub enum MenuState {
    #[default]
    Menu,
    Game,
}

impl fmt::Display for MenuState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub(super) fn on_enter_menu_state(
    menu_state: ResMut<State<MenuState>>,
    mut q_menu_components: Query<&mut Style, Or<(With<GameMenuDimLayer>, With<GameMenu>)>>,
) {
    let mut set_menu_display =
        |d| q_menu_components.iter_mut().for_each(|mut style| style.display = d);
    match menu_state.current() {
        MenuState::Menu => set_menu_display(Display::Flex),
        MenuState::Game => set_menu_display(Display::None),
    }
}
