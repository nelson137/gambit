use std::fmt;

use bevy::prelude::*;

use crate::{
    debug_name,
    utils::{AppPushOrderedStartupStages, StateExts},
};

use super::consts::{Z_GAME_MENU, Z_GAME_MENU_DIM_LAYER};

// ======================================================================
// Plugin
// ======================================================================

pub struct GameMenuPlugin;

impl Plugin for GameMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // States
            .add_state(MenuState::default())
            // Startup
            .push_ordered_startup_stages([
                (
                    SpawnStage::Phase1,
                    SystemStage::parallel()
                        .with_system(spawn_menu_dim_layer)
                        .with_system(spawn_menu),
                ),
                (SpawnStage::Phase2, SystemStage::single(spawn_menu_elements)),
            ])
            // Systems
            .add_system_set(SystemSet::on_enter(MenuState::Menu).with_system(on_enter_menu_state))
            .add_system_set(SystemSet::on_enter(MenuState::Game).with_system(on_enter_menu_state))
            .add_system(start_game_button);
    }
}

// ======================================================================
// State
// ======================================================================

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

fn on_enter_menu_state(
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

// ======================================================================
// Spawn
// ======================================================================

#[derive(Clone, StageLabel)]
enum SpawnStage {
    Phase1,
    Phase2,
}

#[derive(Component)]
pub struct GameMenuDimLayer;

const MENU_DIM_LAYER_COLOR: Color = Color::rgba(0.0, 0.0, 0.0, 0.7);

fn spawn_menu_dim_layer(mut commands: Commands) {
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
pub struct GameMenu;

/// `#302e2b`
const MENU_COLOR: Color = Color::rgba(
    0x30 as f32 / u8::MAX as f32,
    0x2e as f32 / u8::MAX as f32,
    0x2b as f32 / u8::MAX as f32,
    0.975,
);

fn spawn_menu(mut commands: Commands) {
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

#[derive(Component)]
struct StartGameButton;

/// `#7fa650`
const START_BUTTON_COLOR_DEFAULT: Color = Color::rgb(
    0x7f as f32 / u8::MAX as f32,
    0xa6 as f32 / u8::MAX as f32,
    0x50 as f32 / u8::MAX as f32,
);

/// `#8cb15e`
const START_BUTTON_COLOR_HOVER: Color = Color::rgb(
    0x8c as f32 / u8::MAX as f32,
    0xb1 as f32 / u8::MAX as f32,
    0x5e as f32 / u8::MAX as f32,
);

fn spawn_menu_elements(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_menu: Query<Entity, With<GameMenu>>,
) {
    let font = asset_server.load("fonts/montserrat-800.otf");
    const FONT_COLOR: Color = Color::WHITE;

    let title_entity = commands
        .spawn((
            debug_name!("Game Menu Title"),
            TextBundle {
                text: Text::from_section(
                    "Gambit",
                    TextStyle { font: font.clone(), font_size: 128.0, color: FONT_COLOR },
                ),
                ..default()
            },
        ))
        .id();

    const PAD_T_B: Val = Val::Px(8.0);
    const PAD_L_R: Val = Val::Px(16.0);
    let start_button_entity = commands
        .spawn((
            StartGameButton,
            debug_name!("Start Game Button"),
            ButtonBundle {
                background_color: START_BUTTON_COLOR_DEFAULT.into(),
                style: Style {
                    padding: UiRect {
                        top: PAD_T_B,
                        bottom: PAD_T_B,
                        left: PAD_L_R,
                        right: PAD_L_R,
                    },
                    border: UiRect::all(Val::Px(4.0)),
                    ..default()
                },
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

    commands.entity(q_menu.single()).push_children(&[title_entity, start_button_entity]);
}

// ======================================================================
// Systems
// ======================================================================

fn start_game_button(
    mut q_button: Query<
        (&Interaction, &mut BackgroundColor),
        (With<StartGameButton>, Changed<Interaction>),
    >,
    mut menu_state: ResMut<State<MenuState>>,
) {
    if let Ok((interaction, mut bg_color)) = q_button.get_single_mut() {
        match interaction {
            Interaction::Hovered => bg_color.0 = START_BUTTON_COLOR_HOVER,
            Interaction::Clicked => menu_state.transition(MenuState::Game),
            Interaction::None => bg_color.0 = START_BUTTON_COLOR_DEFAULT,
        }
    }
}
