use bevy::prelude::*;
use chess::ChessMove;

use crate::{
    assets::TILE_ASSET_SIZE,
    data::{
        BoardState, DoMove, DoUnselect, Dragging, Dropped, HideHint, HighlightTile, Hover,
        Hoverable, Location, MainCamera, MouseSquare, MouseWorldPosition, Selected, ShowHint,
        ShowingMovesFor, Tile, UiPiece, Z_PIECE, Z_PIECE_SELECTED,
    },
    util::consume,
};

// Source: https://bevy-cheatbook.github.io/cookbook/cursor2world.html
pub fn mouse_screen_position_to_world(
    windows: Res<Windows>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut mouse_world_pos: ResMut<MouseWorldPosition>,
    mut q_dragging: Query<(&Location, &mut Transform), (With<Dragging>, With<UiPiece>)>,
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
    mut mouse_square: ResMut<MouseSquare>,
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
            mouse_square.0 = Some(loc.square());
            return;
        }
    }

    mouse_square.0 = None;
}

pub fn mouse_hover(
    mut commands: Commands,
    mouse_square: Res<MouseSquare>,
    q_hoverable: Query<(Entity, &Location), With<Hoverable>>,
) {
    if let Some(mouse_square) = mouse_square.0 {
        for (entity, loc) in &q_hoverable {
            let mut entity_cmds = commands.entity(entity);
            if loc.square() == mouse_square {
                entity_cmds.insert(Hover);
            } else {
                entity_cmds.remove::<Hover>();
            }
        }
    } else {
        // Remove Hover component from all entities
        q_hoverable.for_each(|entity| consume(commands.entity(entity.0).remove::<Hover>()));
    }
}

pub fn click_handler(
    mut commands: Commands,
    mouse_buttons: Res<Input<MouseButton>>,
    mouse_square: Res<MouseSquare>,
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
        if let Some(mouse_square) = mouse_square.0 {
            for entity in &mut q_new_select {
                if board_state.pieces.contains_key(&mouse_square) {
                    commands.entity(entity).insert(Dragging::new(mouse_square));
                }
            }
        }
    }

    if mouse_buttons.just_released(MouseButton::Left) {
        if let Some(mouse_square) = mouse_square.0 {
            for (entity, loc, dragging, selected) in &mut q_dragging {
                let mut cmds = commands.entity(entity);
                cmds.remove::<Dragging>().insert(Dropped);

                // If the mouse up location is the same as the drag's mouse down
                if mouse_square == dragging.mouse_down_square {
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
                    let move_with_mouse_loc = ChessMove::new(loc.square(), mouse_square, None);
                    // if board_state.is_colors_turn_at(*loc)
                    //     && board_state.get_piece_moves(loc).contains(&move_with_mouse_loc)
                    if true {
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
        (Option<&UiPiece>, Option<&HighlightTile>, &Location, &mut Visibility),
        Added<Dragging>,
    >,
) {
    for (entity, hl_tile, mut vis) in &mut q_unselect {
        commands.entity(entity).remove::<DoUnselect>();
        if hl_tile.is_some() {
            // Hide highlight tile
            vis.is_visible = false;
            // Hide previous move hints
            if showing_piece_moves.0.is_some() {
                board_state.hide_piece_move_hints(&mut commands);
                showing_piece_moves.0 = None;
            }
        }
    }

    for (piece, hl_tile, loc, mut vis) in &mut q_new_select {
        if piece.is_some() {
            // Hide previous move hints
            // Note: this should not happen because q_unselect should take care of it
            if let Some(showing_for_square) = showing_piece_moves.0 {
                if showing_for_square != loc.square() {
                    board_state.hide_piece_move_hints(&mut commands);
                }
            }
            if true && board_state.is_colors_turn_at(loc.square()) {
                // Show move hints
                showing_piece_moves.0 = Some(loc.square());
                board_state.show_piece_move_hints(&mut commands, loc.square());
            }
        } else if hl_tile.is_some() {
            #[allow(clippy::collapsible_if)]
            if board_state.pieces.contains_key(&loc.square()) {
                // Show if it's a highlight tile and it has a piece
                vis.is_visible = true;
            }
        }
    }
}

pub fn piece_move(
    mut commands: Commands,
    mouse_square: Res<MouseSquare>,
    mut board_state: ResMut<BoardState>,
    mut showing_piece_moves: ResMut<ShowingMovesFor>,
    mut q_dragging_piece: Query<
        &mut Location,
        (With<UiPiece>, Added<Dragging>, Without<Dropped>, Without<DoMove>),
    >,
    mut q_dropped: Query<
        (
            Entity,
            Option<&UiPiece>,
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

    if let Some(mouse_square) = mouse_square.0 {
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
                    board_state.move_piece(loc.square(), mouse_square);
                    loc.move_to(mouse_square);
                    board_state.move_count += 1;
                    // Hide move hints
                    if showing_piece_moves.0.is_some() {
                        board_state.hide_piece_move_hints(&mut commands);
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
