use bevy::prelude::*;

use crate::{
    debug_name,
    game::consts::{Z_GAME_MENU, Z_GAME_MENU_DIM_LAYER},
    utils::StateExts,
};

use super::MenuState;

#[derive(Component)]
pub(super) struct GameMenuDimLayer;

const MENU_DIM_LAYER_COLOR: Color = Color::rgba(0.0, 0.0, 0.0, 0.7);

pub(super) fn spawn_menu_dim_layer(mut commands: Commands) {
    commands.spawn((
        GameMenuDimLayer,
        debug_name!("Game Menu Dim Layer"),
        NodeBundle {
            background_color: MENU_DIM_LAYER_COLOR.into(),
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect { top: Val::Px(0.0), left: Val::Px(0.0), ..default() },
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                ..default()
            },
            z_index: ZIndex::Global(Z_GAME_MENU_DIM_LAYER),
            ..default()
        },
    ));
}

#[derive(Component)]
pub(super) struct GameMenu;

/// `#302e2b`
const MENU_COLOR: Color = Color::rgba(
    0x30 as f32 / u8::MAX as f32,
    0x2e as f32 / u8::MAX as f32,
    0x2b as f32 / u8::MAX as f32,
    0.975,
);

pub(super) fn spawn_menu(mut commands: Commands) {
    const SIZE: f32 = 60.0;
    const POS: Val = Val::Percent((100.0 - SIZE) / 2.0);

    commands.spawn((
        GameMenu,
        debug_name!("Game Menu"),
        NodeBundle {
            background_color: MENU_COLOR.into(),
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect { top: POS, left: POS, ..default() },
                size: Size::new(Val::Percent(SIZE), Val::Percent(SIZE)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceEvenly,
                align_items: AlignItems::Center,
                ..default()
            },
            z_index: ZIndex::Global(Z_GAME_MENU),
            ..default()
        },
    ));
}

const MENU_FONT_PATH: &str = "fonts/montserrat-800.otf";

#[derive(Component)]
pub(super) struct GameButtonsContainer;

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

pub(super) fn spawn_menu_elements(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_menu: Query<Entity, With<GameMenu>>,
) {
    let font = asset_server.load(MENU_FONT_PATH);
    let title_style = TextStyle { font, font_size: 128.0, color: Color::WHITE };
    let title_entity = commands
        .spawn((
            debug_name!("Game Menu Title"),
            TextBundle { text: Text::from_section("Gambit", title_style), ..default() },
        ))
        .id();

    let buttons_container_entity = commands
        .spawn((
            GameButtonsContainer,
            debug_name!("Game Menu Buttons Container"),
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    size: Size::new(Val::Percent(100.0), Val::Auto),
                    justify_content: JustifyContent::SpaceEvenly,
                    ..default()
                },
                ..default()
            },
        ))
        .id();

    commands.entity(q_menu.single()).push_children(&[title_entity, buttons_container_entity]);
}

pub(super) fn spawn_menu_buttons(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_menu_buttons_container: Query<Entity, With<GameButtonsContainer>>,
) {
    let font = asset_server.load(MENU_FONT_PATH);
    const FONT_COLOR: Color = Color::WHITE;
    const PAD_T_B: Val = Val::Px(8.0);
    const PAD_L_R: Val = Val::Px(16.0);
    let padding = UiRect { top: PAD_T_B, bottom: PAD_T_B, left: PAD_L_R, right: PAD_L_R };

    let start_button_entity = commands
        .spawn((
            GameMenuButton::Start,
            debug_name!("Start Game Button"),
            ButtonBundle {
                background_color: BUTTON_COLOR_DEFAULT.into(),
                style: Style { padding, border: UiRect::all(Val::Px(4.0)), ..default() },
                ..default()
            },
        ))
        .with_children(|cmds| {
            cmds.spawn(TextBundle {
                text: Text::from_section(
                    "Start",
                    TextStyle { font: font.clone(), font_size: 48.0, color: FONT_COLOR },
                ),
                ..default()
            });
        })
        .id();

    let fen_button_entity = commands
        .spawn((
            GameMenuButton::LoadFen,
            debug_name!("Load FEN Button"),
            ButtonBundle {
                background_color: BUTTON_COLOR_DEFAULT.into(),
                style: Style { padding, border: UiRect::all(Val::Px(4.0)), ..default() },
                ..default()
            },
        ))
        .with_children(|cmds| {
            cmds.spawn(TextBundle {
                text: Text::from_section(
                    "Load FEN",
                    TextStyle { font: font.clone(), font_size: 48.0, color: FONT_COLOR },
                ),
                ..default()
            });
        })
        .id();

    commands
        .entity(q_menu_buttons_container.single())
        .push_children(&[start_button_entity, fen_button_entity]);
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
