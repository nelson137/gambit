use bevy::prelude::*;
use chess::{File, Rank};

use crate::{
    assets::SquareStartingPieceInfo,
    data::{
        BoardPiece, BoardState, DragContainer, HighlightTile, MainCamera, MoveHints, PieceColor,
        PieceType, Tile, Ui, UiBoard, UiPiece, UiSquare, BOARD_TEXT_FONT_SIZE, COLOR_BLACK,
        COLOR_HIGHLIGHT, COLOR_WHITE, Z_HIGHLIGHT_TILE, Z_MOVE_HINT, Z_NOTATION_TEXT, Z_PIECE,
        Z_PIECE_SELECTED, Z_TILE,
    },
};

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default()).insert(MainCamera); // ::new_with_far(1000.0)
}

#[derive(Clone, StageLabel)]
pub enum SpawnStage {
    Ui,
    Board,
    TilesHintsPieces,
}

pub fn spawn_ui(mut commands: Commands) {
    commands.spawn((
        Ui,
        NodeBundle {
            style: Style {
                size: Size { width: Val::Percent(100.0), height: Val::Percent(100.0) },
                position_type: PositionType::Absolute,
                position: UiRect { left: Val::Percent(0.0), top: Val::Percent(0.0), ..default() },
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        },
    ));
}

pub fn spawn_board(mut commands: Commands, q_ui: Query<Entity, With<Ui>>) {
    // let min_size = PANEL_HEIGHT * 2.0;
    let entity = commands
        .spawn((
            UiBoard,
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Auto, Val::Percent(100.0)),
                    // min_size: Size::new(min_size, min_size),
                    aspect_ratio: Some(1.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::WrapReverse,
                    ..default()
                },
                ..default()
            },
        ))
        .id();
    commands.entity(q_ui.single()).add_child(entity);
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
    let pos_top_left = UiRect { top: Val::Px(0.0), left: Val::Px(0.0), ..default() };
    let file_label_margins =
        UiRect { bottom: Val::Percent(3.5), right: Val::Percent(8.0), ..default() };
    let rank_label_margins =
        UiRect { top: Val::Percent(1.0), left: Val::Percent(4.5), ..default() };

    // Iterates through all squares, row-wise, from a1 to h8
    for square in !chess::EMPTY {
        let file = square.get_file();
        let rank = square.get_rank();
        let ui_square = UiSquare::new(square);

        // Tile
        let file_rank_sum = rank.to_index() + file.to_index();
        let color = if file_rank_sum % 2 == 0 { COLOR_BLACK } else { COLOR_WHITE };
        let tile_entity = commands
            .spawn((
                Tile,
                ui_square,
                NodeBundle {
                    background_color: color.into(),
                    style: Style {
                        position_type: PositionType::Relative,
                        size: Size::new(Val::Percent(100.0 / 8.0), Val::Percent(100.0 / 8.0)),
                        ..default()
                    },
                    z_index: ZIndex::Local(Z_TILE),
                    ..default()
                },
            ))
            .with_children(|cmds| {
                // File markers
                if square.get_rank() == Rank::First {
                    let text_style = TextStyle {
                        color: if file.to_index() % 2 == 0 { COLOR_WHITE } else { COLOR_BLACK },
                        font_size: BOARD_TEXT_FONT_SIZE,
                        font: font.clone(),
                    };
                    cmds.spawn(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: pos_top_left,
                            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            justify_content: JustifyContent::FlexEnd,
                            align_items: AlignItems::FlexEnd,
                            ..default()
                        },
                        z_index: ZIndex::Local(Z_NOTATION_TEXT),
                        ..default()
                    })
                    .with_children(|cmds| {
                        cmds.spawn(TextBundle {
                            text: Text::from_section(ui_square.file_char(), text_style),
                            style: Style { margin: file_label_margins, ..default() },
                            ..default()
                        });
                    });
                }

                // Rank markers
                if square.get_file() == File::A {
                    let text_style = TextStyle {
                        color: if rank.to_index() % 2 == 0 { COLOR_WHITE } else { COLOR_BLACK },
                        font_size: BOARD_TEXT_FONT_SIZE,
                        font: font.clone(),
                    };
                    cmds.spawn(NodeBundle {
                        style: Style {
                            position_type: PositionType::Absolute,
                            position: pos_top_left,
                            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::FlexStart,
                            ..default()
                        },
                        z_index: ZIndex::Local(Z_NOTATION_TEXT),
                        ..default()
                    })
                    .with_children(|cmds| {
                        cmds.spawn(TextBundle {
                            text: Text::from_section(ui_square.rank_char(), text_style),
                            style: Style { margin: rank_label_margins, ..default() },
                            ..default()
                        });
                    });
                }

                // Highlight tile
                let hl_tile_entity = cmds
                    .spawn((
                        HighlightTile,
                        ui_square,
                        NodeBundle {
                            background_color: COLOR_HIGHLIGHT.into(),
                            style: Style {
                                position_type: PositionType::Absolute,
                                position: pos_top_left,
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                ..default()
                            },
                            visibility: Visibility { is_visible: false },
                            z_index: ZIndex::Local(Z_HIGHLIGHT_TILE),
                            ..default()
                        },
                    ))
                    .id();
                assert!(
                    board_state.highlights.insert(square, hl_tile_entity).is_none(),
                    "Failed to insert highlight tile into state: tile already at this square"
                );

                // Move hint
                let move_entity = cmds
                    .spawn((
                        ui_square,
                        NodeBundle {
                            style: Style {
                                position_type: PositionType::Absolute,
                                position: pos_top_left,
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            visibility: Visibility { is_visible: false },
                            z_index: ZIndex::Local(Z_MOVE_HINT),
                            ..default()
                        },
                    ))
                    .with_children(|cmds| {
                        cmds.spawn(ImageBundle {
                            image: UiImage(move_hint_texture.clone()),
                            style: Style {
                                size: Size::new(
                                    Val::Percent(100.0 / 3.0),
                                    Val::Percent(100.0 / 3.0),
                                ),
                                ..default()
                            },
                            ..default()
                        });
                    })
                    .id();

                // Capture hint
                let capture_entity = cmds
                    .spawn((
                        ui_square,
                        ImageBundle {
                            image: UiImage(capture_hint_texture.clone()),
                            style: Style {
                                position_type: PositionType::Absolute,
                                position: pos_top_left,
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                ..default()
                            },
                            visibility: Visibility { is_visible: false },
                            z_index: ZIndex::Local(Z_MOVE_HINT),
                            ..default()
                        },
                    ))
                    .id();

                let hint = MoveHints { capture_entity, move_entity };
                assert!(
                    board_state.move_hints.insert(square, hint).is_none(),
                    "Failed to insert board hint into state: hint already at this square"
                );

                // Piece
                if let Some((image_path, color, typ)) = square.starting_piece_info() {
                    let (piece_color, piece_type) = (PieceColor(color), PieceType(typ));
                    let piece_entity = cmds
                        .spawn((
                            UiPiece::new(piece_color, piece_type),
                            ui_square,
                            ImageBundle {
                                image: UiImage(asset_server.load(image_path)),
                                style: Style {
                                    position_type: PositionType::Absolute,
                                    position: pos_top_left,
                                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                    ..default()
                                },
                                z_index: ZIndex::Local(Z_PIECE),
                                ..default()
                            },
                        ))
                        .id();
                    assert!(
                        board_state
                            .pieces
                            .insert(square, BoardPiece::new(piece_entity, piece_color, piece_type))
                            .is_none(),
                        "Failed to insert board piece into state: piece already at this square"
                    );
                }
            })
            .id();
        commands.entity(board).add_child(tile_entity);
        assert!(
            board_state.tiles.insert(square, tile_entity).is_none(),
            "Failed to insert board tile into state: tile already at this square"
        );
    }
}

pub fn spawn_drag_container(mut commands: Commands) {
    commands.spawn((
        DragContainer,
        NodeBundle { z_index: ZIndex::Global(Z_PIECE_SELECTED), ..default() },
    ));
}
