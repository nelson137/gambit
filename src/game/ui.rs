use std::sync::Arc;

use bevy::{ecs::system::Command, prelude::*};

use super::{board::PieceColor, captures::CaptureState};

#[derive(Component)]
pub struct Ui;

pub fn spawn_ui(mut commands: Commands) {
    commands.spawn((
        Ui,
        NodeBundle {
            style: Style {
                size: Size { width: Val::Percent(100.0), height: Val::Percent(100.0) },
                position_type: PositionType::Absolute,
                position: UiRect { left: Val::Percent(0.0), top: Val::Percent(0.0), ..default() },
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        },
    ));
}

pub fn spawn_panels(mut commands: Commands, q_ui: Query<Entity, With<Ui>>) {
    let container = commands
        .spawn(NodeBundle {
            style: Style {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .id();
    commands.entity(q_ui.single()).add_child(container);

    commands.add(PanelData { container, color: PieceColor(chess::Color::Black) });

    commands.add(PanelData { container, color: PieceColor(chess::Color::White) });
}

struct PanelData {
    container: Entity,
    color: PieceColor,
}

const CAPTURES_IMAGE_MARGIN: Val = Val::Px(8.0);

impl PanelData {
    fn into_bundle(self) -> impl Bundle {
        NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Px(32.0)),
                margin: UiRect {
                    top: CAPTURES_IMAGE_MARGIN,
                    bottom: CAPTURES_IMAGE_MARGIN,
                    ..default()
                },
                ..default()
            },
            ..default()
        }
    }
}

impl Command for PanelData {
    fn write(self, world: &mut World) {
        let color = self.color;

        let mut image_entities = Vec::with_capacity(5);

        let capture_state = Arc::clone(world.resource::<CaptureState>());
        world.entity_mut(self.container).with_children(|cmds| {
            cmds.spawn(self.into_bundle()).with_children(|cmds| {
                for cap_state in capture_state[color].iter() {
                    let handles = &cap_state.image_handles;
                    let entity = cmds
                        .spawn(ImageBundle {
                            image: UiImage(handles[handles.len() - 1].clone()),
                            style: Style {
                                display: Display::None,
                                margin: UiRect { left: CAPTURES_IMAGE_MARGIN, ..default() },
                                flex_shrink: 0.0,
                                ..default()
                            },
                            ..default()
                        })
                        .id();
                    image_entities.push(entity);
                }
            });
        });
        drop(capture_state);

        let mut capture_state = world.resource_mut::<CaptureState>();
        let state_entities = Arc::get_mut(&mut capture_state).unwrap();
        for (cap_state, entity) in state_entities[color].iter_mut().zip(image_entities) {
            cap_state.image_entity = entity;
        }
    }
}
