use bevy::{
    ecs::component::{ComponentHooks, StorageType},
    prelude::*,
    ui::{FocusPolicy, UiSystem},
};

use crate::{
    debug_name_f,
    game::{
        board::{MovePiece, SelectionEvent},
        consts::{FONT_PATH, Z_PROMOTER},
    },
    utils::{hook, NoopExts},
};

use super::{BoardState, PieceColor, PieceMeta, PieceType, Square, Tile};

pub struct PromotionPlugin;

impl Plugin for PromotionPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            .add_systems(Update, promotion_ui_sizes)
            .add_systems(
                PreUpdate,
                promotion_click_handler
                    .pipe(promotion_result_handler)
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
pub struct PromotionButton(PieceType);

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
                    node: Node {
                        position_type: PositionType::Absolute,
                        left,
                        top,
                        bottom,
                        flex_direction,
                        ..default()
                    },
                    visibility: Visibility::Hidden,
                    ..default()
                },
                GlobalZIndex(Z_PROMOTER),
            ))
            .with_children(|cmds| {
                const PROMO_TILE_COLOR: Color = Color::WHITE;

                for typ in [PieceType::QUEEN, PieceType::KNIGHT, PieceType::ROOK, PieceType::BISHOP]
                {
                    let asset_path = PieceMeta::new(color, typ).asset_path();

                    cmds.spawn((
                        debug_name_f!("Promotion Button ({color}) ({typ})"),
                        PromotionButton(typ),
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
                                image: ImageNode::new(asset_server.load(asset_path)),
                                focus_policy: FocusPolicy::Pass,
                                node: Node {
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
                    Color::srgb(0xf1 as f32 / 255.0, 0xf1 as f32 / 255.0, 0xf1 as f32 / 255.0);

                /// `#8b8987`
                const CANCEL_BUTTON_FG_COLOR: Color =
                    Color::srgb(0x8b as f32 / 255.0, 0x89 as f32 / 255.0, 0x87 as f32 / 255.0);

                cmds.spawn((
                    debug_name_f!("Promotion Cancel Button Wrapper ({color})"),
                    NodeBundle { node: Node { flex_direction, ..default() }, ..default() },
                ))
                .with_children(|cmds| {
                    cmds.spawn((
                        debug_name_f!("Promotion Cancel Button ({color})"),
                        PromotionCancelButton,
                        ButtonBundle {
                            background_color: CANCEL_BUTTON_BG_COLOR.into(),
                            focus_policy: FocusPolicy::Block,
                            node: Node {
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
                        cmds.spawn((
                            Text("x".to_string()),
                            TextFont { font, font_size: 24.0, ..default() },
                            TextColor(CANCEL_BUTTON_FG_COLOR),
                        ));
                    });
                });
            })
            .id();

        let tile_entity = board_state.tile(Square::H8);
        commands.entity(tile_entity).add_child(promo_entity);
    }
}

#[derive(Clone, Copy)]
pub struct PromotingPiece {
    from_sq: Square,
    to_sq: Square,
}

impl PromotingPiece {
    pub fn new(from_sq: Square, to_sq: Square) -> Self {
        Self { from_sq, to_sq }
    }
}

impl Component for PromotingPiece {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks
            .noop()
            .on_add(hook!(PromotingPiece => start_promotion))
            .on_remove(hook!(PromotingPiece => end_promotion))
            .noop();
    }
}

pub fn start_promotion(
    In((piece, promoting)): In<(Entity, PromotingPiece)>,
    mut commands: Commands,
    board_state: Res<BoardState>,
    q_piece_meta: Query<&PieceMeta>,
    mut q_visibility: Query<&mut Visibility>,
    mut q_promoters: Query<(Entity, &PromotionUi)>,
) {
    let PromotingPiece { from_sq, to_sq } = promoting;
    let Ok(&PieceMeta { color, .. }) = q_piece_meta.get(piece) else { return };
    let Ok(mut vis) = q_visibility.get_mut(piece) else { return };

    trace!(?color, %from_sq, %to_sq, "Start promotion");

    // Clear selection & hints
    commands.trigger(SelectionEvent::Unselect);

    // Hide the piece
    *vis = Visibility::Hidden;

    // Show the promoter UI
    if let Some((entity, _)) = q_promoters.iter_mut().find(|(_, promo)| promo.0 == color) {
        commands.entity(entity).set_parent(board_state.tile(to_sq));
        if let Ok(mut vis) = q_visibility.get_mut(entity) {
            *vis = Visibility::Visible;
        }
    }
}

pub fn end_promotion(
    In((piece, promoting)): In<(Entity, PromotingPiece)>,
    q_piece_meta: Query<&PieceMeta>,
    mut q_visibility: Query<&mut Visibility>,
    mut q_promoters: Query<(Entity, &PromotionUi)>,
) {
    let PromotingPiece { from_sq, to_sq } = promoting;
    let Ok(&PieceMeta { color, .. }) = q_piece_meta.get(piece) else { return };
    let Ok(mut piece_vis) = q_visibility.get_mut(piece) else { return };

    trace!(?color, %from_sq, %to_sq, "End promotion");

    // Show the piece
    *piece_vis = Visibility::Visible;

    // Hide the promoter UI
    if let Some((entity, _)) = q_promoters.iter_mut().find(|(_, promo)| promo.0 == color) {
        if let Ok(mut vis) = q_visibility.get_mut(entity) {
            *vis = Visibility::Hidden;
        }
    }
}

pub fn is_promoting_piece(q_promo: Query<(), With<PromotingPiece>>) -> bool {
    !q_promo.is_empty()
}

type PromoButtonD<'a> = (&'a ViewVisibility, &'a mut Node);

fn promotion_ui_sizes(
    q_tile: Query<&ComputedNode, With<Tile>>,
    mut q_node: Query<PromoButtonD>,
    mut q_promo_button: Query<(), With<PromotionButton>>,
    mut q_cancel_buttons: Query<(), With<PromotionCancelButton>>,
) {
    let Some(tile_computed_node) = q_tile.iter().next() else { return };
    let tile_size = tile_computed_node.size();

    let mut lens = q_promo_button.join::<PromoButtonD, PromoButtonD>(&mut q_node);
    for (_, mut node) in lens.query().iter_mut().filter(|(vis, _)| vis.get()) {
        node.width = Val::Px(tile_size.x);
        node.height = Val::Px(tile_size.y);
    }

    let mut lens = q_cancel_buttons.join::<PromoButtonD, PromoButtonD>(&mut q_node);
    for (_, mut node) in lens.query().iter_mut().filter(|(vis, _)| vis.get()) {
        node.width = Val::Px(tile_size.x);
        node.height = Val::Px(tile_size.y / 2.0);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PromotionResult {
    Promote(PieceType),
    Cancel,
}

pub fn promotion_click_handler(
    q_buttons: Query<(&PromotionButton, &Interaction), Changed<Interaction>>,
    q_cancel_buttons: Query<(&PromotionCancelButton, &Interaction), Changed<Interaction>>,
    mut mouse_buttons: ResMut<ButtonInput<MouseButton>>,
) -> Option<PromotionResult> {
    if let Some((button, _)) = q_buttons.iter().find(|(_, i)| **i == Interaction::Pressed) {
        mouse_buttons.reset_all();
        return Some(PromotionResult::Promote(button.0));
    }

    if q_cancel_buttons.iter().any(|(_, i)| *i == Interaction::Pressed) {
        mouse_buttons.reset_all();
        return Some(PromotionResult::Cancel);
    }

    if mouse_buttons.just_pressed(MouseButton::Left) {
        mouse_buttons.reset_all();
        return Some(PromotionResult::Cancel);
    }

    None
}

pub fn promotion_result_handler(
    In(promotion_result): In<Option<PromotionResult>>,
    mut commands: Commands,
    board_state: Res<BoardState>,
    q_promo: Query<(Entity, &PieceMeta, &PromotingPiece), Without<PromotionUi>>,
) {
    let Some(result) = promotion_result else { return };

    let Ok((entity, &PieceMeta { color, .. }, &PromotingPiece { from_sq, to_sq })) =
        q_promo.get_single()
    else {
        warn!("Ignoring promotion event as no piece is awaiting promotion");
        return;
    };

    trace!(?color, %from_sq, %to_sq, ?result, "Finish promotion");

    commands.entity(entity).remove::<PromotingPiece>();

    match result {
        PromotionResult::Promote(promo_typ) => {
            // Move the piece
            commands
                .trigger_targets(MovePiece::new(from_sq, to_sq, Some(promo_typ), false), entity);
        }
        PromotionResult::Cancel => {
            // Re-parent piece back to its original square
            commands.entity(board_state.tile(from_sq)).add_children(&[entity]);
        }
    }
}
