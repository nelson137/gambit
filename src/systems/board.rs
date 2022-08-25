use bevy::prelude::*;

use crate::{assets::PIECE_ASSET_SIZE, data::Location};

fn location_to_coords(location: &Location, win_size: f32) -> Vec3 {
    let tile_size = win_size / 8.0;
    let origin_x = -(win_size / 2.0);
    let origin_y = origin_x;
    let x = origin_x + location.file as f32 * tile_size + tile_size / 2.0;
    let y = origin_y + location.rank as f32 * tile_size + tile_size / 2.0;
    Vec3::new(x, y, location.z)
}

pub fn update_translation_for_location(
    windows: Res<Windows>,
    mut locations: Query<(&Location, &mut Transform)>,
) {
    let win = {
        if let Some(w) = windows.get_primary() {
            w
        } else {
            // eprintln!("ERROR: No primary window");
            return;
        }
    };
    let win_size = win.width().min(win.height());
    let tile_scale = win_size / 8.0 / PIECE_ASSET_SIZE;

    for (loc, mut transf) in &mut locations {
        transf.scale = Vec3::splat(tile_scale);
        transf.translation = location_to_coords(loc, win_size);
    }
}
