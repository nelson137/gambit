use bevy::{prelude::*, ui::UiSystem};
use bevy_startup_tree::{startup_tree, AddStartupTree};

use crate::debug_name;

use super::{
    board::{
        board_size, end_game_icon_size, spawn_board, spawn_end_game_icons, spawn_highlight_tiles,
        spawn_hints, spawn_pieces, spawn_promoters, spawn_tiles, BoardState,
    },
    consts::{MIN_BOARD_SIZE, UI_GAP_VAL},
    menu::GameMenuUiPlugin,
    mouse::MouseUiPlugin,
    panels::UiPanelsPlugin,
    utils::SortIndex,
};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GameMenuUiPlugin)
            .add_plugins(MouseUiPlugin)
            .add_plugins(UiPanelsPlugin)
            // Resources
            .init_resource::<BoardState>()
            // Systems
            .add_startup_tree(startup_tree! {
                spawn_ui => {
                    spawn_board => {
                        spawn_tiles => {
                            spawn_highlight_tiles,
                            spawn_hints,
                            spawn_pieces,
                            spawn_promoters,
                            spawn_end_game_icons,
                        },
                    },
                }
            })
            .add_systems(PostUpdate, (board_size, end_game_icon_size).before(UiSystem::Layout));
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
