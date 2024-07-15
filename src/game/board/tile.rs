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
    let board = q_board.single();

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
                        width: Val::Percent(100.0 / 8.0),
                        height: Val::Percent(100.0 / 8.0),
                        ..default()
                    },
                    z_index: ZIndex::Global(Z_TILE),
                    ..default()
                },
            ))
            .with_children(|cmds| {
                pub const BOARD_TEXT_FONT_SIZE: f32 = 20.0;

                let text_style = TextStyle {
                    color: if file_rank_sum % 2 == 0 { COLOR_WHITE } else { COLOR_BLACK },
                    font_size: BOARD_TEXT_FONT_SIZE,
                    font: asset_server.load(FONT_PATH),
                };

                // File markers
                if square.get_rank() == Rank::First {
                    cmds.spawn(TextBundle {
                        text: Text::from_section(square.file_char(), text_style.clone()),
                        style: Style {
                            position_type: PositionType::Absolute,
                            bottom: Val::Percent(3.5),
                            right: Val::Percent(8.0),
                            ..default()
                        },
                        ..default()
                    });
                }

                // Rank markers
                if square.get_file() == File::A {
                    cmds.spawn(TextBundle {
                        text: Text::from_section(square.rank_char(), text_style),
                        style: Style {
                            position_type: PositionType::Absolute,
                            top: Val::Percent(1.0),
                            left: Val::Percent(4.5),
                            ..default()
                        },
                        ..default()
                    });
                }
            })
            .id();

        commands.entity(board).add_child(tile_entity);
        board_state.set_tile(square, tile_entity);
    }
}
