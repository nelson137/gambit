use bevy::prelude::*;

use crate::{
    assets::{PIECE_ASSET_COORDS, PIECE_ASSET_PATHS},
    data::Location,
    WIN_HEIGHT, WIN_WIDTH,
};

pub fn resize_window(mut windows: ResMut<Windows>) {
    windows.primary_mut().set_resolution(WIN_WIDTH, WIN_HEIGHT);
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default()); // ::new_with_far(1000.0)
}

pub fn setup_board(mut commands: Commands, asset_server: Res<AssetServer>) {
    let black = asset_server.load("tiles/black.png");
    let white = asset_server.load("tiles/white.png");

    for rank in 0..8_u8 {
        for file in 0..8_u8 {
            let tile = (if (rank + file) % 2 == 0 { &black } else { &white }).clone();
            commands
                .spawn_bundle(SpriteBundle { texture: tile, ..default() })
                .insert(Location::new(file, rank, 0.0));
        }
    }

    let pice_paths_and_coords = PIECE_ASSET_PATHS
        .iter()
        .zip(PIECE_ASSET_COORDS)
        .flat_map(|(paths, coords)| paths.iter().copied().zip(coords.iter().copied()));
    for (path, (file, rank)) in pice_paths_and_coords {
        commands
            .spawn_bundle(SpriteBundle { texture: asset_server.load(path), ..default() })
            .insert(Location::new(file, rank, 1.0));
    }
}
