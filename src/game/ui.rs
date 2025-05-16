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
pub struct BoardContainer;

pub fn spawn_ui(mut commands: Commands) {
    commands
        .spawn((
            debug_name!("Ui Wrapper"),
            NodeBundle {
                node: Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    position_type: PositionType::Absolute,
                    left: Val::Percent(0.0),
                    top: Val::Percent(0.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|cmds| {
            cmds.spawn((
                Ui,
                debug_name!("Ui"),
                NodeBundle {
                    node: Node {
                        height: Val::Percent(100.0),
                        padding: UiRect::all(UI_GAP_VAL),
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    ..default()
                },
            ))
            .with_children(|cmds| {
                cmds.spawn((
                    EvaluationBarContainer,
                    debug_name!("Evaluation Bar Container"),
                    NodeBundle {
                        node: Node {
                            margin: UiRect::right(UI_GAP_VAL),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            row_gap: UI_GAP_VAL,
                            ..default()
                        },
                        ..default()
                    },
                ));

                cmds.spawn((
                    BoardAndPanelsContainer,
                    debug_name!("Board and Panels Container"),
                    NodeBundle {
                        node: Node { flex_direction: FlexDirection::Column, ..default() },
                        ..default()
                    },
                ))
                .with_children(|cmds| {
                    cmds.spawn((
                        BoardContainer,
                        debug_name!("Board Container"),
                        SortIndex(1),
                        NodeBundle {
                            node: Node {
                                flex_grow: 1.0,
                                min_width: MIN_BOARD_SIZE,
                                min_height: MIN_BOARD_SIZE,
                                ..default()
                            },
                            ..default()
                        },
                    ));
                });
            });
        });
}
