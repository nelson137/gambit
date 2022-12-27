use bevy::prelude::*;
use chess::{File, Rank, Square};

use crate::{
    assets::{PIECE_ASSET_COORDS, PIECE_ASSET_PATHS, PIECE_COLORS_TYPES, TILE_ASSET_SIZE},
    data::{
        BoardPiece, BoardState, HighlightTile, MainCamera, MoveHints, Tile, UiBoard, UiPiece,
        UiSquare, BOARD_FILE_TEXT_OFFSET_X, BOARD_FILE_TEXT_OFFSET_Y, BOARD_RANK_TEXT_OFFSET_X,
        BOARD_RANK_TEXT_OFFSET_Y, BOARD_TEXT_FONT_SIZE, COLOR_BLACK, COLOR_HIGHLIGHT, COLOR_WHITE,
        Z_HIGHLIGHT_TILE, Z_MOVE_HINT, Z_NOTATION_TEXT, Z_PIECE, Z_TILE,
    },
};

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default()).insert(MainCamera); // ::new_with_far(1000.0)
}

#[derive(Clone, StageLabel)]
pub enum SpawnStage {
    Board,
    TilesHintsPieces,
}

pub fn spawn_board(mut commands: Commands) {
    commands.spawn((SpatialBundle::default(), UiBoard));
}

pub fn spawn_tiles_hints_pieces(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut board_state: ResMut<BoardState>,
    q_board: Query<Entity, With<UiBoard>>,
) {
    let board = q_board.single();
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");
    let move_hint_texture = asset_server.load("hints/move.png");
    let capture_hint_texture = asset_server.load("hints/capture.png");

    // Iterates through all squares, row-wise, from a1 to h8
    for square in !chess::EMPTY {
        let file = square.get_file();
        let rank = square.get_rank();
        let ui_square = UiSquare::new(square);

        // Tile
        let file_rank_sum = rank.to_index() + file.to_index();
        let mut tile = commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: if file_rank_sum % 2 == 0 { COLOR_BLACK } else { COLOR_WHITE },
                custom_size: Some(Vec2::splat(TILE_ASSET_SIZE)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::Z * Z_TILE),
            ..default()
        });

        tile.insert(Tile);
        tile.insert(ui_square);

        // File markers
        if square.get_rank() == Rank::First {
            let style = TextStyle {
                color: if file.to_index() % 2 == 0 { COLOR_WHITE } else { COLOR_BLACK },
                font_size: BOARD_TEXT_FONT_SIZE,
                font: font.clone(),
            };
            tile.with_children(|cmds| {
                cmds.spawn(Text2dBundle {
                    text: Text::from_section(ui_square.file_char(), style)
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
        if square.get_file() == File::A {
            let style = TextStyle {
                color: if rank.to_index() % 2 == 0 { COLOR_WHITE } else { COLOR_BLACK },
                font_size: BOARD_TEXT_FONT_SIZE,
                font: font.clone(),
            };
            tile.with_children(|cmds| {
                cmds.spawn(Text2dBundle {
                    text: Text::from_section(ui_square.rank_char(), style)
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

        let tile_entity = tile.id();

        // Highlight tile
        let hl_tile_entity = commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    color: COLOR_HIGHLIGHT,
                    custom_size: Some(Vec2::splat(TILE_ASSET_SIZE)),
                    ..default()
                },
                visibility: Visibility { is_visible: false },
                transform: Transform::from_translation(Vec3::Z * Z_HIGHLIGHT_TILE),
                ..default()
            })
            .insert(HighlightTile)
            .insert(ui_square)
            .id();
        assert!(
            board_state.highlights.insert(square, hl_tile_entity).is_none(),
            "Failed to insert highlight tile into state: tile already at this square"
        );

        // Move hint
        let move_entity = commands
            .spawn(SpriteBundle {
                texture: move_hint_texture.clone(),
                visibility: Visibility { is_visible: false },
                transform: Transform::from_translation(Vec3::Z * Z_MOVE_HINT),
                ..default()
            })
            .insert(ui_square)
            .id();

        // Capture hint
        let capture_entity = commands
            .spawn(SpriteBundle {
                texture: capture_hint_texture.clone(),
                visibility: Visibility { is_visible: false },
                transform: Transform::from_translation(Vec3::Z * Z_MOVE_HINT),
                ..default()
            })
            .insert(ui_square)
            .id();

        let hint = MoveHints { capture_entity, move_entity };
        assert!(
            board_state.move_hints.insert(square, hint).is_none(),
            "Failed to insert board hint into state: hint already at this square"
        );

        commands.entity(board).push_children(&[
            tile_entity,
            hl_tile_entity,
            move_entity,
            capture_entity,
        ]);
    }

    let pice_paths_and_coords = PIECE_ASSET_PATHS
        .iter()
        .copied()
        .flatten()
        .zip(PIECE_ASSET_COORDS.iter().copied().flatten())
        .zip(PIECE_COLORS_TYPES.iter().copied().flatten());
    for ((&path, &(rank, file)), &(color, typ)) in pice_paths_and_coords {
        let square = Square::make_square(rank, file);
        let entity = commands
            .spawn(SpriteBundle {
                texture: asset_server.load(path),
                transform: Transform::from_translation(Vec3::Z * Z_PIECE),
                ..default()
            })
            .insert(UiPiece::new(color, typ))
            .insert(UiSquare::new(square))
            .id();
        assert!(
            board_state.pieces.insert(square, BoardPiece::new(entity, color, typ)).is_none(),
            "Failed to insert board piece into state: piece already at this square"
        );
    }
}
