use bevy::prelude::*;

use crate::{
    debug_name,
    utils::{NoopExts, SortIndex},
};

use super::{
    board::{BoardPlugin, CapturePlugin, PromotionPlugin},
    consts::{MIN_BOARD_SIZE, UI_GAP_VAL},
    eval_bar::EvaluationBarPlugin,
    menu::GameMenuUiPlugin,
    mouse::MouseUiPlugin,
    panels::UiPanelsPlugin,
};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            .add_plugins(GameMenuUiPlugin)
            .add_plugins(MouseUiPlugin)
            .add_plugins(BoardPlugin)
            .add_plugins(CapturePlugin)
            .add_plugins(UiPanelsPlugin)
            .add_plugins(EvaluationBarPlugin)
            .add_plugins(PromotionPlugin)
            .add_systems(Startup, spawn_ui)
            .noop();
    }
}

#[derive(Component)]
pub struct Ui;

#[derive(Component)]
pub struct EvaluationBarContainer;

#[derive(Component)]
pub struct BoardAndPanelsContainer;

#[derive(Component)]
#[require(SortIndex(1))]
pub struct BoardContainer;

pub fn spawn_ui(mut commands: Commands) {
    commands.spawn((
        debug_name!("Ui Wrapper"),
        Node {
            width: percent(100.0),
            height: percent(100.0),
            position_type: PositionType::Absolute,
            left: percent(0.0),
            top: percent(0.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            debug_name!("Ui"),
            Ui,
            Node {
                height: percent(100.0),
                padding: UI_GAP_VAL.all(),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            children![
                (
                    debug_name!("Evaluation Bar Container"),
                    EvaluationBarContainer,
                    Node {
                        margin: UI_GAP_VAL.right(),
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        row_gap: UI_GAP_VAL,
                        ..default()
                    },
                ),
                (
                    debug_name!("Board and Panels Container"),
                    BoardAndPanelsContainer,
                    Node { flex_direction: FlexDirection::Column, ..default() },
                    children![(
                        debug_name!("Board Container"),
                        BoardContainer,
                        Node {
                            flex_grow: 1.0,
                            min_width: MIN_BOARD_SIZE,
                            min_height: MIN_BOARD_SIZE,
                            ..default()
                        },
                    )],
                )
            ],
        )],
    ));
}
