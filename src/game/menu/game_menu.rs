use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    game::consts::{
        INIT_MENU_BUTTON_TEXT_SIZE, INIT_MENU_HEIGHT, INIT_MENU_TITLE_SIZE, INIT_MENU_WIDTH,
        INIT_WIN_HEIGHT, INIT_WIN_WIDTH, MENU_HEIGHT_RATIO, MENU_WIDTH_RATIO, TITLE_FONT_PATH, z,
    },
    utils::{RoundToNearest, recolor_on, set_state_on, text_default_font, text_font},
};

use super::MenuState;

#[derive(Component, Clone, Default)]
pub(super) struct GameMenuDimLayer;

const MENU_DIM_LAYER_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.7);

pub fn game_menu() -> impl Scene {
    bsn! {
        menu_dim_layer()
        Children [
            menu()
        ]
    }
}

fn menu_dim_layer() -> impl Scene {
    bsn! {
        #GameMenuDimLayer
        GameMenuDimLayer
        BackgroundColor(MENU_DIM_LAYER_COLOR)
        GlobalZIndex(z::G_MENU)
        Node {
            position_type: PositionType::Absolute,
            top: px(0.0),
            left: px(0.0),
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
    }
}

#[derive(Component, Clone, Default)]
pub struct GameMenu;

/// `#302e2b`
const MENU_COLOR: Color = Color::srgba(
    0x30 as f32 / u8::MAX as f32,
    0x2e as f32 / u8::MAX as f32,
    0x2b as f32 / u8::MAX as f32,
    0.975,
);

fn menu() -> impl Scene {
    bsn! {
        #GameMenu
        GameMenu
        BackgroundColor(MENU_COLOR)
        Node {
            width: px(INIT_MENU_WIDTH),
            height: px(INIT_MENU_HEIGHT),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
        }
        Children [
            menu_title(),
            menu_buttons(),
        ]
    }
}

pub fn menu_size(
    win: Single<&Window, With<PrimaryWindow>>,
    mut menu_node: Single<&mut Node, With<GameMenu>>,
) {
    let win_width_scale = win.width() / INIT_WIN_WIDTH as f32;
    let win_height_scale = win.height() / INIT_WIN_HEIGHT as f32;
    let scale = win_width_scale.min(win_height_scale);

    let width_scale = scale * MENU_WIDTH_RATIO;
    let height_scale = scale * MENU_HEIGHT_RATIO;

    let width = (width_scale * INIT_WIN_WIDTH as f32) as u32;
    let height = (height_scale * INIT_WIN_HEIGHT as f32) as u32;

    const MENU_SIZE_STEP: u32 = 32;
    let width_stepped = width.round_to_nearest(MENU_SIZE_STEP);
    let height_stepped = height.round_to_nearest(MENU_SIZE_STEP);

    menu_node.width = px(width_stepped as f32);
    menu_node.height = px(height_stepped as f32);
}

#[derive(Component, Clone, Default)]
pub(super) struct GameMenuTitle;

#[derive(Component, Clone, Default)]
pub(super) struct GameMenuButtonsText;

#[derive(Component, Clone, Copy, Debug, Default)]
enum GameMenuButton {
    #[default]
    Start,
    LoadFen,
}

/// `#7fa650`
const BUTTON_COLOR_DEFAULT: Color = Color::srgb(
    0x7f as f32 / u8::MAX as f32,
    0xa6 as f32 / u8::MAX as f32,
    0x50 as f32 / u8::MAX as f32,
);

/// `#8cb15e`
const BUTTON_COLOR_HOVER: Color = Color::srgb(
    0x8c as f32 / u8::MAX as f32,
    0xb1 as f32 / u8::MAX as f32,
    0x5e as f32 / u8::MAX as f32,
);

fn menu_title() -> impl Scene {
    bsn! {
        #GameMenuTitle
        GameMenuTitle
        Text("Gambit")
        TextColor(Color::WHITE)
        text_font(TITLE_FONT_PATH, INIT_MENU_TITLE_SIZE)
        Node { margin: { percent(4.0).top() } }
    }
}

fn menu_buttons() -> impl Scene {
    bsn! {
        #GameMenuButtonsContainer
        Node {
            width: percent(100.0),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
        }
        Children [
            (
                #StartGameButton
                template_value(GameMenuButton::Start)
                menu_btn("Start".into(), MenuState::Game)
            ),
            (
                #LoadFenButton
                template_value(GameMenuButton::LoadFen)
                menu_btn("Load FEN".into(), MenuState::FenInput)
            ),
        ]
    }
}

fn menu_btn(text: String, target_menu_state: MenuState) -> impl Scene {
    bsn! {
        Button
        BackgroundColor(BUTTON_COLOR_DEFAULT)
        Node {
            width: percent(50.0),
            padding: { px(8.0).vertical() },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        on(recolor_on::<_, Pointer<Over>, _>(BUTTON_COLOR_HOVER))
        on(recolor_on::<_, Pointer<Out>, _>(BUTTON_COLOR_DEFAULT))
        on(set_state_on::<_, MenuState, Pointer<Click>, _>(target_menu_state))
        Children [
            (
                #GameMenuButtonText
                GameMenuButtonsText
                Text(text)
                text_default_font(48.)
                TextColor(Color::WHITE)
            )
        ]
    }
}

pub(super) fn game_menu_elements_sizes(
    menu_computed_node: Single<&ComputedNode, With<GameMenu>>,
    mut q_text: ParamSet<(
        Query<&mut TextFont, With<GameMenuTitle>>,
        Query<&mut TextFont, With<GameMenuButtonsText>>,
    )>,
) {
    let menu_width = menu_computed_node.size().x * menu_computed_node.inverse_scale_factor();
    let scale = menu_width / INIT_MENU_WIDTH;

    fn set_text_font_size_impl(font_size: f32) -> impl FnMut(Mut<TextFont>) {
        let stepped_font_size = (font_size as u32).round_to_nearest(8) as f32;
        move |mut text_font: Mut<TextFont>| {
            text_font.font_size = FontSize::Px(stepped_font_size);
        }
    }

    q_text.p0().iter_mut().for_each(set_text_font_size_impl(scale * INIT_MENU_TITLE_SIZE));

    q_text.p1().iter_mut().for_each(set_text_font_size_impl(scale * INIT_MENU_BUTTON_TEXT_SIZE));
}
