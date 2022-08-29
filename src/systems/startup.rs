use bevy::prelude::*;

use crate::{
    assets::{PIECE_ASSET_COORDS, PIECE_ASSET_PATHS, TILE_ASSET_SIZE},
    data::{Board, Location, MainCamera, Piece, Tile, COLOR_BLACK, COLOR_WHITE},
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
        for rank in 0..8_u8 {
            for file in 0..8_u8 {
                parent
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: if (rank + file) % 2 == 0 { COLOR_BLACK } else { COLOR_WHITE },
                            custom_size: Some(Vec2::splat(TILE_ASSET_SIZE)),
                            ..default()
                        },
                        ..default()
                    })
                    .insert(Tile)
                    .insert(Location::new(file, rank, 0.0));
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
