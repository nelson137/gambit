use bevy::{
    ecs::system::{Command, SystemState},
    prelude::*,
    ui::FocusPolicy,
};

use crate::{
    assets::PieceColorAndTypeAssetPath,
    debug_name_f,
    game::{
        board::PromoteUiPiece,
        consts::{FONT_PATH, Z_PROMOTER},
        moves::MovePiece,
    },
};

use super::{BoardState, PieceColor, PieceType, Square, Tile};

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

fn set_promoter_visibility(world: &mut World, color: PieceColor, square: Option<Square>) {
    let mut state = SystemState::<(
        Commands,
        Res<BoardState>,
        Query<(Entity, &PromotionUi, &mut Visibility)>,
    )>::new(world);
    let (mut commands, board_state, mut q_promo_ui) = state.get_mut(world);

    for (entity, &PromotionUi(ui_color), mut vis) in &mut q_promo_ui {
        if ui_color == color {
            match square {
                Some(square) => {
                    commands.entity(entity).set_parent(board_state.tile(square));
                    *vis = Visibility::Visible;
                }
                None => *vis = Visibility::Hidden,
            }
            break;
        }
    }

    state.apply(world);
}

pub struct StartPromotion {
    entity: Entity,
    color: PieceColor,
    from_sq: Square,
    to_sq: Square,
}

impl StartPromotion {
    pub fn new(entity: Entity, color: PieceColor, from_sq: Square, to_sq: Square) -> Self {
        Self { entity, color, from_sq, to_sq }
    }
}

impl Command for StartPromotion {
    fn apply(self, world: &mut World) {
        let Self { entity, color, from_sq, to_sq } = self;
        trace!(?color, %from_sq, %to_sq, "Start promotion");

        // Hide the piece
        if let Some(mut vis) = world.entity_mut(entity).get_mut::<Visibility>() {
            *vis = Visibility::Hidden;
        }

        set_promoter_visibility(world, color, Some(to_sq));

        world.entity_mut(entity).insert(PromotingPiece::new(color, from_sq, to_sq));
    }
}

pub struct FinishPromotion {
    entity: Entity,
    color: PieceColor,
    from_sq: Square,
    to_sq: Square,
    event: PromotionEvent,
}

impl FinishPromotion {
    pub fn new(
        entity: Entity,
        color: PieceColor,
        from_sq: Square,
        to_sq: Square,
        event: PromotionEvent,
    ) -> Self {
        Self { entity, color, from_sq, to_sq, event }
    }
}

impl Command for FinishPromotion {
    fn apply(self, world: &mut World) {
        let Self { entity, color, from_sq, to_sq, event } = self;
        trace!(?color, %from_sq, %to_sq, ?event, "Finish promotion");

        set_promoter_visibility(world, color, None);

        match event {
            PromotionEvent::Promote(promo_typ) => {
                MovePiece::new(entity, color, PieceType::PAWN, from_sq, to_sq, Some(promo_typ))
                    .apply(world);
                PromoteUiPiece::new(entity, color, promo_typ).apply(world);
            }
            PromotionEvent::Cancel => {
                let from_sq_entity = world.resource_mut::<BoardState>().tile(from_sq);
                world.entity_mut(from_sq_entity).push_children(&[entity]);
            }
        }

        world.entity_mut(entity).remove::<PromotingPiece>();

        // Show the piece
        let mut e = world.entity_mut(entity);
        if let Some(mut vis) = e.get_mut::<Visibility>() {
            *vis = Visibility::Visible;
        }
    }
}

#[derive(Component)]
pub struct PromotingPiece {
    color: PieceColor,
    from_sq: Square,
    to_sq: Square,
}

impl PromotingPiece {
    pub fn new(color: PieceColor, from_sq: Square, to_sq: Square) -> Self {
        Self { color, from_sq, to_sq }
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
) {
    for (button, interaction) in &q_button {
        if let Interaction::Pressed = interaction {
            event_writer.send(PromotionEvent::Promote(button.1));
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
    mut event_reader: EventReader<PromotionEvent>,
    q_promo: Query<(Entity, &PromotingPiece)>,
) {
    let mut event_iter = event_reader.read();
    if let Some(event) = event_iter.next().copied() {
        // Exhaust the rest of the events.
        // Currently there is no easy way to scope click events to entities given the state of
        // `bevy_ui`. The easiest way to handle promotions and cancels is to always have the cancel
        // event fire on any click but run the system after the buttons system. That way, within 1
        // frame, there may be 2 events fired but only the first is used.
        event_iter.count();

        let Ok((entity, &PromotingPiece { color, from_sq, to_sq })) = q_promo.get_single() else {
            warn!("Ignoring received promotion event, not in promotion state");
            return;
        };

        commands.add(FinishPromotion::new(entity, color, from_sq, to_sq, event));
    }
}
