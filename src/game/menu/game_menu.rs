use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    debug_name,
    game::consts::{
        FONT_PATH, INIT_MENU_BUTTON_TEXT_SIZE, INIT_MENU_HEIGHT, INIT_MENU_TITLE_SIZE,
        INIT_MENU_WIDTH, INIT_WIN_HEIGHT, INIT_WIN_WIDTH, MENU_HEIGHT_RATIO, MENU_WIDTH_RATIO,
        TITLE_FONT_PATH, z,
    },
    utils::{RoundToNearest, recolor_on, set_state_on},
};

use super::MenuState;

#[derive(Component)]
#[require(BackgroundColor(MENU_DIM_LAYER_COLOR), GlobalZIndex(z::G_MENU))]
pub(super) struct GameMenuDimLayer;

const MENU_DIM_LAYER_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.7);

pub fn spawn_menu_dim_layer(mut commands: Commands) {
    commands.spawn((
        debug_name!("Game Menu Dim Layer"),
        GameMenuDimLayer,
        Node {
            position_type: PositionType::Absolute,
            top: px(0.0),
            left: px(0.0),
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
    ));
}

#[derive(Component)]
#[require(BackgroundColor(MENU_COLOR))]
pub struct GameMenu;

/// `#302e2b`
const MENU_COLOR: Color = Color::srgba(
    0x30 as f32 / u8::MAX as f32,
    0x2e as f32 / u8::MAX as f32,
    0x2b as f32 / u8::MAX as f32,
    0.975,
);

pub(super) fn spawn_menu(
    mut commands: Commands,
    parent_entity: Single<Entity, With<GameMenuDimLayer>>,
) {
    let menu_entity = commands
        .spawn((
            debug_name!("Game Menu"),
            GameMenu,
            Node {
                width: px(INIT_MENU_WIDTH),
                height: px(INIT_MENU_HEIGHT),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .id();
    commands.entity(*parent_entity).add_child(menu_entity);
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

#[derive(Component)]
#[require(Text("Gambit".to_string()), TextColor(Color::WHITE))]
pub struct GameMenuTitle;

#[derive(Component)]
pub struct GameMenuButtonsContainer;

#[derive(Component)]
#[require(Text("Start".to_string()), TextColor(Color::WHITE))]
pub struct GameMenuButtonsText;

#[derive(Clone, Copy, Debug, Component)]
#[require(Button, BackgroundColor(BUTTON_COLOR_DEFAULT))]
pub(super) enum GameMenuButton {
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

pub fn spawn_menu_elements(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    menu: Single<Entity, With<GameMenu>>,
) {
    let font = asset_server.load(TITLE_FONT_PATH);
    let title_entity = commands
        .spawn((
            debug_name!("Game Menu Title"),
            GameMenuTitle,
            TextFont { font, font_size: INIT_MENU_TITLE_SIZE, ..default() },
            Node { margin: percent(4.0).top(), ..default() },
        ))
        .id();

    let buttons_container_entity = commands
        .spawn((
            debug_name!("Game Menu Buttons Container"),
            GameMenuButtonsContainer,
            Node {
                width: percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                ..default()
            },
        ))
        .id();

    commands.entity(*menu).add_children(&[title_entity, buttons_container_entity]);
}

pub fn spawn_menu_buttons(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    menu_buttons_container: Single<Entity, With<GameMenuButtonsContainer>>,
) {
    let button_node = Node {
        width: percent(50.0),
        padding: px(8.0).vertical(),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let font = asset_server.load(FONT_PATH);
    let text_font = TextFont { font, font_size: 48.0, ..default() };

    let start_button_entity = commands
        .spawn((
            debug_name!("Start Game Button"),
            GameMenuButton::Start,
            button_node.clone(),
            children![(
                debug_name!("Start Game Button Text"),
                GameMenuButtonsText,
                text_font.clone(),
            )],
        ))
        .observe(recolor_on::<_, Pointer<Over>, _>(BUTTON_COLOR_HOVER))
        .observe(recolor_on::<_, Pointer<Out>, _>(BUTTON_COLOR_DEFAULT))
        .observe(set_state_on::<_, MenuState, Pointer<Click>, _>(MenuState::Game))
        .id();

    let fen_button_entity = commands
        .spawn((
            debug_name!("Load FEN Button"),
            GameMenuButton::LoadFen,
            button_node,
            children![(
                debug_name!("Load FEN Button Text"),
                GameMenuButtonsText,
                Text("Load FEN".to_string()),
                text_font,
            )],
        ))
        .observe(recolor_on::<_, Pointer<Over>, _>(BUTTON_COLOR_HOVER))
        .observe(recolor_on::<_, Pointer<Out>, _>(BUTTON_COLOR_DEFAULT))
        .observe(set_state_on::<_, MenuState, Pointer<Click>, _>(MenuState::FenInput))
        .id();

    commands
        .entity(*menu_buttons_container)
        .add_children(&[start_button_entity, fen_button_entity]);
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
            text_font.font_size = stepped_font_size;
        }
    }

    q_text.p0().iter_mut().for_each(set_text_font_size_impl(scale * INIT_MENU_TITLE_SIZE));

    q_text.p1().iter_mut().for_each(set_text_font_size_impl(scale * INIT_MENU_BUTTON_TEXT_SIZE));
}
