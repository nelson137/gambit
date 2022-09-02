use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
};

use crate::{
    assets::TILE_ASSET_SIZE,
    data::{
        Dragging, Dropped, HighlightTile, Hover, Hoverable, Location, MainCamera, MouseLocation,
        MouseWorldPosition, Piece, Selected, Tile,
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
    mut mouse_buttons_inputs_reader: EventReader<MouseButtonInput>,
    q_piece_locs: Query<&Location, With<Piece>>,

    // Entities that are `Selected && !Hover`:
    //   On press Selected will be removed from them.
    q_prev_selected: Query<Entity, (With<Selected>, Without<Hover>)>,

    // HighlightTiles that are `Selected && !Hover`:
    //   On press these will be hidden.
    mut q_prev_selected_hl_tile: Query<
        &mut Visibility,
        (With<HighlightTile>, With<Selected>, Without<Hover>),
    >,

    // Entities that are `Selected && Hover`:
    //   On release Selected will be removed from them.
    q_curr_selected: Query<Entity, (With<Selected>, With<Hover>)>,

    // HighlightTiles that are `Selected && Hover`:
    //   On release these will be hidden.
    mut q_curr_selected_hl_tile: Query<
        &mut Visibility,
        (With<HighlightTile>, With<Selected>, With<Hover>),
    >,

    // HighlightTiles and Pieces that are `Hover && !Selected`:
    //   On release Selected will be added to them.
    q_next_selected: Query<
        Entity,
        (Or<(With<HighlightTile>, With<Piece>)>, With<Hover>, Without<Selected>),
    >,

    // Pieces that are `Hover && !Selected`:
    //   On press Dragging will be added to them.
    q_next_selected_piece: Query<Entity, (With<Piece>, With<Hover>, Without<Selected>)>,

    // HighlightTiles that are `Hover && !Selected`:
    //   On press these will be shown.
    mut q_next_selected_hl_tile_vis: Query<
        (&Location, &mut Visibility),
        (With<HighlightTile>, With<Hover>, Without<Selected>),
    >,

    // Entities that are `Dragging`:
    //   On release Dropped will be added to them.
    q_dragging: Query<Entity, With<Dragging>>,
) {
    for event in mouse_buttons_inputs_reader.iter() {
        match event.state {
            ButtonState::Pressed => {
                q_prev_selected
                    .for_each(|entity| drop(commands.entity(entity).remove::<Selected>()));
                for mut vis in &mut q_prev_selected_hl_tile {
                    vis.is_visible = false;
                }

                for (hl_tile_loc, mut vis) in &mut q_next_selected_hl_tile_vis {
                    if q_piece_locs.iter().any(|piece_loc| *piece_loc == *hl_tile_loc) {
                        vis.is_visible = true;
                    }
                }

                q_next_selected_piece
                    .for_each(|entity| drop(commands.entity(entity).insert(Dragging)));
            }

            ButtonState::Released => {
                q_dragging.for_each(|entity| drop(commands.entity(entity).insert(Dropped)));

                q_curr_selected
                    .for_each(|entity| drop(commands.entity(entity).remove::<Selected>()));
                for mut vis in &mut q_curr_selected_hl_tile {
                    vis.is_visible = false;
                }

                q_next_selected.for_each(|entity| drop(commands.entity(entity).insert(Selected)));
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
