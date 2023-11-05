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
        menu::MenuState,
        moves::MovePiece,
    },
    utils::StateExts,
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
    let pos_top_left = UiRect { top: Val::Px(0.0), left: Val::Px(0.0), ..default() };

    for (color, flex_direction) in [
        (PieceColor::WHITE, FlexDirection::Column),
        (PieceColor::BLACK, FlexDirection::ColumnReverse),
    ] {
        let promo_entity = commands
            .spawn((
                PromotionUi(color),
                debug_name_f!("Promoter ({color})"),
                NodeBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: if let PieceColor::WHITE = color {
                            pos_top_left
                        } else {
                            UiRect { bottom: Val::Px(0.0), left: Val::Px(0.0), ..default() }
                        },
                        flex_direction,
                        size: Size::AUTO,
                        ..default()
                    },
                    visibility: Visibility::INVISIBLE,
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
                                image: UiImage(asset_server.load(asset_path)),
                                focus_policy: FocusPolicy::Pass,
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    position: pos_top_left,
                                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
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
                                size: Size::new(Val::Percent(100.0), Val::Auto),
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
                    vis.is_visible = true;
                }
                None => vis.is_visible = false,
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
    fn write(self, world: &mut World) {
        let Self { entity, color, from_sq, to_sq } = self;
        trace!(?color, %from_sq, %to_sq, "Start promotion");

        // Hide the piece
        if let Some(mut vis) = world.entity_mut(entity).get_mut::<Visibility>() {
            vis.is_visible = false;
        }

        set_promoter_visibility(world, color, Some(to_sq));

        world.resource_mut::<State<MenuState>>().transition(MenuState::GamePromotion {
            entity,
            color,
            from_sq,
            to_sq,
        });
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
    fn write(self, world: &mut World) {
        let Self { entity, color, from_sq, to_sq, event } = self;
        trace!(?color, %from_sq, %to_sq, ?event, "Finish promotion");

        set_promoter_visibility(world, color, None);

        match event {
            PromotionEvent::Promote(promo_typ) => {
                MovePiece::new(entity, color, PieceType::PAWN, from_sq, to_sq, Some(promo_typ))
                    .write(world);
                PromoteUiPiece::new(entity, color, promo_typ).write(world);
            }
            PromotionEvent::Cancel => {
                let from_sq_entity = world.resource_mut::<BoardState>().tile(from_sq);
                world.entity_mut(from_sq_entity).push_children(&[entity]);
            }
        }

        // Show the piece
        let mut e = world.entity_mut(entity);
        if let Some(mut vis) = e.get_mut::<Visibility>() {
            vis.is_visible = true;
        }

        world.resource_mut::<State<MenuState>>().transition(MenuState::Game);
    }
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

    let promo_piece_button_size = Size::new(Val::Px(tile_size.x), Val::Px(tile_size.y));
    let promo_cancel_button_size = Size::new(Val::Px(tile_size.x), Val::Px(tile_size.y / 2.0));

    for mut style in &mut button_set.p0() {
        style.size = promo_piece_button_size;
    }

    for mut style in &mut button_set.p1() {
        style.size = promo_cancel_button_size;
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PromotionEvent {
    Promote(PieceType),
    Cancel,
}

pub fn promotion_buttons(
    q_button: Query<(&PromotionButton, &Interaction), Changed<Interaction>>,
    mut event_writer: EventWriter<PromotionEvent>,
) {
    for (button, interaction) in &q_button {
        if let Interaction::Clicked = interaction {
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
    menu_state: Res<State<MenuState>>,
) {
    let mut event_iter = event_reader.iter();
    if let Some(event) = event_iter.next().copied() {
        // Exhaust the rest of the events.
        // Currently there is no easy way to scope click events to entities given the state of
        // `bevy_ui`. The easiest way to handle promotions and cancels is to always have the cancel
        // event fire on any click but run the system after the buttons system. That way, within 1
        // frame, there may be 2 events fired but only the first is used.
        event_iter.count();

        let &MenuState::GamePromotion { entity, color, from_sq, to_sq } = menu_state.current()
        else {
            warn!("Ignoring received promotion event, not in promotion state");
            return;
        };

        commands.add(FinishPromotion::new(entity, color, from_sq, to_sq, event));
    }
}
