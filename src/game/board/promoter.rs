use bevy::{
    prelude::*,
    ui::{FocusPolicy, UiSystem},
};

use crate::{
    assets::PieceColorAndTypeAssetPath,
    debug_name_f,
    game::{
        consts::{FONT_PATH, Z_PROMOTER},
        moves::MovePiece,
    },
    utils::AppNoop,
};

use super::{BoardState, PieceColor, PieceType, Square, Tile, UiPiece};

pub struct PromotionPlugin;

impl Plugin for PromotionPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            .add_systems(Update, start_promotion)
            .add_systems(Update, promotion_ui_sizes.run_if(is_promoting_piece))
            .add_systems(
                PreUpdate,
                (promotion_buttons, promotion_cancel_click_handler, promotion_event_handler)
                    .chain()
                    .in_set(PromoterSystem)
                    .run_if(is_promoting_piece)
                    .after(UiSystem::Focus),
            )
            .noop();
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, SystemSet)]
pub struct PromoterSystem;

#[derive(Component)]
pub struct PromotionUi(PieceColor);

#[derive(Component, Debug)]
pub struct PromotionButton(PieceColor, PieceType);

#[derive(Component)]
pub struct PromotionCancelButton;

pub fn spawn_promoters(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    board_state: Res<BoardState>,
) {
    for (color, flex_direction) in [
        (PieceColor::WHITE, FlexDirection::Column),
        (PieceColor::BLACK, FlexDirection::ColumnReverse),
    ] {
        let (left, top, bottom) = if let PieceColor::WHITE = color {
            (Val::Px(0.0), Val::Px(0.0), Val::Auto)
        } else {
            (Val::Px(0.0), Val::Auto, Val::Px(0.0))
        };
        let promo_entity = commands
            .spawn((
                PromotionUi(color),
                debug_name_f!("Promoter ({color})"),
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        left,
                        top,
                        bottom,
                        flex_direction,
                        ..default()
                    },
                    visibility: Visibility::Hidden,
                    z_index: ZIndex::Global(Z_PROMOTER),
                    ..default()
                },
            ))
            .with_children(|cmds| {
                const PROMO_TILE_COLOR: Color = Color::WHITE;

                for typ in [PieceType::QUEEN, PieceType::KNIGHT, PieceType::ROOK, PieceType::BISHOP]
                {
                    let asset_path = (color, typ).asset_path();

                    cmds.spawn((
                        debug_name_f!("Promotion Button ({color}) ({typ})"),
                        PromotionButton(color, typ),
                        ButtonBundle {
                            background_color: PROMO_TILE_COLOR.into(),
                            focus_policy: FocusPolicy::Block,
                            ..default()
                        },
                    ))
                    .with_children(|cmds| {
                        cmds.spawn((
                            debug_name_f!("Promotion Piece ({color}) ({typ})"),
                            ImageBundle {
                                image: UiImage::new(asset_server.load(asset_path)),
                                focus_policy: FocusPolicy::Pass,
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    top: Val::Px(0.0),
                                    left: Val::Px(0.0),
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                ..default()
                            },
                        ));
                    });
                }

                /// `#f1f1f1`
                const CANCEL_BUTTON_BG_COLOR: Color =
                    Color::rgb(0xf1 as f32 / 255.0, 0xf1 as f32 / 255.0, 0xf1 as f32 / 255.0);

                /// `#8b8987`
                const CANCEL_BUTTON_FG_COLOR: Color =
                    Color::rgb(0x8b as f32 / 255.0, 0x89 as f32 / 255.0, 0x87 as f32 / 255.0);

                cmds.spawn((
                    debug_name_f!("Promotion Cancel Button Wrapper ({color})"),
                    NodeBundle { style: Style { flex_direction, ..default() }, ..default() },
                ))
                .with_children(|cmds| {
                    cmds.spawn((
                        debug_name_f!("Promotion Cancel Button ({color})"),
                        // PromotionButton(color, None),
                        PromotionCancelButton,
                        // ButtonBundle {
                        NodeBundle {
                            background_color: CANCEL_BUTTON_BG_COLOR.into(),
                            style: Style {
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                width: Val::Percent(100.0),
                                ..default()
                            },
                            ..default()
                        },
                    ))
                    .with_children(|cmds| {
                        let font = asset_server.load(FONT_PATH);
                        let text_style =
                            TextStyle { font, font_size: 24.0, color: CANCEL_BUTTON_FG_COLOR };
                        cmds.spawn(TextBundle {
                            text: Text::from_section("x", text_style),
                            focus_policy: FocusPolicy::Block,
                            ..default()
                        });
                    });
                });
            })
            .id();

        let tile_entity = board_state.tile(Square::H8);
        commands.entity(tile_entity).add_child(promo_entity);
    }
}

pub fn start_promotion(
    mut commands: Commands,
    board_state: Res<BoardState>,
    mut q_added: Query<
        (&UiPiece, &PromotingPiece, &mut Visibility),
        (Added<PromotingPiece>, Without<PromotionUi>),
    >,
    mut q_promoters: Query<(Entity, &PromotionUi, &mut Visibility), Without<PromotingPiece>>,
) {
    for (&UiPiece { color, .. }, &PromotingPiece { from_sq, to_sq }, mut vis) in &mut q_added {
        trace!(?color, %from_sq, %to_sq, "Start promotion");

        *vis = Visibility::Hidden;

        if let Some((entity, _, mut vis)) =
            q_promoters.iter_mut().find(|(_, promo, _)| promo.0 == color)
        {
            commands.entity(entity).set_parent(board_state.tile(to_sq));
            *vis = Visibility::Visible;
        }
    }
}

#[derive(Component)]
pub struct PromotingPiece {
    from_sq: Square,
    to_sq: Square,
}

impl PromotingPiece {
    pub fn new(from_sq: Square, to_sq: Square) -> Self {
        Self { from_sq, to_sq }
    }
}

pub fn is_promoting_piece(q_promo: Query<(), With<PromotingPiece>>) -> bool {
    !q_promo.is_empty()
}

pub fn promotion_ui_sizes(
    q_tile: Query<&Node, With<Tile>>,
    mut button_set: ParamSet<(
        Query<&mut Style, With<PromotionButton>>,
        Query<&mut Style, With<PromotionCancelButton>>,
    )>,
) {
    let Some(tile_node) = q_tile.iter().next() else { return };
    let tile_size = tile_node.size();

    for mut style in &mut button_set.p0() {
        style.width = Val::Px(tile_size.x);
        style.height = Val::Px(tile_size.y);
    }

    for mut style in &mut button_set.p1() {
        style.width = Val::Px(tile_size.x);
        style.height = Val::Px(tile_size.y / 2.0);
    }
}

#[derive(Clone, Copy, Debug, Event)]
pub enum PromotionEvent {
    Promote(PieceType),
    Cancel,
}

pub fn promotion_buttons(
    q_button: Query<(&PromotionButton, &Interaction), Changed<Interaction>>,
    mut event_writer: EventWriter<PromotionEvent>,
    mut mouse_buttons: ResMut<Input<MouseButton>>,
) {
    for (button, interaction) in &q_button {
        if let Interaction::Pressed = interaction {
            event_writer.send(PromotionEvent::Promote(button.1));
            mouse_buttons.reset_all();
        }
    }
}

pub fn promotion_cancel_click_handler(
    mouse_buttons: Res<Input<MouseButton>>,
    mut event_writer: EventWriter<PromotionEvent>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        event_writer.send(PromotionEvent::Cancel);
    }
}

pub fn promotion_event_handler(
    mut commands: Commands,
    board_state: Res<BoardState>,
    asset_server: Res<AssetServer>,
    mut event_reader: EventReader<PromotionEvent>,
    mut q_promo: Query<
        (Entity, &UiPiece, &PromotingPiece, &mut Visibility, &mut UiImage),
        Without<PromotionUi>,
    >,
    mut q_promoters: Query<(&PromotionUi, &mut Visibility), Without<PromotingPiece>>,
) {
    let mut event_iter = event_reader.read();
    if let Some(event) = event_iter.next().copied() {
        // Exhaust the rest of the events.
        // Currently there is no easy way to scope click events to entities given the state of
        // `bevy_ui`. The easiest way to handle promotions and cancels is to always have the cancel
        // event fire on any click but run the system after the buttons system. That way, within 1
        // frame, there may be 2 events fired but only the first is used.
        event_iter.count();

        let Ok((
            entity,
            &UiPiece { color, .. },
            &PromotingPiece { from_sq, to_sq },
            mut vis,
            mut image,
        )) = q_promo.get_single_mut()
        else {
            warn!("Ignoring received promotion event, not in promotion state");
            return;
        };

        trace!(?color, %from_sq, %to_sq, ?event, "Finish promotion");

        let mut entity_cmds = commands.entity(entity);
        entity_cmds.remove::<PromotingPiece>();

        // Hide the promoter UI
        if let Some((_, mut vis)) = q_promoters.iter_mut().find(|(promo, _)| promo.0 == color) {
            *vis = Visibility::Hidden;
        }

        match event {
            PromotionEvent::Promote(promo_typ) => {
                let new_asset_path = (color, promo_typ).asset_path();
                image.texture = asset_server.load(new_asset_path);
                entity_cmds.insert(MovePiece::new(from_sq, to_sq, Some(promo_typ)));
            }
            PromotionEvent::Cancel => {
                // Re-parent piece back to its original square
                commands.entity(board_state.tile(from_sq)).push_children(&[entity]);
            }
        }

        // Show the piece
        *vis = Visibility::Visible;
    }
}
