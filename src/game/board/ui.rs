use bevy::prelude::*;

use crate::{debug_name, game::ui::BoardContainer, utils::ReparentInTag};

#[derive(Component)]
pub struct UiBoard;

pub fn spawn_board(mut commands: Commands) {
    let entity = commands
        .spawn((
            UiBoard,
            debug_name!("Board"),
            NodeBundle {
                node: Node {
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
    commands.reparent_in_tag::<BoardContainer>([entity]);
}
