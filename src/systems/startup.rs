use bevy::prelude::*;

use crate::{
    assets::{PIECE_ASSET_COORDS, PIECE_ASSET_PATHS, PIECE_COLORS_TYPES, TILE_ASSET_SIZE},
    data::{
        Board, BoardPiece, BoardState, HighlightTile, Hoverable, Location, MainCamera, MoveHints,
        Piece, Tile, BOARD_FILE_TEXT_OFFSET_X, BOARD_FILE_TEXT_OFFSET_Y, BOARD_RANK_TEXT_OFFSET_X,
        BOARD_RANK_TEXT_OFFSET_Y, BOARD_TEXT_FONT_SIZE, COLOR_BLACK, COLOR_HIGHLIGHT, COLOR_WHITE,
        Z_HIGHLIGHT_TILE, Z_MOVE_HINT, Z_NOTATION_TEXT, Z_PIECE, Z_TILE,
    },
    WIN_HEIGHT, WIN_WIDTH,
};

pub fn resize_window(mut windows: ResMut<Windows>) {
    windows.primary_mut().set_resolution(WIN_WIDTH, WIN_HEIGHT);
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default()).insert(MainCamera); // ::new_with_far(1000.0)
}

pub fn setup_board(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut board_state: ResMut<BoardState>,
) {
    commands.spawn_bundle(SpatialBundle::default()).insert(Board).with_children(|parent| {
        let font = asset_server.load("fonts/FiraMono-Medium.ttf");
        let move_hint_texture = asset_server.load("hints/move.png");
        let capture_hint_texture = asset_server.load("hints/capture.png");

        for rank in 0..8_u8 {
            for file in 0..8_u8 {
                let location = Location::new_with_z(file, rank, Z_TILE);

                // Tile
                let mut tile = parent.spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: if (rank + file) % 2 == 0 { COLOR_BLACK } else { COLOR_WHITE },
                        custom_size: Some(Vec2::splat(TILE_ASSET_SIZE)),
                        ..default()
                    },
                    ..default()
                });

                tile.insert(Tile);
                tile.insert(location);

                // File markers
                if rank == 0 {
                    let style = TextStyle {
                        color: if file % 2 == 0 { COLOR_WHITE } else { COLOR_BLACK },
                        font_size: BOARD_TEXT_FONT_SIZE,
                        font: font.clone(),
                    };
                    tile.with_children(|cmds| {
                        cmds.spawn_bundle(Text2dBundle {
                            text: Text::from_section(location.file_char(), style)
                                .with_alignment(TextAlignment::CENTER),
                            transform: Transform::from_translation(Vec3::from_slice(&[
                                BOARD_FILE_TEXT_OFFSET_X,
                                BOARD_FILE_TEXT_OFFSET_Y,
                                Z_NOTATION_TEXT,
                            ])),
                            ..default()
                        });
                    });
                }

                // Rank markers
                if file == 0 {
                    let style = TextStyle {
                        color: if rank % 2 == 0 { COLOR_WHITE } else { COLOR_BLACK },
                        font_size: BOARD_TEXT_FONT_SIZE,
                        font: font.clone(),
                    };
                    tile.with_children(|cmds| {
                        cmds.spawn_bundle(Text2dBundle {
                            text: Text::from_section(location.rank_char(), style)
                                .with_alignment(TextAlignment::CENTER),
                            transform: Transform::from_translation(Vec3::new(
                                BOARD_RANK_TEXT_OFFSET_X,
                                BOARD_RANK_TEXT_OFFSET_Y,
                                Z_NOTATION_TEXT,
                            )),
                            ..default()
                        });
                    });
                }

                // Highlight tile
                parent
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: COLOR_HIGHLIGHT,
                            custom_size: Some(Vec2::splat(TILE_ASSET_SIZE)),
                            ..default()
                        },
                        visibility: Visibility { is_visible: false },
                        ..default()
                    })
                    .insert(HighlightTile)
                    .insert(location.with_z(Z_HIGHLIGHT_TILE))
                    .insert(Hoverable);

                // Move hint
                let move_entity = parent
                    .spawn_bundle(SpriteBundle {
                        texture: move_hint_texture.clone(),
                        visibility: Visibility { is_visible: false },
                        ..default()
                    })
                    .insert(location.with_z(Z_MOVE_HINT))
                    .id();

                // Capture hint
                let capture_entity = parent
                    .spawn_bundle(SpriteBundle {
                        texture: capture_hint_texture.clone(),
                        visibility: Visibility { is_visible: false },
                        ..default()
                    })
                    .insert(location.with_z(Z_MOVE_HINT))
                    .id();

                let hint = MoveHints { entity_capture: capture_entity, entity_move: move_entity };
                assert!(
                    board_state.move_hints.insert(location, hint).is_none(),
                    "Failed to insert board hint into state: hint already at this location"
                );
            }
        }

        let pice_paths_and_coords = PIECE_ASSET_PATHS
            .iter()
            .copied()
            .flatten()
            .zip(PIECE_ASSET_COORDS.iter().copied().flatten())
            .zip(PIECE_COLORS_TYPES.iter().copied().flatten());
        for ((&path, &(file, rank)), &(color, typ)) in pice_paths_and_coords {
            let location = Location::new_with_z(file, rank, Z_PIECE);
            assert!(
                board_state.pieces.insert(location, BoardPiece::new(color, typ)).is_none(),
                "Failed to insert board piece into state: piece already at this location"
            );
            parent
                .spawn_bundle(SpriteBundle { texture: asset_server.load(path), ..default() })
                .insert(Piece)
                .insert(color)
                .insert(typ)
                .insert(location)
                .insert(Hoverable);
        }
    });
}
