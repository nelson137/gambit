use bevy::prelude::*;

use crate::{
    assets::{PIECE_ASSET_COORDS, PIECE_ASSET_PATHS, TILE_ASSET_SIZE},
    data::{
        Board, Location, MainCamera, Piece, Tile, BOARD_FILE_TEXT_OFFSET_X,
        BOARD_FILE_TEXT_OFFSET_Y, BOARD_RANK_TEXT_OFFSET_X, BOARD_RANK_TEXT_OFFSET_Y,
        BOARD_TEXT_FONT_SIZE, COLOR_BLACK, COLOR_WHITE,
    },
    WIN_HEIGHT, WIN_WIDTH,
};

pub fn resize_window(mut windows: ResMut<Windows>) {
    windows.primary_mut().set_resolution(WIN_WIDTH, WIN_HEIGHT);
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default()).insert(MainCamera); // ::new_with_far(1000.0)
}

pub fn setup_board(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(SpatialBundle::default()).insert(Board).with_children(|parent| {
        let font = asset_server.load("fonts/FiraMono-Medium.ttf");

        for rank in 0..8_u8 {
            for file in 0..8_u8 {
                let location = Location::new(file, rank, 0.0);

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
                                0.1,
                            ])),
                            ..default()
                        });
                    });
                }

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
                                0.1,
                            )),
                            ..default()
                        });
                    });
                }
            }
        }

        let pice_paths_and_coords = PIECE_ASSET_PATHS
            .iter()
            .zip(PIECE_ASSET_COORDS)
            .flat_map(|(paths, coords)| paths.iter().copied().zip(coords.iter().copied()));
        for (path, (file, rank)) in pice_paths_and_coords {
            parent
                .spawn_bundle(SpriteBundle { texture: asset_server.load(path), ..default() })
                .insert(Piece)
                .insert(Location::new(file, rank, 1.0));
        }
    });
}
