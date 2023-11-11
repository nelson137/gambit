use bevy::{prelude::*, window::PrimaryWindow};

use crate::{
    debug_name,
    game::{consts::UI_GAP, panels::UiPanel, ui::BoardContainer},
};

#[derive(Component)]
pub struct UiBoard;

#[derive(Debug)]
pub struct MoveHints {
    pub move_entity: Entity,
    pub capture_entity: Entity,
}

pub fn spawn_board(mut commands: Commands) {
    let entity = commands
        .spawn((
            UiBoard,
            debug_name!("Board"),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    aspect_ratio: Some(1.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::WrapReverse,
                    ..default()
                },
                ..default()
            },
        ))
        .id();
    commands.add(move |world: &mut World| {
        let parent = world.query_filtered::<Entity, With<BoardContainer>>().single(world);
        world.entity_mut(parent).add_child(entity);
    });
}

pub fn board_size(
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_panels: Query<&Node, With<UiPanel>>,
    mut q_board: Query<&mut Style, With<BoardContainer>>,
) {
    let panels_height: f32 = q_panels.iter().map(|node| node.size().y + UI_GAP).sum();

    let Ok(win) = q_window.get_single() else { return };
    let Ok(mut board_style) = q_board.get_single_mut() else { return };

    let size = {
        let available_width = win.width() - 2.0 * UI_GAP;
        let available_height = win.height() - panels_height - 2.0 * UI_GAP;
        Val::Px(available_width.min(available_height))
    };
    board_style.width = size;
    board_style.height = size;
}
