use bevy::prelude::*;

use crate::{debug_name, game::ui::Ui};

#[derive(Component)]
pub struct UiBoard;

#[derive(Debug)]
pub struct MoveHints {
    pub move_entity: Entity,
    pub capture_entity: Entity,
}

pub fn spawn_board(mut commands: Commands, q_ui: Query<Entity, With<Ui>>) {
    // let min_size = PANEL_HEIGHT * 2.0;
    let entity = commands
        .spawn((
            UiBoard,
            debug_name!("Board"),
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Auto, Val::Percent(100.0)),
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
    commands.entity(q_ui.single()).add_child(entity);
}
