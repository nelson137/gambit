use bevy::prelude::*;

use crate::{
    debug_name,
    game::consts::CAPTURES_PANEL_HEIGHT,
    utils::{AppNoop, SortIndex},
};

use super::{
    board::{BoardPlugin, CapturePlugin, PromotionPlugin},
    consts::{MIN_BOARD_SIZE, UI_GAP_VAL},
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
            .add_plugins(PromotionPlugin)
            .add_systems(Startup, spawn_ui)
            .noop();
    }
}

#[derive(Component)]
pub struct Ui;

#[derive(Component)]
pub struct BoardAndPanelsContainer;

#[derive(Component)]
pub struct BoardContainer;

pub fn spawn_ui(mut commands: Commands) {
    commands
        .spawn((
            debug_name!("Ui Wrapper"),
            NodeBundle {
                style: Style {
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
                    style: Style {
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
                    debug_name!("Evaluation Bar Container"),
                    NodeBundle {
                        style: Style {
                            margin: UiRect::right(UI_GAP_VAL),
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            row_gap: UI_GAP_VAL,
                            ..default()
                        },
                        ..default()
                    },
                ))
                .with_children(|cmds| {
                    const SPACER_H: Val = Val::Px(CAPTURES_PANEL_HEIGHT);
                    let spacer_bundle = || NodeBundle {
                        style: Style { height: SPACER_H, flex_shrink: 0.0, ..default() },
                        ..default()
                    };

                    cmds.spawn((debug_name!("Evaluation Bar Spacer (Top)"), spacer_bundle()));

                    cmds.spawn((
                        debug_name!("Evaluation Bar"),
                        NodeBundle {
                            background_color: BackgroundColor(Color::rgb_u8(0x40, 0x3d, 0x39)),
                            style: Style {
                                position_type: PositionType::Relative,
                                width: Val::Px(20.0),
                                flex_grow: 1.0,
                                min_height: MIN_BOARD_SIZE,
                                ..default()
                            },
                            ..default()
                        },
                    ))
                    .with_children(|cmds| {
                        cmds.spawn((
                            debug_name!("Evaluation Bar White"),
                            NodeBundle {
                                background_color: BackgroundColor(Color::WHITE),
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    bottom: Val::Px(0.0),
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(50.0),
                                    ..default()
                                },
                                ..default()
                            },
                        ));
                    });

                    cmds.spawn((debug_name!("Evaluation Bar Spacer (Bottom)"), spacer_bundle()));
                });

                cmds.spawn((
                    BoardAndPanelsContainer,
                    debug_name!("Board and Panels Container"),
                    NodeBundle {
                        style: Style { flex_direction: FlexDirection::Column, ..default() },
                        ..default()
                    },
                ))
                .with_children(|cmds| {
                    cmds.spawn((
                        BoardContainer,
                        debug_name!("Board Container"),
                        SortIndex(1),
                        NodeBundle {
                            style: Style {
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
