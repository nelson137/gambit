use bevy::prelude::*;

use crate::{
    debug_name,
    game::ui::{BoardContainer, UiPanel},
};

#[derive(Component)]
pub struct UiBoard;

#[derive(Debug)]
pub struct MoveHints {
    pub move_entity: Entity,
    pub capture_entity: Entity,
}

pub fn spawn_board(mut commands: Commands, q_container: Query<Entity, With<BoardContainer>>) {
    // let min_size = PANEL_HEIGHT * 2.0;
    let entity = commands
        .spawn((
            UiBoard,
            debug_name!("Board"),
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    // min_size: Size::new(min_size, min_size),
                    aspect_ratio: Some(1.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::WrapReverse,
                    ..default()
                },
                ..default()
            },
        ))
        .id();
    commands.entity(q_container.single()).add_child(entity);
}

pub fn board_size(
    windows: Res<Windows>,
    q_panels: Query<&Node, With<UiPanel>>,
    mut q_board: Query<&mut Style, With<BoardContainer>>,
) {
    let panels_height: f32 = q_panels.iter().map(|node| node.size().y).sum();
    if panels_height == 0.0 {
        return;
    }

    let Some(win) = windows.get_primary() else { return };
    let Ok(mut board_style) = q_board.get_single_mut() else { return };
    let size = win.width().min(win.height() - panels_height);
    board_style.size = Size::new(Val::Px(size), Val::Px(size));
}
