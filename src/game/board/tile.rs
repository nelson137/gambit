use bevy::prelude::*;
use chess::{File, Rank};

use crate::{
    debug_name_f,
    game::consts::{FONT_PATH, Z_NOTATION_TEXT, Z_TILE},
};

use super::{BoardState, Square, UiBoard};

#[derive(Component)]
pub struct Tile;

/// The "black" bord color.
///
/// `#769656`
pub const COLOR_BLACK: Color = Color::rgb(
    0x76 as f32 / u8::MAX as f32,
    0x96 as f32 / u8::MAX as f32,
    0x56 as f32 / u8::MAX as f32,
);

/// The "white" bord color.
///
/// `#eeeed2`
pub const COLOR_WHITE: Color = Color::rgb(
    0xee as f32 / u8::MAX as f32,
    0xee as f32 / u8::MAX as f32,
    0xd2 as f32 / u8::MAX as f32,
);

pub fn spawn_tiles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut board_state: ResMut<BoardState>,
    q_board: Query<Entity, With<UiBoard>>,
) {
    let board = q_board.single();
    let font = asset_server.load(FONT_PATH);
    let pos_top_left = UiRect { top: Val::Px(0.0), left: Val::Px(0.0), ..default() };
    let file_label_margins =
        UiRect { bottom: Val::Percent(3.5), right: Val::Percent(8.0), ..default() };
    let rank_label_margins =
        UiRect { top: Val::Percent(1.0), left: Val::Percent(4.5), ..default() };

    for square in chess::ALL_SQUARES.map(Square::new) {
        let file = square.get_file();
        let rank = square.get_rank();

        let file_rank_sum = rank.to_index() + file.to_index();
        let color = if file_rank_sum % 2 == 0 { COLOR_BLACK } else { COLOR_WHITE };

        let tile_entity = commands
            .spawn((
                Tile,
                debug_name_f!("Tile ({square})"),
                square,
                NodeBundle {
                    background_color: color.into(),
                    style: Style {
                        position_type: PositionType::Relative,
                        size: Size::new(Val::Percent(100.0 / 8.0), Val::Percent(100.0 / 8.0)),
                        ..default()
                    },
                    z_index: ZIndex::Local(Z_TILE),
                    ..default()
                },
            ))
            .with_children(|cmds| {
                pub const BOARD_TEXT_FONT_SIZE: f32 = 20.0;

                // File markers
                if square.get_rank() == Rank::First {
                    let text_style = TextStyle {
                        color: if file.to_index() % 2 == 0 { COLOR_WHITE } else { COLOR_BLACK },
                        font_size: BOARD_TEXT_FONT_SIZE,
                        font: font.clone(),
                    };
                    cmds.spawn(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: pos_top_left,
                            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            justify_content: JustifyContent::FlexEnd,
                            align_items: AlignItems::FlexEnd,
                            ..default()
                        },
                        z_index: ZIndex::Local(Z_NOTATION_TEXT),
                        ..default()
                    })
                    .with_children(|cmds| {
                        cmds.spawn(TextBundle {
                            text: Text::from_section(square.file_char(), text_style),
                            style: Style { margin: file_label_margins, ..default() },
                            ..default()
                        });
                    });
                }

                // Rank markers
                if square.get_file() == File::A {
                    let text_style = TextStyle {
                        color: if rank.to_index() % 2 == 0 { COLOR_WHITE } else { COLOR_BLACK },
                        font_size: BOARD_TEXT_FONT_SIZE,
                        font: font.clone(),
                    };
                    cmds.spawn(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: pos_top_left,
                            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::FlexStart,
                            ..default()
                        },
                        z_index: ZIndex::Local(Z_NOTATION_TEXT),
                        ..default()
                    })
                    .with_children(|cmds| {
                        cmds.spawn(TextBundle {
                            text: Text::from_section(square.rank_char(), text_style),
                            style: Style { margin: rank_label_margins, ..default() },
                            ..default()
                        });
                    });
                }
            })
            .id();

        commands.entity(board).add_child(tile_entity);
        board_state.set_tile(square, tile_entity);
    }
}
