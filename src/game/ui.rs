use std::sync::Arc;

use bevy::{ecs::system::Command, prelude::*};

use crate::debug_name;

use super::{
    board::PieceColor,
    captures::CaptureState,
    consts::{FONT_PATH, MIN_BOARD_SIZE, UI_GAP_VAL},
};

#[derive(Component)]
pub struct Ui;

#[derive(Component)]
pub struct UiPanel;

#[derive(Component)]
pub struct TopPanelContainer;

#[derive(Component)]
pub struct BottomPanelContainer;

#[derive(Component)]
pub struct BoardContainer;

pub fn spawn_ui(mut commands: Commands) {
    commands
        .spawn((
            debug_name!("Ui Wrapper"),
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        left: Val::Percent(0.0),
                        top: Val::Percent(0.0),
                        ..default()
                    },
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
                        size: Size::new(Val::Auto, Val::Percent(100.0)),
                        padding: UiRect::all(UI_GAP_VAL),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                },
            ))
            .with_children(|cmds| {
                let black_panel = PanelBuilder {
                    name: "Top Panel",
                    color: PieceColor(chess::Color::Black),
                    profile_image_path: "images/profiles/black.png",
                    profile_label: "Black",
                    margin: UiRect::bottom(UI_GAP_VAL),
                };
                let black_panel_entity = cmds.spawn(black_panel.as_bundle()).id();
                cmds.add_command(black_panel.build(black_panel_entity));

                cmds.spawn((
                    BoardContainer,
                    debug_name!("Board Container"),
                    NodeBundle {
                        style: Style {
                            min_size: Size::new(Val::Px(MIN_BOARD_SIZE), Val::Px(MIN_BOARD_SIZE)),
                            ..default()
                        },
                        ..default()
                    },
                ));

                let white_panel = PanelBuilder {
                    name: "Bottom Panel",
                    color: PieceColor(chess::Color::White),
                    profile_image_path: "images/profiles/white.png",
                    profile_label: "White",
                    margin: UiRect::top(UI_GAP_VAL),
                };
                let white_panel_entity = cmds.spawn(white_panel.as_bundle()).id();
                cmds.add_command(white_panel.build(white_panel_entity));
            });
        });
}

struct PanelBuilder {
    name: &'static str,
    color: PieceColor,
    profile_image_path: &'static str,
    profile_label: &'static str,
    margin: UiRect,
}

const CAPTURES_PANEL_HEIGHT: f32 = 32.0;

impl PanelBuilder {
    fn as_bundle(&self) -> impl Bundle {
        let _name = self.name;
        (
            debug_name!(_name),
            UiPanel,
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Px(CAPTURES_PANEL_HEIGHT)),
                    min_size: Size::new(Val::Auto, Val::Px(CAPTURES_PANEL_HEIGHT)),
                    margin: self.margin,
                    ..default()
                },
                ..default()
            },
        )
    }

    fn build(self, entity: Entity) -> PanelBuilderCmd {
        PanelBuilderCmd { data: self, entity }
    }
}

struct PanelBuilderCmd {
    data: PanelBuilder,
    entity: Entity,
}

#[derive(Component)]
pub struct ProfileImage;

#[derive(Component)]
pub struct PanelInnerContainer;

#[derive(Component)]
pub struct CapturesImage;

const PROFILE_IMAGE_SIZE: f32 = CAPTURES_PANEL_HEIGHT;
const PROFILE_IMAGE_SIZE_VAL: Val = Val::Px(PROFILE_IMAGE_SIZE);

impl Command for PanelBuilderCmd {
    fn write(self, world: &mut World) {
        let color = self.data.color;

        let asset_server = world.resource::<AssetServer>();
        let profile_image_handle = asset_server.load(self.data.profile_image_path);
        let font = asset_server.load(FONT_PATH);

        let mut image_entities = Vec::with_capacity(5);

        let capture_state = Arc::clone(world.resource::<CaptureState>());
        world.entity_mut(self.entity).with_children(|cmds| {
            cmds.spawn((
                ProfileImage,
                debug_name!("Profile Image"),
                ImageBundle {
                    image: UiImage(profile_image_handle),
                    style: Style {
                        size: Size::new(PROFILE_IMAGE_SIZE_VAL, PROFILE_IMAGE_SIZE_VAL),
                        ..default()
                    },
                    ..default()
                },
            ));

            cmds.spawn((
                PanelInnerContainer,
                debug_name!("Panel Inner Container"),
                NodeBundle {
                    style: Style {
                        size: Size::new(Val::Auto, Val::Percent(100.0)),
                        margin: UiRect::left(UI_GAP_VAL),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        flex_grow: 1.0,
                        ..default()
                    },
                    ..default()
                },
            ))
            .with_children(|cmds| {
                let text_style = TextStyle { color: Color::WHITE, font, font_size: 14.0 };
                cmds.spawn((
                    debug_name!("Profile Label"),
                    TextBundle {
                        text: Text::from_section(self.data.profile_label, text_style),
                        ..default()
                    },
                ));

                cmds.spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        margin: UiRect::top(UI_GAP_VAL),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|cmds| {
                    for cap_state in capture_state[color].iter() {
                        let handles = &cap_state.image_handles;
                        let entity = cmds
                            .spawn((
                                CapturesImage,
                                ImageBundle {
                                    image: UiImage(handles[0].clone()),
                                    style: Style {
                                        display: Display::None,
                                        margin: UiRect::right(UI_GAP_VAL),
                                        ..default()
                                    },
                                    ..default()
                                },
                            ))
                            .id();
                        image_entities.push(entity);
                    }
                });
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

pub fn captures_images_sizes(
    image_assets: Res<Assets<Image>>,
    mut q_captures_images: Query<(&UiImage, &Node, &mut Style), With<CapturesImage>>,
) {
    for (ui_image, node, mut style) in &mut q_captures_images {
        let image_size = image_assets.get(&ui_image.0).unwrap().size();
        let size = node.size();
        let scale = size.y / image_size.y;
        style.size.width = Val::Px(image_size.x * scale);
    }
}
