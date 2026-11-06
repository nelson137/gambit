use bevy::{feathers, prelude::*};

use crate::utils::NoopExts;

use self::fen_popup::*;

pub use self::{game_menu::*, state::*};

mod fen_popup;
mod game_menu;
mod state;

pub struct GameMenuUiPlugin;

impl Plugin for GameMenuUiPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            .add_plugins(feathers::FeathersPlugins)
            .insert_resource(feathers::theme::UiTheme(create_theme()))
            .add_systems(Startup, game_menu.spawn())
            .noop();
    }
}

fn create_theme() -> feathers::theme::ThemeProps {
    let mut theme = feathers::dark_theme::create_dark_theme();
    theme.color.insert(tokens::GAME_MENU_BG, MENU_COLOR);
    theme.color.insert(feathers::tokens::BUTTON_BG, BUTTON_COLOR_DEFAULT);
    theme.color.insert(feathers::tokens::BUTTON_BG_HOVER, BUTTON_COLOR_HOVER);
    theme.color.insert(feathers::tokens::BUTTON_BG_PRESSED, BUTTON_COLOR_PRESSED);
    theme
}

mod tokens {
    use bevy::feathers::theme::ThemeToken;

    pub const GAME_MENU_BG: ThemeToken = ThemeToken::new_static("bevy.menu.body.bg");
}

pub struct GameMenuLogicPlugin;

impl Plugin for GameMenuLogicPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            // Resources
            .init_resource::<PopupState>()
            // States
            .init_state::<MenuState>()
            // Observers
            .add_observer(set_state_to_game_on_load)
            // Systems
            .add_systems(Startup, init_menu_state_from_cli)
            .add_systems(OnEnter(MenuState::FenInput), on_enter_menu_state_fen_input)
            .add_systems(OnEnter(MenuState::Menu), on_enter_menu_state_menu)
            .add_systems(OnEnter(MenuState::Game), on_enter_menu_state_game)
            .add_systems(Update, fen_menu.run_if(in_state(MenuState::FenInput)))
            .noop();
    }
}

#[cfg(test)]
pub mod test {
    use crate::utils::NoopExts;

    use super::*;

    pub struct TestMenuStateInGamePlugin;

    impl Plugin for TestMenuStateInGamePlugin {
        fn build(&self, app: &mut App) {
            app.noop()
                .init_state::<MenuState>()
                .add_systems(PreStartup, |mut s: ResMut<NextState<_>>| s.set(MenuState::Game))
                .noop();
        }
    }
}
