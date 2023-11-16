use bevy::prelude::*;

use crate::{debug_name, utils::AppNoop};

use super::{
    board::{BoardPlugin, CapturePlugin, PromotionPlugin},
    consts::{MIN_BOARD_SIZE, UI_GAP_VAL},
    menu::GameMenuUiPlugin,
    mouse::MouseUiPlugin,
    panels::UiPanelsPlugin,
    utils::SortIndex,
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
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
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
                            min_width: MIN_BOARD_SIZE,
                            min_height: MIN_BOARD_SIZE,
                            ..default()
                        },
                        ..default()
                    },
                ));
            });
        });
}
