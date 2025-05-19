use bevy::prelude::*;
use chess::{File, Rank};

use crate::{
    debug_name_f,
    game::consts::{FONT_PATH, Z_TILE},
};

use super::{BoardState, Square, UiBoard};

#[derive(Component)]
pub struct Tile;

/// The "black" bord color.
///
/// `#769656`
pub const COLOR_BLACK: Color = Color::srgb(
    0x76 as f32 / u8::MAX as f32,
    0x96 as f32 / u8::MAX as f32,
    0x56 as f32 / u8::MAX as f32,
);

/// The "white" bord color.
///
/// `#eeeed2`
pub const COLOR_WHITE: Color = Color::srgb(
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
    let board = q_board.single().unwrap();

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
                Node {
                    position_type: PositionType::Relative,
                    width: Val::Percent(100.0 / 8.0),
                    height: Val::Percent(100.0 / 8.0),
                    ..default()
                },
                BackgroundColor(color),
                GlobalZIndex(Z_TILE),
            ))
            .with_children(|cmds| {
                pub const BOARD_TEXT_FONT_SIZE: f32 = 16.0;

                let text_font = TextFont {
                    font_size: BOARD_TEXT_FONT_SIZE,
                    font: asset_server.load(FONT_PATH),
                    ..default()
                };
                let text_color =
                    TextColor(if file_rank_sum % 2 == 0 { COLOR_WHITE } else { COLOR_BLACK });

                // File markers
                if square.get_rank() == Rank::First {
                    cmds.spawn((
                        Text(square.file_char().to_string()),
                        text_font.clone(),
                        text_color,
                        Node {
                            position_type: PositionType::Absolute,
                            bottom: Val::Percent(3.5),
                            right: Val::Percent(8.0),
                            ..default()
                        },
                    ));
                }

                // Rank markers
                if square.get_file() == File::A {
                    cmds.spawn((
                        Text(square.rank_char().to_string()),
                        text_font.clone(),
                        text_color,
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Percent(1.0),
                            left: Val::Percent(4.5),
                            ..default()
                        },
                    ));
                }
            })
            .id();

        commands.entity(board).add_child(tile_entity);
        board_state.set_tile(square, tile_entity);
    }
}
