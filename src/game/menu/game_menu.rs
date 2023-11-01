use bevy::prelude::*;

use crate::{
    debug_name,
    game::consts::{
        FONT_PATH, INIT_MENU_BUTTON_TEXT_SIZE, INIT_MENU_HEIGHT, INIT_MENU_TITLE_SIZE,
        INIT_MENU_WIDTH, INIT_WIN_HEIGHT, INIT_WIN_WIDTH, MENU_HEIGHT_RATIO, MENU_WIDTH_RATIO,
        TITLE_FONT_PATH, Z_GAME_MENU, Z_GAME_MENU_DIM_LAYER,
    },
    utils::{RoundToNearest, StateExts},
};

use super::MenuState;

#[derive(Component)]
pub(super) struct GameMenuDimLayer;

const MENU_DIM_LAYER_COLOR: Color = Color::rgba(0.0, 0.0, 0.0, 0.7);

pub fn spawn_menu_dim_layer(mut commands: Commands) {
    commands.spawn((
        GameMenuDimLayer,
        debug_name!("Game Menu Dim Layer"),
        NodeBundle {
            background_color: MENU_DIM_LAYER_COLOR.into(),
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect { top: Val::Px(0.0), left: Val::Px(0.0), ..default() },
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            z_index: ZIndex::Global(Z_GAME_MENU_DIM_LAYER),
            ..default()
        },
    ));
}

#[derive(Component)]
pub struct GameMenu;

/// `#302e2b`
const MENU_COLOR: Color = Color::rgba(
    0x30 as f32 / u8::MAX as f32,
    0x2e as f32 / u8::MAX as f32,
    0x2b as f32 / u8::MAX as f32,
    0.975,
);

pub(super) fn spawn_menu(mut commands: Commands, q_parent: Query<Entity, With<GameMenuDimLayer>>) {
    let Ok(parent_entity) = q_parent.get_single() else { return };

    let menu_entity = commands
        .spawn((
            GameMenu,
            debug_name!("Game Menu"),
            NodeBundle {
                background_color: MENU_COLOR.into(),
                style: Style {
                    size: Size::new(Val::Px(INIT_MENU_WIDTH), Val::Px(INIT_MENU_HEIGHT)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,
                    ..default()
                },
                z_index: ZIndex::Global(Z_GAME_MENU),
                ..default()
            },
        ))
        .id();
    commands.entity(parent_entity).add_child(menu_entity);
}

pub fn menu_size(windows: Res<Windows>, mut q_menu: Query<&mut Style, With<GameMenu>>) {
    let Some(win) = windows.get_primary() else { return };
    let Ok(mut menu_style) = q_menu.get_single_mut() else { return };

    let win_width_scale = win.width() / INIT_WIN_WIDTH;
    let win_height_scale = win.height() / INIT_WIN_HEIGHT;
    let scale = win_width_scale.min(win_height_scale);

    let width_scale = scale * MENU_WIDTH_RATIO;
    let height_scale = scale * MENU_HEIGHT_RATIO;

    let width = (width_scale * INIT_WIN_WIDTH) as u32;
    let height = (height_scale * INIT_WIN_HEIGHT) as u32;

    const MENU_SIZE_STEP: u32 = 32;
    let width_stepped = width.round_to_nearest(MENU_SIZE_STEP);
    let height_stepped = height.round_to_nearest(MENU_SIZE_STEP);

    menu_style.size = Size::new(Val::Px(width_stepped as f32), Val::Px(height_stepped as f32));
}

#[derive(Component)]
pub struct GameMenuTitle;

#[derive(Component)]
pub struct GameMenuButtonsContainer;

#[derive(Component)]
pub struct GameMenuButtonsText;

#[derive(Component)]
pub(super) enum GameMenuButton {
    Start,
    LoadFen,
}

/// `#7fa650`
const BUTTON_COLOR_DEFAULT: Color = Color::rgb(
    0x7f as f32 / u8::MAX as f32,
    0xa6 as f32 / u8::MAX as f32,
    0x50 as f32 / u8::MAX as f32,
);

/// `#8cb15e`
const BUTTON_COLOR_HOVER: Color = Color::rgb(
    0x8c as f32 / u8::MAX as f32,
    0xb1 as f32 / u8::MAX as f32,
    0x5e as f32 / u8::MAX as f32,
);

pub fn spawn_menu_elements(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_menu: Query<Entity, With<GameMenu>>,
) {
    let font = asset_server.load(TITLE_FONT_PATH);
    let title_style = TextStyle { font, font_size: INIT_MENU_TITLE_SIZE, color: Color::WHITE };
    let margin = UiRect::top(Val::Percent(4.0));
    let title_entity = commands
        .spawn((
            GameMenuTitle,
            debug_name!("Game Menu Title"),
            TextBundle {
                text: Text::from_section("Gambit", title_style),
                style: Style { margin, ..default() },
                ..default()
            },
        ))
        .id();

    let buttons_container_entity = commands
        .spawn((
            GameMenuButtonsContainer,
            debug_name!("Game Menu Buttons Container"),
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Auto),
                    flex_grow: 1.0,
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    commands.entity(q_menu.single()).push_children(&[title_entity, buttons_container_entity]);
}

pub fn spawn_menu_buttons(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_menu_buttons_container: Query<Entity, With<GameMenuButtonsContainer>>,
) {
    let button_style = Style {
        size: Size::new(Val::Percent(50.0), Val::Auto),
        padding: UiRect::vertical(Val::Px(8.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let font = asset_server.load(FONT_PATH);
    let text_style = TextStyle { font, font_size: 48.0, color: Color::WHITE };

    let start_button_entity = commands
        .spawn((
            GameMenuButton::Start,
            debug_name!("Start Game Button"),
            ButtonBundle {
                background_color: BUTTON_COLOR_DEFAULT.into(),
                style: button_style.clone(),
                ..default()
            },
        ))
        .with_children(|cmds| {
            cmds.spawn((
                debug_name!("Start Game Button Text"),
                GameMenuButtonsText,
                TextBundle { text: Text::from_section("Start", text_style.clone()), ..default() },
            ));
        })
        .id();

    let fen_button_entity = commands
        .spawn((
            GameMenuButton::LoadFen,
            debug_name!("Load FEN Button"),
            ButtonBundle {
                background_color: BUTTON_COLOR_DEFAULT.into(),
                style: button_style,
                ..default()
            },
        ))
        .with_children(|cmds| {
            cmds.spawn((
                debug_name!("Load FEN Button Text"),
                GameMenuButtonsText,
                TextBundle { text: Text::from_section("Load FEN", text_style), ..default() },
            ));
        })
        .id();

    commands
        .entity(q_menu_buttons_container.single())
        .push_children(&[start_button_entity, fen_button_entity]);
}

pub(super) fn game_menu_elements_sizes(
    q_menu: Query<&Node, With<GameMenu>>,
    mut q_text: ParamSet<(
        Query<&mut Text, With<GameMenuTitle>>,
        Query<&mut Text, With<GameMenuButtonsText>>,
    )>,
) {
    let Ok(menu_node) = q_menu.get_single() else { return };

    let menu_width = menu_node.size().x;
    let scale = menu_width / INIT_MENU_WIDTH;

    fn set_text_font_size_impl(font_size: f32) -> impl FnMut(Mut<Text>) {
        let stepped_font_size = (font_size as u32).round_to_nearest(8) as f32;
        move |mut text: Mut<Text>| {
            for section in &mut text.sections {
                section.style.font_size = stepped_font_size;
            }
        }
    }

    q_text.p0().for_each_mut(set_text_font_size_impl(scale * INIT_MENU_TITLE_SIZE));

    q_text.p1().for_each_mut(set_text_font_size_impl(scale * INIT_MENU_BUTTON_TEXT_SIZE));
}

pub(super) fn game_menu_buttons(
    mut q_button: Query<
        (&GameMenuButton, &Interaction, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut menu_state: ResMut<State<MenuState>>,
) {
    if let Ok((button, interaction, mut bg_color)) = q_button.get_single_mut() {
        match interaction {
            Interaction::Hovered => bg_color.0 = BUTTON_COLOR_HOVER,
            Interaction::Clicked => match *button {
                GameMenuButton::Start => menu_state.transition(MenuState::Game),
                GameMenuButton::LoadFen => menu_state.transition_push(MenuState::FenInput),
            },
            Interaction::None => bg_color.0 = BUTTON_COLOR_DEFAULT,
        }
    }
}
