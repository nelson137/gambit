use bevy::{
    ecs::system::{Command, SystemState},
    prelude::*,
    ui::UiSystem,
};

use crate::{debug_name, utils::AppNoop};

use super::{
    board::PieceColor,
    captures::CaptureState,
    consts::{CAPTURES_PANEL_HEIGHT, FONT_PATH, UI_GAP_VAL},
    ui::{spawn_ui, Ui},
    utils::SortIndex,
};

pub struct UiPanelsPlugin;

impl Plugin for UiPanelsPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            .add_systems(Startup, spawn_panels.after(spawn_ui))
            .add_systems(PostUpdate, captures_images_sizes.before(UiSystem::Layout))
            .noop();
    }
}

#[derive(Component)]
pub struct UiPanel;

fn spawn_panels(mut commands: Commands) {
    let black_panel = PanelBuilder {
        name: "Top Panel",
        index: 0,
        color: PieceColor::BLACK,
        profile_image_path: "images/profiles/black.png",
        profile_label: "Black",
        margin: UiRect::bottom(UI_GAP_VAL),
    };
    let black_panel_entity = commands.spawn(black_panel.as_bundle()).id();
    commands.add(black_panel.build(black_panel_entity));

    let white_panel = PanelBuilder {
        name: "Bottom Panel",
        index: 2,
        color: PieceColor::WHITE,
        profile_image_path: "images/profiles/white.png",
        profile_label: "White",
        margin: UiRect::top(UI_GAP_VAL),
    };
    let white_panel_entity = commands.spawn(white_panel.as_bundle()).id();
    commands.add(white_panel.build(white_panel_entity));

    commands.add(move |world: &mut World| {
        let parent = world.query_filtered::<Entity, With<Ui>>().single(world);
        world.entity_mut(parent).add_child(black_panel_entity).add_child(white_panel_entity);
    });
}

pub struct PanelBuilder {
    pub name: &'static str,
    pub index: usize,
    pub color: PieceColor,
    pub profile_image_path: &'static str,
    pub profile_label: &'static str,
    pub margin: UiRect,
}

impl PanelBuilder {
    pub fn as_bundle(&self) -> impl Bundle {
        let _name = self.name;
        (
            debug_name!(_name),
            UiPanel,
            SortIndex(self.index),
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Px(CAPTURES_PANEL_HEIGHT),
                    min_height: Val::Px(CAPTURES_PANEL_HEIGHT),
                    margin: self.margin,
                    ..default()
                },
                ..default()
            },
        )
    }

    pub fn build(self, entity: Entity) -> PanelBuilderCmd {
        PanelBuilderCmd { data: self, entity }
    }
}

pub struct PanelBuilderCmd {
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
    fn apply(self, world: &mut World) {
        let color = self.data.color;

        let asset_server = world.resource::<AssetServer>();
        let profile_image_handle = asset_server.load(self.data.profile_image_path);
        let font = asset_server.load(FONT_PATH);

        let mut state = SystemState::<(Commands, ResMut<CaptureState>)>::new(world);
        let (mut commands, mut capture_state) = state.get_mut(world);

        commands.entity(self.entity).with_children(|cmds| {
            cmds.spawn((
                ProfileImage,
                debug_name!("Profile Image"),
                ImageBundle {
                    image: UiImage::new(profile_image_handle),
                    style: Style {
                        width: PROFILE_IMAGE_SIZE_VAL,
                        height: PROFILE_IMAGE_SIZE_VAL,
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
                        height: Val::Percent(100.0),
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
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        margin: UiRect::top(UI_GAP_VAL),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|cmds| {
                    for cap_state in capture_state[color].iter_mut() {
                        let handles = &cap_state.image_handles;
                        cap_state.image_entity = cmds
                            .spawn((
                                CapturesImage,
                                ImageBundle {
                                    image: UiImage::new(handles[0].clone()),
                                    style: Style {
                                        display: Display::None,
                                        margin: UiRect::right(UI_GAP_VAL),
                                        ..default()
                                    },
                                    ..default()
                                },
                            ))
                            .id();
                    }
                });
            });
        });

        state.apply(world);
    }
}

fn captures_images_sizes(
    image_assets: Res<Assets<Image>>,
    mut q_captures_images: Query<(&UiImage, &Node, &mut Style), With<CapturesImage>>,
) {
    for (ui_image, node, mut style) in &mut q_captures_images {
        if let Some(img) = image_assets.get(&ui_image.texture) {
            let image_size = img.size();
            let size = node.size();
            let scale = size.y / image_size.y as f32;
            style.width = Val::Px(image_size.x as f32 * scale);
        }
    }
}
