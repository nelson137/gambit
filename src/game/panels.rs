use bevy::{prelude::*, ui::UiSystems};

use crate::utils::{NoopExts, text_default_font};

use super::{
    board::{CapturePlugin, CaptureState, PieceColor},
    consts::{CAPTURES_PANEL_HEIGHT, UI_GAP_VAL},
};

pub struct UiPanelsPlugin;

impl Plugin for UiPanelsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<CapturePlugin>() {
            panic!("Attempted to add plugin without required dependency: {CapturePlugin:?}");
        }

        app.noop()
            .add_systems(PostStartup, spawn_captures_images)
            .add_systems(PostUpdate, captures_images_sizes.before(UiSystems::Layout))
            .noop();
    }
}

#[derive(Component, Clone, Default)]
pub struct UiPanel;

#[derive(Component, Clone, Default)]
pub struct ProfileImage;

#[derive(Component, Clone, Default)]
pub struct PanelInnerContainer;

#[derive(Component, Clone, Default)]
pub struct CapturesImage;

const PROFILE_IMAGE_SIZE: f32 = CAPTURES_PANEL_HEIGHT;
const PROFILE_IMAGE_SIZE_VAL: Val = Val::Px(PROFILE_IMAGE_SIZE);

#[derive(Component, Clone, Default, Deref)]
pub struct MaterialAdvantageLabel(PieceColor);

pub fn black_panel() -> impl Scene {
    PanelBuilder {
        name: "Top Panel",
        color: PieceColor::BLACK,
        profile_image_path: "images/profiles/black.png",
        profile_label: "Black",
        margin: UI_GAP_VAL.bottom(),
    }
    .into_scene()
}

pub fn white_panel() -> impl Scene {
    PanelBuilder {
        name: "Bottom Panel",
        color: PieceColor::WHITE,
        profile_image_path: "images/profiles/white.png",
        profile_label: "White",
        margin: UI_GAP_VAL.top(),
    }
    .into_scene()
}

pub struct PanelBuilder {
    pub name: &'static str,
    pub color: PieceColor,
    pub profile_image_path: &'static str,
    pub profile_label: &'static str,
    pub margin: UiRect,
}

impl PanelBuilder {
    fn into_scene(self) -> impl Scene {
        let color = self.color;
        let profile_image_path = self.profile_image_path;

        bsn! {
            Name::new(self.name)
            UiPanel
            Node {
                width: percent(100.0),
                height: px(CAPTURES_PANEL_HEIGHT),
                min_height: px(CAPTURES_PANEL_HEIGHT),
                margin: { self.margin },
            }
            Children [
                (
                    #ProfileImage
                    ProfileImage
                    template(move |ctx| {
                        let asset_server = ctx.resource::<AssetServer>();
                        let profile_image_handle = asset_server.load(profile_image_path);
                        Ok(ImageNode::new(profile_image_handle))
                    })
                    Node { width: PROFILE_IMAGE_SIZE_VAL, height: PROFILE_IMAGE_SIZE_VAL }
                ),
                (
                    #PanelInnerContainer
                    PanelInnerContainer
                    Node {
                        height: percent(100.0),
                        margin: { UI_GAP_VAL.left() },
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        flex_grow: 1.0,
                    }
                    Children [
                        (
                            #ProfileLabel
                            Text({ self.profile_label.to_string() })
                            text_default_font(12.)
                            TextColor(Color::WHITE)
                        ),
                        (
                            #CapturesImagesContainer
                            Node {
                                width: percent(100.0),
                                height: percent(100.0),
                                margin: { UI_GAP_VAL.top() },
                            }
                            template(move |ctx| {
                                Ok(SpawnCapturesImages {
                                    entity: ctx.entity.id(),
                                    color
                                })
                            })
                        )
                    ]
                )
            ]
        }
    }
}

#[derive(Component)]
struct SpawnCapturesImages {
    entity: Entity,
    color: PieceColor,
}

fn spawn_captures_images(
    mut commands: Commands,
    mut capture_state: ResMut<CaptureState>,
    q_spawn_captures_images: Query<&SpawnCapturesImages>,
) {
    for spawn in &q_spawn_captures_images {
        for cap_state in capture_state[spawn.color].iter_mut() {
            let image = cap_state.image_handles[0].clone();
            cap_state.image_entity = commands
                .spawn_scene(bsn! {
                    CapturesImage
                    ChildOf({ spawn.entity })
                    ImageNode{ image }
                    Node { display: Display::None, margin: { UI_GAP_VAL.right() } }
                })
                .id();
        }

        commands.spawn_scene(bsn! {
            MaterialAdvantageLabel({ spawn.color })
            ChildOf({ spawn.entity })
            text_default_font(12.)
            Text("+6")
            TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5))
            Visibility::Hidden
        });
    }
}

fn captures_images_sizes(
    image_assets: Res<Assets<Image>>,
    mut q_captures_images: Query<(&ImageNode, &mut Node, &ComputedNode), With<CapturesImage>>,
) {
    for (ImageNode { image, .. }, mut node, computed_node) in &mut q_captures_images {
        if let Some(img) = image_assets.get(image) {
            let image_size = img.size();
            let size = computed_node.size() * computed_node.inverse_scale_factor();
            let scale = size.y / image_size.y as f32;
            node.width = px(image_size.x as f32 * scale);
        }
    }
}
