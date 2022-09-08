use bevy::prelude::*;

use crate::{
    assets::TILE_ASSET_SIZE,
    data::{
        Dragging, Dropped, HighlightTile, Hover, Hoverable, Location, MainCamera, MouseLocation,
        MouseWorldPosition, Piece, Selected, Selecting, Tile,
    },
};

// Source: https://bevy-cheatbook.github.io/cookbook/cursor2world.html
pub fn mouse_screen_position_to_world(
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_world_pos: ResMut<MouseWorldPosition>,
    mut q_dragging: Query<&mut Transform, With<Dragging>>,
) {
    let win = windows.primary();

    if let Some(screen_pos) = win.cursor_position() {
        let (camera, camera_transf) = q_camera.single();

        let window_size = Vec2::new(win.width() as f32, win.height() as f32);

        // Convert mouse position on screen [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = 2.0 * (screen_pos / window_size) - Vec2::ONE;

        // Matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transf.compute_matrix() * camera.projection_matrix().inverse();

        // Convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        mouse_world_pos.0 = world_pos.truncate();

        for mut transf in &mut q_dragging {
            transf.translation.x = world_pos.x;
            transf.translation.y = world_pos.y;
        }
    }
}

pub fn mouse_world_position_to_location(
    mouse_world_pos: Res<MouseWorldPosition>,
    mut mouse_loc: ResMut<MouseLocation>,
    q_tiles: Query<(&Location, &Transform), With<Tile>>,
) {
    let mouse_pos = mouse_world_pos.0.extend(0.0);

    for (loc, transf) in &q_tiles {
        let collision = bevy::sprite::collide_aabb::collide(
            mouse_pos,
            Vec2::ZERO,
            transf.translation, // The z component doesn't matter, it is truncated away
            Vec2::splat(TILE_ASSET_SIZE) * transf.scale.truncate(),
        );
        if collision.is_some() {
            mouse_loc.0 = Some(*loc);
            return;
        }
    }

    mouse_loc.0 = None;
}

pub fn mouse_hover(
    mut commands: Commands,
    mouse_loc: Res<MouseLocation>,
    q_hoverable: Query<(Entity, &Location), With<Hoverable>>,
) {
    if let Some(mouse_loc) = mouse_loc.0 {
        for (entity, loc) in &q_hoverable {
            let mut entity_cmds = commands.entity(entity);
            if *loc == mouse_loc {
                entity_cmds.insert(Hover);
            } else {
                entity_cmds.remove::<Hover>();
            }
        }
    } else {
        // Remove Hover component from all entities
        q_hoverable.for_each(|entity| drop(commands.entity(entity.0).remove::<Hover>()));
    }
}

pub fn click_handler(
    mut commands: Commands,
    mouse_buttons: Res<Input<MouseButton>>,
    mouse_loc: Res<MouseLocation>,
    q_piece_locs: Query<&Location, With<Piece>>,
    mut q_prev_select: Query<
        (Entity, Option<&HighlightTile>, &mut Visibility),
        (With<Selected>, Without<Hover>, Without<Selecting>),
    >,
    mut q_new_select: Query<
        (Entity, Option<&Piece>, &Location, &mut Visibility),
        (With<Hover>, Without<Selecting>),
    >,
    q_dragging: Query<Entity, With<Dragging>>,
    mut q_selecting: Query<(
        Entity,
        Option<&HighlightTile>,
        &mut Visibility,
        &Selecting,
        Option<&Selected>,
    )>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        // For each entity (pieces & highlight tiles) that is selected and not being hovered
        for (entity, hl_tile, mut vis) in &mut q_prev_select {
            commands.entity(entity).remove::<Selected>();
            if hl_tile.is_some() {
                // Hide if it's a highlight tile
                vis.is_visible = false;
            }
        }

        // Highlight and begin dragging the current hover target
        // TODO: introduce HasPiece component which, when present, means that loc has piece
        if let Some(mouse_loc) = mouse_loc.0 {
            // For each entity (pieces & highlight tiles) that are being hovered
            for (entity, piece, loc, mut vis) in &mut q_new_select {
                let mut cmds = commands.entity(entity);
                cmds.insert(Selecting::new(mouse_loc));

                if piece.is_some() {
                    // Insert Dragging if it's a piece
                    cmds.insert(Dragging);
                } else if q_piece_locs.iter().any(|piece_loc| *piece_loc == *loc) {
                    // Show if it's a highlight tile and it has a piece
                    vis.is_visible = true;
                }
            }
        }
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        // For each entity (piece) that is Dragging, insert Dropped
        q_dragging.for_each(|entity| drop(commands.entity(entity).insert(Dropped)));

        if let Some(mouse_loc) = mouse_loc.0 {
            // For each entity (pieces & highlight tiles) that is Selecting
            for (entity, hl_tile, mut vis, selecting, selected) in &mut q_selecting {
                let mut cmds = commands.entity(entity);
                cmds.remove::<Selecting>();

                // If already Selected
                if selected.is_some() {
                    // If the current mouse location is the same as the last mouse down location
                    if mouse_loc == selecting.mouse_down_location {
                        cmds.remove::<Selected>();
                        if hl_tile.is_some() {
                            // Hide if it's a highlight tile
                            vis.is_visible = false;
                        }
                    }
                } else {
                    cmds.insert(Selected);
                }
            }
        }
    }
}

pub fn click_handler2(
    mut commands: Commands,
    mut q_added_dragging: Query<&mut Location, (Added<Dragging>, Without<Dropped>)>,
    mut q_added_dropped: Query<(Entity, &mut Location), Added<Dropped>>,
) {
    q_added_dragging.for_each_mut(|mut loc| loc.snap = false);

    for (entity, mut loc) in &mut q_added_dropped {
        commands.entity(entity).remove::<Dragging>().remove::<Dropped>();
        loc.snap = true;
    }
}
