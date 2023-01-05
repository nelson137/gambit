use std::sync::Arc;

use bevy::{ecs::system::Command, prelude::*};
use chess::{File, Rank};

use crate::{
    assets::SquareStartingPieceInfo,
    data::{
        BoardPiece, BoardState, DragContainer, HighlightTile, MainCamera, MoveHints, PieceColor,
        PieceType, Tile, Ui, UiBoard, UiPiece, UiSquare, BOARD_TEXT_FONT_SIZE, COLOR_BLACK,
        COLOR_HIGHLIGHT, COLOR_WHITE, Z_HIGHLIGHT_TILE, Z_MOVE_HINT, Z_NOTATION_TEXT, Z_PIECE,
        Z_PIECE_SELECTED, Z_TILE,
    },
    game::captures::CaptureState,
};

pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default()).insert(MainCamera); // ::new_with_far(1000.0)
}

#[derive(Clone, StageLabel)]
pub enum SpawnStage {
    Phase1,
    Phase2,
    Phase3,
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
    let move_hint_texture = asset_server.load("images/hints/move.png");
    let capture_hint_texture = asset_server.load("images/hints/capture.png");
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
                board_state.set_highlight(square, hl_tile_entity);

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

                board_state.set_move_hints(square, MoveHints { capture_entity, move_entity });

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
                    board_state
                        .set_piece(square, BoardPiece::new(piece_entity, piece_color, piece_type));
                }
            })
            .id();
        commands.entity(board).add_child(tile_entity);
        board_state.set_tile(square, tile_entity);
    }
}

pub fn spawn_panels(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut capture_state: ResMut<CaptureState>,
    q_ui: Query<Entity, With<Ui>>,
) {
    const BLACK: PieceColor = PieceColor(chess::Color::Black);
    const WHITE: PieceColor = PieceColor(chess::Color::White);
    const PAWN: PieceType = PieceType(chess::Piece::Pawn);
    const BISHOP: PieceType = PieceType(chess::Piece::Bishop);
    const KNIGHT: PieceType = PieceType(chess::Piece::Knight);
    const ROOK: PieceType = PieceType(chess::Piece::Rook);
    const QUEEN: PieceType = PieceType(chess::Piece::Queen);
    let capture_state = Arc::get_mut(&mut capture_state).unwrap();
    capture_state[BLACK][PAWN].image_handles.extend([
        asset_server.load("images/captures/white-pawns-8.png"),
        asset_server.load("images/captures/white-pawns-7.png"),
        asset_server.load("images/captures/white-pawns-6.png"),
        asset_server.load("images/captures/white-pawns-5.png"),
        asset_server.load("images/captures/white-pawns-4.png"),
        asset_server.load("images/captures/white-pawns-3.png"),
        asset_server.load("images/captures/white-pawns-2.png"),
        asset_server.load("images/captures/white-pawns-1.png"),
    ]);
    capture_state[BLACK][BISHOP].image_handles.extend([
        asset_server.load("images/captures/white-bishops-2.png"),
        asset_server.load("images/captures/white-bishops-1.png"),
    ]);
    capture_state[BLACK][KNIGHT].image_handles.extend([
        asset_server.load("images/captures/white-knights-2.png"),
        asset_server.load("images/captures/white-knights-1.png"),
    ]);
    capture_state[BLACK][ROOK].image_handles.extend([
        asset_server.load("images/captures/white-rooks-2.png"),
        asset_server.load("images/captures/white-rooks-1.png"),
    ]);
    capture_state[BLACK][QUEEN]
        .image_handles
        .push(asset_server.load("images/captures/white-queen.png"));
    capture_state[WHITE][PAWN].image_handles.extend([
        asset_server.load("images/captures/black-pawns-8.png"),
        asset_server.load("images/captures/black-pawns-7.png"),
        asset_server.load("images/captures/black-pawns-6.png"),
        asset_server.load("images/captures/black-pawns-5.png"),
        asset_server.load("images/captures/black-pawns-4.png"),
        asset_server.load("images/captures/black-pawns-3.png"),
        asset_server.load("images/captures/black-pawns-2.png"),
        asset_server.load("images/captures/black-pawns-1.png"),
    ]);
    capture_state[WHITE][BISHOP].image_handles.extend([
        asset_server.load("images/captures/black-bishops-2.png"),
        asset_server.load("images/captures/black-bishops-1.png"),
    ]);
    capture_state[WHITE][KNIGHT].image_handles.extend([
        asset_server.load("images/captures/black-knights-2.png"),
        asset_server.load("images/captures/black-knights-1.png"),
    ]);
    capture_state[WHITE][ROOK].image_handles.extend([
        asset_server.load("images/captures/black-rooks-2.png"),
        asset_server.load("images/captures/black-rooks-1.png"),
    ]);
    capture_state[WHITE][QUEEN]
        .image_handles
        .push(asset_server.load("images/captures/black-queen.png"));

    let container = commands
        .spawn(NodeBundle {
            style: Style {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .id();
    commands.entity(q_ui.single()).add_child(container);

    commands.add(PanelData { container, color: PieceColor(chess::Color::Black) });

    commands.add(PanelData { container, color: PieceColor(chess::Color::White) });
}

struct PanelData {
    container: Entity,
    color: PieceColor,
}

const CAPTURES_IMAGE_MARGIN: Val = Val::Px(8.0);

impl PanelData {
    fn into_bundle(self) -> impl Bundle {
        NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Px(32.0)),
                margin: UiRect {
                    top: CAPTURES_IMAGE_MARGIN,
                    bottom: CAPTURES_IMAGE_MARGIN,
                    ..default()
                },
                ..default()
            },
            ..default()
        }
    }
}

impl Command for PanelData {
    fn write(self, world: &mut World) {
        let color = self.color;

        let mut image_entities = Vec::with_capacity(5);

        let capture_state = Arc::clone(world.resource::<CaptureState>());
        world.entity_mut(self.container).with_children(|cmds| {
            cmds.spawn(self.into_bundle()).with_children(|cmds| {
                for cap_state in capture_state[color].iter() {
                    let handles = &cap_state.image_handles;
                    let entity = cmds
                        .spawn(ImageBundle {
                            image: UiImage(handles[handles.len() - 1].clone()),
                            style: Style {
                                display: Display::None,
                                margin: UiRect { left: CAPTURES_IMAGE_MARGIN, ..default() },
                                flex_shrink: 0.0,
                                ..default()
                            },
                            ..default()
                        })
                        .id();
                    image_entities.push(entity);
                }
            });
        });
        drop(capture_state);

        let mut capture_state = world.resource_mut::<CaptureState>();
        let state_entities = Arc::get_mut(&mut capture_state).unwrap();
        for (cap_state, entity) in state_entities[color].iter_mut().zip(image_entities) {
            cap_state.image_entity = entity;
        }
    }
}

pub fn spawn_drag_container(mut commands: Commands) {
    commands.spawn((
        DragContainer,
        NodeBundle { z_index: ZIndex::Global(Z_PIECE_SELECTED), ..default() },
    ));
}
