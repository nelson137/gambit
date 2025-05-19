use bevy::{ecs::system::SystemState, prelude::*, ui::UiSystem};

use crate::{
    debug_name,
    utils::{NoopExts, ReparentInTag, SortIndex},
};

use super::{
    board::{CapturePlugin, CaptureState, PieceColor},
    consts::{CAPTURES_PANEL_HEIGHT, FONT_PATH, UI_GAP_VAL},
    ui::{spawn_ui, BoardAndPanelsContainer},
};

pub struct UiPanelsPlugin;

impl Plugin for UiPanelsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<CapturePlugin>() {
            panic!("Attempted to add plugin without required dependency: {CapturePlugin:?}");
        }

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
    commands.queue(black_panel.build(black_panel_entity));

    let white_panel = PanelBuilder {
        name: "Bottom Panel",
        index: 2,
        color: PieceColor::WHITE,
        profile_image_path: "images/profiles/white.png",
        profile_label: "White",
        margin: UiRect::top(UI_GAP_VAL),
    };
    let white_panel_entity = commands.spawn(white_panel.as_bundle()).id();
    commands.queue(white_panel.build(white_panel_entity));

    commands.reparent_in_tag::<BoardAndPanelsContainer>([black_panel_entity, white_panel_entity]);
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
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(CAPTURES_PANEL_HEIGHT),
                min_height: Val::Px(CAPTURES_PANEL_HEIGHT),
                margin: self.margin,
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
                ImageNode::new(profile_image_handle),
                Node { width: PROFILE_IMAGE_SIZE_VAL, height: PROFILE_IMAGE_SIZE_VAL, ..default() },
            ));

            cmds.spawn((
                PanelInnerContainer,
                debug_name!("Panel Inner Container"),
                Node {
                    height: Val::Percent(100.0),
                    margin: UiRect::left(UI_GAP_VAL),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    flex_grow: 1.0,
                    ..default()
                },
            ))
            .with_children(|cmds| {
                cmds.spawn((
                    debug_name!("Profile Label"),
                    Text(self.data.profile_label.to_string()),
                    TextFont { font: font.clone(), font_size: 12.0, ..default() },
                    TextColor(Color::WHITE),
                ));

                cmds.spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    margin: UiRect::top(UI_GAP_VAL),
                    ..default()
                })
                .with_children(|cmds| {
                    for cap_state in capture_state[color].iter_mut() {
                        let handle = cap_state.image_handles[0].clone();
                        cap_state.image_entity = cmds
                            .spawn((
                                CapturesImage,
                                ImageNode::new(handle),
                                Node {
                                    display: Display::None,
                                    margin: UiRect::right(UI_GAP_VAL),
                                    ..default()
                                },
                            ))
                            .id();
                    }

                    cmds.spawn((
                        MaterialAdvantageLabel(color),
                        Text("+6".to_string()),
                        TextFont { font, font_size: 12.0, ..default() },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                        Visibility::Hidden,
                    ));
                });
            });
        });

        state.apply(world);
    }
}

#[derive(Deref, Component)]
pub struct MaterialAdvantageLabel(PieceColor);

fn captures_images_sizes(
    image_assets: Res<Assets<Image>>,
    mut q_captures_images: Query<(&ImageNode, &mut Node, &ComputedNode), With<CapturesImage>>,
) {
    for (ImageNode { image, .. }, mut node, computed_node) in &mut q_captures_images {
        if let Some(img) = image_assets.get(image) {
            let image_size = img.size();
            let size = computed_node.size() * computed_node.inverse_scale_factor();
            let scale = size.y / image_size.y as f32;
            node.width = Val::Px(image_size.x as f32 * scale);
        }
    }
}
