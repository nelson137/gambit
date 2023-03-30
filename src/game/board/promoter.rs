use bevy::{prelude::*, ui::FocusPolicy};

use crate::{
    assets::PieceColorAndTypeAssetPath,
    debug_name_f,
    game::consts::{FONT_PATH, Z_PROMOTER},
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

    for (color, flex_direction, square) in [
        (PieceColor::WHITE, FlexDirection::Column, chess::Square::F8),
        (PieceColor::BLACK, FlexDirection::ColumnReverse, chess::Square::C1),
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
                    visibility: Visibility::VISIBLE,
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

        let tile_entity = board_state.tile(Square::new(square));
        commands.entity(tile_entity).add_child(promo_entity);
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
