use bevy::prelude::*;

use crate::{
    assets::TILE_ASSET_SIZE,
    data::{
        BoardState, DoMove, DoUnselect, Dragging, Dropped, HideHint, HighlightTile, Hover,
        Hoverable, Location, MainCamera, MouseLocation, MouseWorldPosition, Piece, PieceMoveType,
        PossibleMove, Selected, ShowHint, ShowingMovesFor, Tile, Z_PIECE, Z_PIECE_SELECTED,
    },
};

// Source: https://bevy-cheatbook.github.io/cookbook/cursor2world.html
pub fn mouse_screen_position_to_world(
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_world_pos: ResMut<MouseWorldPosition>,
    mut q_dragging: Query<(&Location, &mut Transform), (With<Dragging>, With<Piece>)>,
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

        for (loc, mut transf) in &mut q_dragging {
            transf.translation.x = world_pos.x;
            transf.translation.y = world_pos.y;
            transf.translation.z = loc.z;
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
    board_state: Res<BoardState>,
    q_prev_select: Query<Entity, (With<Selected>, Without<Hover>, Without<Dragging>)>,
    mut q_new_select: Query<Entity, (With<Hover>, Without<Dragging>)>,
    mut q_dragging: Query<(Entity, &Location, &Dragging, Option<&Selected>)>,
) {
    if mouse_buttons.just_pressed(MouseButton::Left) {
        // Unselect the current selection if not hovered
        for entity in &q_prev_select {
            commands.entity(entity).remove::<Selected>().insert(DoUnselect);
        }

        // Start drag
        if let Some(mouse_loc) = mouse_loc.0 {
            for entity in &mut q_new_select {
                if board_state.pieces.contains_key(&mouse_loc) {
                    commands.entity(entity).insert(Dragging::new(mouse_loc));
                }
            }
        }
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        if let Some(mouse_loc) = mouse_loc.0 {
            for (entity, loc, dragging, selected) in &mut q_dragging {
                let mut cmds = commands.entity(entity);
                cmds.remove::<Dragging>().insert(Dropped);

                // If the mouse up location is the same as the drag's mouse down
                if mouse_loc == dragging.mouse_down_location {
                    if selected.is_some() {
                        // Un-select
                        // Mouse up in same location as mouse down when selected
                        cmds.remove::<Selected>().insert(DoUnselect);
                    } else {
                        // Select
                        // Mouse up in same location as mouse down when *not* selected
                        cmds.insert(Selected);
                    }
                } else {
                    // The move type doesn't matter here, hashing is done only by location
                    let move_with_mouse_loc = PossibleMove::new(mouse_loc, PieceMoveType::Move);
                    if board_state.get_piece_moves(&loc).contains(&move_with_mouse_loc) {
                        // Move
                        // Mouse up in different location than the drag's mouse down and is a valid
                        // move
                        cmds.insert(DoMove);
                    } else {
                        // Select
                        // Mouse up in different location than the drag's mouse down and is *not* a
                        // valid move
                        cmds.insert(Selected);
                    }
                }
            }
        }
    }
}

pub fn selections(
    mut commands: Commands,
    mut board_state: ResMut<BoardState>,
    mut showing_piece_moves: ResMut<ShowingMovesFor>,
    mut q_unselect: Query<
        (Entity, Option<&HighlightTile>, &mut Visibility),
        (Added<DoUnselect>, Without<Dragging>),
    >,
    mut q_new_select: Query<
        (Option<&Piece>, Option<&HighlightTile>, &Location, &mut Visibility),
        Added<Dragging>,
    >,
) {
    for (entity, hl_tile, mut vis) in &mut q_unselect {
        commands.entity(entity).remove::<DoUnselect>();
        if hl_tile.is_some() {
            // Hide highlight tile
            vis.is_visible = false;
            // Hide previous move hints
            if let Some(showing_loc) = showing_piece_moves.0 {
                board_state.hide_piece_move_hints(&mut commands, &showing_loc);
                showing_piece_moves.0 = None;
            }
        }
    }

    for (piece, hl_tile, loc, mut vis) in &mut q_new_select {
        if piece.is_some() {
            // Hide previous move hints
            // Note: this should not happen because q_unselect should take care of it
            if let Some(showing_loc) = showing_piece_moves.0 {
                if showing_loc != *loc {
                    board_state.hide_piece_move_hints(&mut commands, &showing_loc);
                }
            }
            // Show move hints
            showing_piece_moves.0 = Some(*loc);
            board_state.show_piece_move_hints(&mut commands, *loc);
        } else if hl_tile.is_some() {
            if board_state.pieces.contains_key(&loc) {
                // Show if it's a highlight tile and it has a piece
                vis.is_visible = true;
            }
        }
    }
}

pub fn piece_move(
    mut commands: Commands,
    mouse_loc: Res<MouseLocation>,
    mut board_state: ResMut<BoardState>,
    mut showing_piece_moves: ResMut<ShowingMovesFor>,
    mut q_dragging_piece: Query<
        &mut Location,
        (With<Piece>, Added<Dragging>, Without<Dropped>, Without<DoMove>),
    >,
    mut q_dropped: Query<
        (
            Entity,
            Option<&Piece>,
            Option<&HighlightTile>,
            Option<&DoMove>,
            &mut Visibility,
            &mut Location,
        ),
        Added<Dropped>,
    >,
) {
    for mut loc in &mut q_dragging_piece {
        loc.snap = false;
        loc.z = Z_PIECE_SELECTED;
    }

    if let Some(mouse_loc) = mouse_loc.0 {
        // Finish select
        for (entity, piece, hl_tile, do_move, mut vis, mut loc) in &mut q_dropped {
            commands.entity(entity).remove::<Dropped>().remove::<DoMove>();
            if hl_tile.is_some() {
                if do_move.is_some() {
                    // Hide highlight tile
                    vis.is_visible = false;
                }
            } else if piece.is_some() {
                loc.snap = true;
                loc.z = Z_PIECE;
                if do_move.is_some() {
                    // Move piece location
                    board_state.move_piece(*loc, mouse_loc);
                    loc.move_to(mouse_loc);
                    // Hide move hints
                    if let Some(showing_loc) = showing_piece_moves.0 {
                        board_state.hide_piece_move_hints(&mut commands, &showing_loc);
                        showing_piece_moves.0 = None;
                    }
                }
            }
        }
    }
}

pub fn hints_hide(
    mut commands: Commands,
    mut q_hide_hints: Query<(Entity, &mut Visibility), Added<HideHint>>,
) {
    for (entity, mut vis) in &mut q_hide_hints {
        commands.entity(entity).remove::<HideHint>();
        vis.is_visible = false;
    }
}

pub fn hints_show(
    mut commands: Commands,
    mut q_show_hints: Query<(Entity, &mut Visibility), Added<ShowHint>>,
) {
    for (entity, mut vis) in &mut q_show_hints {
        commands.entity(entity).remove::<ShowHint>();
        vis.is_visible = true;
    }
}
