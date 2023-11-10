use bevy::{
    ecs::system::{Command, SystemState},
    prelude::*,
    ui::UiSystem,
};
use bevy_startup_tree::{startup_tree, AddStartupTree};

use crate::debug_name;

use super::{
    board::{
        board_size, end_game_icon_size, spawn_board, spawn_end_game_icons, spawn_highlight_tiles,
        spawn_hints, spawn_pieces, spawn_promoters, spawn_tiles, BoardState, PieceColor,
    },
    captures::CaptureState,
    consts::{CAPTURES_PANEL_HEIGHT, FONT_PATH, MIN_BOARD_SIZE, UI_GAP_VAL},
    menu::GameMenuUiPlugin,
    mouse::MouseUiPlugin,
};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GameMenuUiPlugin)
            .add_plugins(MouseUiPlugin)
            // Resources
            .init_resource::<BoardState>()
            .init_resource::<CaptureState>()
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
            .add_systems(
                PostUpdate,
                (board_size, captures_images_sizes, end_game_icon_size).before(UiSystem::Layout),
            );
    }
}

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
                let black_panel = PanelBuilder {
                    name: "Top Panel",
                    color: PieceColor::BLACK,
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
                            min_width: MIN_BOARD_SIZE,
                            min_height: MIN_BOARD_SIZE,
                            ..default()
                        },
                        ..default()
                    },
                ));

                let white_panel = PanelBuilder {
                    name: "Bottom Panel",
                    color: PieceColor::WHITE,
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

impl PanelBuilder {
    fn as_bundle(&self) -> impl Bundle {
        let _name = self.name;
        (
            debug_name!(_name),
            UiPanel,
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
        // eprintln!("== get image {:?}", ui_image.0.id());
        if let Some(img) = image_assets.get(&ui_image.texture) {
            let image_size = img.size();
            let size = node.size();
            let scale = size.y / image_size.y as f32;
            style.width = Val::Px(image_size.x as f32 * scale);
        }
    }
}
