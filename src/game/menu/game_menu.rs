use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    debug_name,
    game::consts::{
        FONT_PATH, INIT_MENU_BUTTON_TEXT_SIZE, INIT_MENU_HEIGHT, INIT_MENU_TITLE_SIZE,
        INIT_MENU_WIDTH, INIT_WIN_HEIGHT, INIT_WIN_WIDTH, MENU_HEIGHT_RATIO, MENU_WIDTH_RATIO,
        TITLE_FONT_PATH, Z_MENU,
    },
    utils::RoundToNearest,
};

use super::MenuState;

#[derive(Component)]
pub(super) struct GameMenuDimLayer;

const MENU_DIM_LAYER_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.7);

pub fn spawn_menu_dim_layer(mut commands: Commands) {
    commands.spawn((
        GameMenuDimLayer,
        debug_name!("Game Menu Dim Layer"),
        NodeBundle {
            background_color: MENU_DIM_LAYER_COLOR.into(),
            node: Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        },
        GlobalZIndex(Z_MENU),
    ));
}

#[derive(Component)]
pub struct GameMenu;

/// `#302e2b`
const MENU_COLOR: Color = Color::srgba(
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
                node: Node {
                    width: Val::Px(INIT_MENU_WIDTH),
                    height: Val::Px(INIT_MENU_HEIGHT),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceEvenly,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .id();
    commands.entity(parent_entity).add_child(menu_entity);
}

pub fn menu_size(
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_menu: Query<&mut Node, With<GameMenu>>,
) {
    let Ok(win) = q_window.get_single() else { return };
    let Ok(mut menu_node) = q_menu.get_single_mut() else { return };

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

    menu_node.width = Val::Px(width_stepped as f32);
    menu_node.height = Val::Px(height_stepped as f32);
}

#[derive(Component)]
pub struct GameMenuTitle;

#[derive(Component)]
pub struct GameMenuButtonsContainer;

#[derive(Component)]
pub struct GameMenuButtonsText;

#[derive(Clone, Copy, Debug, Component)]
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
    q_menu: Query<Entity, With<GameMenu>>,
) {
    let font = asset_server.load(TITLE_FONT_PATH);
    let title_entity = commands
        .spawn((
            GameMenuTitle,
            debug_name!("Game Menu Title"),
            Text("Gambit".to_string()),
            TextFont { font, font_size: INIT_MENU_TITLE_SIZE, ..default() },
            TextColor(Color::WHITE),
            Node { margin: UiRect::top(Val::Percent(4.0)), ..default() },
        ))
        .id();

    let buttons_container_entity = commands
        .spawn((
            GameMenuButtonsContainer,
            debug_name!("Game Menu Buttons Container"),
            NodeBundle {
                node: Node {
                    width: Val::Percent(100.0),
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

    commands.entity(q_menu.single()).add_children(&[title_entity, buttons_container_entity]);
}

pub fn spawn_menu_buttons(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_menu_buttons_container: Query<Entity, With<GameMenuButtonsContainer>>,
) {
    let button_node = Node {
        width: Val::Percent(50.0),
        padding: UiRect::vertical(Val::Px(8.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let font = asset_server.load(FONT_PATH);
    let text_font = TextFont { font, font_size: 48.0, ..default() };

    let start_button_entity = commands
        .spawn((
            GameMenuButton::Start,
            debug_name!("Start Game Button"),
            ButtonBundle {
                background_color: BUTTON_COLOR_DEFAULT.into(),
                node: button_node.clone(),
                ..default()
            },
        ))
        .with_children(|cmds| {
            cmds.spawn((
                debug_name!("Start Game Button Text"),
                GameMenuButtonsText,
                Text("Start".to_string()),
                text_font.clone(),
                TextColor(Color::WHITE),
            ));
        })
        .id();

    let fen_button_entity = commands
        .spawn((
            GameMenuButton::LoadFen,
            debug_name!("Load FEN Button"),
            ButtonBundle {
                background_color: BUTTON_COLOR_DEFAULT.into(),
                node: button_node,
                ..default()
            },
        ))
        .with_children(|cmds| {
            cmds.spawn((
                debug_name!("Load FEN Button Text"),
                GameMenuButtonsText,
                Text("Load FEN".to_string()),
                text_font,
            ));
        })
        .id();

    commands
        .entity(q_menu_buttons_container.single())
        .add_children(&[start_button_entity, fen_button_entity]);
}

pub(super) fn game_menu_elements_sizes(
    q_menu: Query<&ComputedNode, With<GameMenu>>,
    mut q_text: ParamSet<(
        Query<&mut TextFont, With<GameMenuTitle>>,
        Query<&mut TextFont, With<GameMenuButtonsText>>,
    )>,
) {
    let Ok(menu_computed_node) = q_menu.get_single() else { return };

    let menu_width = menu_computed_node.size().x;
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

pub(super) fn game_menu_buttons_hover(
    mut q_button: Query<(&Interaction, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (&interaction, mut bg_color) in &mut q_button {
        match interaction {
            Interaction::Hovered => bg_color.0 = BUTTON_COLOR_HOVER,
            Interaction::None => bg_color.0 = BUTTON_COLOR_DEFAULT,
            _ => {}
        }
    }
}

pub(super) fn game_menu_buttons(
    mut pressed_button: Local<Option<(GameMenuButton, Rect)>>,
    q_button: Query<
        (&GameMenuButton, &Interaction, &ComputedNode, &GlobalTransform),
        Changed<Interaction>,
    >,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
) {
    for (&button, &interaction, computed_node, global_transf) in &q_button {
        if let Interaction::Pressed = interaction {
            let size = computed_node.size();
            let pos = global_transf.translation().truncate() - size / 2.0;
            let button_area = Rect { min: pos, max: pos + size };
            *pressed_button = Some((button, button_area));
        }
    }

    let Ok(win) = q_window.get_single() else { return };
    let Some(mouse_pos) = win.cursor_position() else { return };
    let true = mouse_buttons.just_released(MouseButton::Left) else { return };
    let Some((pressed_button, pressed_button_area)) = pressed_button.take() else { return };

    if pressed_button_area.contains(mouse_pos) {
        next_menu_state.set(match pressed_button {
            GameMenuButton::Start => MenuState::Game,
            GameMenuButton::LoadFen => MenuState::FenInput,
        });
    }
}
