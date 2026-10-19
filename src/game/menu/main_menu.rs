use bevy::{feathers, prelude::*, text};

use crate::utils::{GameFontWeight, GameInheritableFont, set_state_on, text_font};

use super::MenuState;

#[derive(Component, Clone, Default)]
pub(super) struct GameMenuDimLayer;

#[derive(Component, Clone, Default)]
pub(super) struct GameMenu;

#[derive(Component, Clone, Copy, Debug, Default)]
enum GameMenuButton {
    #[default]
    Start,
    LoadFen,
}

/// `#302e2b`
pub(super) const MENU_COLOR: Color = Color::srgba(
    0x30 as f32 / u8::MAX as f32,
    0x2e as f32 / u8::MAX as f32,
    0x2b as f32 / u8::MAX as f32,
    0.975,
);

/// `#7fa650`
pub(super) const BUTTON_COLOR_DEFAULT: Color = Color::srgb(
    0x7f as f32 / u8::MAX as f32,
    0xa6 as f32 / u8::MAX as f32,
    0x50 as f32 / u8::MAX as f32,
);

/// `#8cb25f`
///
/// 10% lighter than `BUTTON_COLOR_DEFAULT`.
pub(super) const BUTTON_COLOR_HOVER: Color = Color::srgb(
    0x8c as f32 / u8::MAX as f32,
    0xb2 as f32 / u8::MAX as f32,
    0x5f as f32 / u8::MAX as f32,
);

/// `#97ba6f`
///
/// 10% lighter than `BUTTON_COLOR_HOVER`.
pub(super) const BUTTON_COLOR_PRESSED: Color = Color::srgb(
    0x8c as f32 / u8::MAX as f32,
    0xb2 as f32 / u8::MAX as f32,
    0x5f as f32 / u8::MAX as f32,
);

pub(super) fn ui() -> impl Scene {
    bsn! {
        #GameMenu
        GameMenu
        feathers::containers::pane()
        Children [
            (
                feathers::containers::pane_body()
                feathers::theme::ThemeBackgroundColor({ super::tokens::GAME_MENU_BG })
                Node {
                    padding: { UiRect::axes(vh(6), vh(5)) },
                    border_radius: {
                        feathers::rounded_corners::RoundedCorners::All.to_border_radius(12.0)
                    },
                }
                Children [
                    (
                        Text("Gambit")
                        text_font(FontSize::Vh(16.), GameFontWeight::Bold)
                        template_value(text::LineHeight::RelativeToFont(1.))
                    ),
                    (
                        Node {
                            width: percent(100),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            margin: { vh(4).top() }
                        }
                        Children [
                            (
                                #StartGameButton
                                template_value(GameMenuButton::Start)
                                menu_btn("Start".into(), MenuState::Game)
                            ),
                            (
                                Node { height: vh(4) }
                            ),
                            (
                                #LoadFenButton
                                template_value(GameMenuButton::LoadFen)
                                menu_btn("Load FEN".into(), MenuState::FenInput)
                            ),
                        ]
                    ),
                ]
            ),
        ]
    }
}

fn menu_btn(text: String, target_menu_state: MenuState) -> impl Scene {
    bsn! {
        @feathers::controls::FeathersButton
        Node { width: percent(78), height: vh(10) }
        @GameInheritableFont { @font_size: FontSize::Vh(7.) }
        on(set_state_on::<_, MenuState, Pointer<Click>, _>(target_menu_state))
        Children [
            (Text(text) feathers::theme::ThemedText),
        ]
    }
}
