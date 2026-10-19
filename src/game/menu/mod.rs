use bevy::{feathers, picking, prelude::*, ui};

use crate::{game::consts::z, utils::NoopExts};

use self::{fen_input_menu::*, main_menu::*};

pub use self::{fen_input_menu::FenInputMenuState, state::*};

mod fen_input_menu;
mod main_menu;
mod state;

pub struct GameMenuUiPlugin;

impl Plugin for GameMenuUiPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            .add_plugins(feathers::FeathersPlugins)
            .add_plugins(CustomButtonThemePlugin)
            .insert_resource(feathers::theme::UiTheme(create_theme()))
            .add_systems(Startup, menu_ui.spawn())
            .add_systems(Update, update_fen_text_input_invalid_styles)
            .noop();
    }
}

fn create_theme() -> feathers::theme::ThemeProps {
    let mut theme = feathers::dark_theme::create_dark_theme();
    theme.color.insert(feathers::tokens::BUTTON_BG, BUTTON_COLOR_DEFAULT);
    theme.color.insert(feathers::tokens::BUTTON_BG_HOVER, BUTTON_COLOR_HOVER);
    theme.color.insert(feathers::tokens::BUTTON_BG_PRESSED, BUTTON_COLOR_PRESSED);
    theme.color.insert(tokens::GAME_MENU_BG, MENU_COLOR);
    theme
}

mod tokens {
    use bevy::feathers::theme::ThemeToken;

    pub const GAME_MENU_BG: ThemeToken = ThemeToken::new_static("gambit.menu.body.bg");
}

struct CustomButtonThemePlugin;

impl Plugin for CustomButtonThemePlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            .configure_sets(
                PreUpdate,
                CustomButtonThemeSystems.after(picking::PickingSystems::Last),
            )
            .add_systems(
                PreUpdate,
                (update_custom_button_styles, update_custom_button_styles_remove)
                    .in_set(CustomButtonThemeSystems),
            )
            .add_observer(on_add_custom_button_theme)
            .noop();
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
struct CustomButtonThemeSystems;

#[derive(Component, Clone, Default)]
struct CustomButtonTheme(Color);

fn on_add_custom_button_theme(
    add: On<Add, CustomButtonTheme>,
    mut q_theme: Query<(&CustomButtonTheme, &mut BackgroundColor)>,
) {
    if let Ok((theme, mut bg)) = q_theme.get_mut(add.entity) {
        **bg = theme.0;
    }
}

fn update_custom_button_styles(
    q_buttons: Query<
        (
            Entity,
            Has<ui::InteractionDisabled>,
            Has<ui::Pressed>,
            &picking::hover::Hovered,
            &CustomButtonTheme,
        ),
        (
            With<CustomButtonTheme>,
            Or<(
                Changed<picking::hover::Hovered>,
                Added<ui::Pressed>,
                Added<ui::InteractionDisabled>,
            )>,
        ),
    >,
    mut commands: Commands,
) {
    for (button_ent, disabled, pressed, hovered, custom_theme) in q_buttons.iter() {
        set_custom_button_styles(
            button_ent,
            disabled,
            pressed,
            hovered.0,
            custom_theme,
            &mut commands,
        );
    }
}

fn update_custom_button_styles_remove(
    q_buttons: Query<
        (
            Entity,
            Has<ui::InteractionDisabled>,
            Has<ui::Pressed>,
            &picking::hover::Hovered,
            &CustomButtonTheme,
        ),
        With<CustomButtonTheme>,
    >,
    mut removed_disabled: RemovedComponents<ui::InteractionDisabled>,
    mut removed_pressed: RemovedComponents<ui::Pressed>,
    mut commands: Commands,
) {
    removed_disabled.read().chain(removed_pressed.read()).for_each(|ent| {
        if let Ok((button_ent, disabled, pressed, hovered, custom_theme)) = q_buttons.get(ent) {
            set_custom_button_styles(
                button_ent,
                disabled,
                pressed,
                hovered.0,
                custom_theme,
                &mut commands,
            );
        }
    });
}

fn set_custom_button_styles(
    button_ent: Entity,
    disabled: bool,
    pressed: bool,
    hovered: bool,
    custom_theme: &CustomButtonTheme,
    // font_color: &feathers::theme::InheritableThemeTextColor,
    commands: &mut Commands,
) {
    let bg_color: Color = match (disabled, pressed, hovered) {
        (true, _, _) => feathers::palette::GRAY_2,
        (false, true, _) => custom_theme.0.lighter(0.1),
        (false, false, true) => custom_theme.0.lighter(0.05),
        (false, false, false) => custom_theme.0,
    };

    // let font_color_token = match (variant, disabled) {
    //     (ButtonVariant::Primary, true) => tokens::BUTTON_PRIMARY_TEXT_DISABLED,
    //     (ButtonVariant::Primary, false) => tokens::BUTTON_PRIMARY_TEXT,
    //     (ButtonVariant::Normal | ButtonVariant::Plain, true) => tokens::BUTTON_TEXT_DISABLED,
    //     (ButtonVariant::Normal | ButtonVariant::Plain, false) => tokens::BUTTON_TEXT,
    // };

    let cursor_shape = match disabled {
        true => bevy::window::SystemCursorIcon::NotAllowed,
        false => bevy::window::SystemCursorIcon::Pointer,
    };

    commands.entity(button_ent).insert(BackgroundColor(bg_color));

    // // Change font color
    // if font_color.0 != font_color_token {
    //     commands.entity(button_ent).insert(InheritableThemeTextColor(font_color_token));
    // }

    // Change cursor shape
    commands.entity(button_ent).insert(feathers::cursor::EntityCursor::System(cursor_shape));
}

fn menu_ui() -> impl Scene {
    bsn! {
        menu_dim_layer()
        Children [
            main_menu::ui(),
            fen_input_menu::ui(),
        ]
    }
}

fn menu_dim_layer() -> impl Scene {
    bsn! {
        #GameMenuDimLayer
        GameMenuDimLayer
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7))
        GlobalZIndex(z::G_MENU)
        Node {
            position_type: PositionType::Absolute,
            top: px(0),
            left: px(0),
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
    }
}

pub struct GameMenuLogicPlugin;

impl Plugin for GameMenuLogicPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            // Resources
            .init_resource::<fen_input_menu::FenInputMenuState>()
            // States
            .init_state::<MenuState>()
            // Observers
            .add_observer(set_state_to_game_on_load)
            // Systems
            .add_systems(Startup, init_menu_state_from_cli)
            .add_systems(OnEnter(MenuState::Main), OnEnterMenuState::main)
            .add_systems(OnEnter(MenuState::FenInput), OnEnterMenuState::fen_input)
            .add_systems(OnEnter(MenuState::Game), OnEnterMenuState::game)
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
