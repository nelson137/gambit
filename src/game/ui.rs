use bevy::prelude::*;

use crate::utils::NoopExts;

use super::{
    board::{BoardPlugin, CapturePlugin, PromotionPlugin},
    consts::{MIN_BOARD_SIZE, UI_GAP_VAL},
    eval_bar::EvaluationBarPlugin,
    menu::GameMenuUiPlugin,
    mouse::MouseUiPlugin,
    panels::UiPanelsPlugin,
};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            .add_plugins(GameMenuUiPlugin)
            .add_plugins(MouseUiPlugin)
            .add_plugins(BoardPlugin)
            .add_plugins(CapturePlugin)
            .add_plugins(UiPanelsPlugin)
            .add_plugins(EvaluationBarPlugin)
            .add_plugins(PromotionPlugin)
            .add_systems(Startup, ui.spawn())
            .noop();
    }
}

#[derive(Component, Clone, Default)]
pub struct Ui;

#[derive(Component, Clone, Default)]
pub struct EvaluationBarContainer;

#[derive(Component, Clone, Default)]
pub struct BoardAndPanelsContainer;

#[derive(Component, Clone, Default)]
pub struct BoardContainer;

fn ui() -> impl Scene {
    bsn! {
        #UiWrapper
        Node {
            width: percent(100.0),
            height: percent(100.0),
            position_type: PositionType::Absolute,
            left: percent(0.0),
            top: percent(0.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
        }
        Children [
            (
                #Ui
                Ui
                Node {
                    height: percent(100.0),
                    padding: { UI_GAP_VAL.all() },
                    flex_direction: FlexDirection::Row,
                }
                Children [
                    (
                        #EvaluationBarContainer
                        EvaluationBarContainer
                        Node {
                            margin: { UI_GAP_VAL.right() },
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            row_gap: UI_GAP_VAL,
                        }
                        Children [
                            super::eval_bar::eval_bar_spacer_top(),
                            super::eval_bar::eval_bar(),
                            super::eval_bar::eval_bar_spacer_bottom(),
                        ]
                    ),
                    (
                        #BoardAndPanelsContainer
                        BoardAndPanelsContainer
                        Node { flex_direction: FlexDirection::Column }
                        Children [
                            super::panels::black_panel(),
                            (
                                #BoardContainer
                                BoardContainer
                                Node {
                                    flex_grow: 1.0,
                                    min_width: MIN_BOARD_SIZE,
                                    min_height: MIN_BOARD_SIZE,
                                }
                                Children [
                                    super::board::board()
                                ]
                            ),
                            super::panels::white_panel(),
                        ]
                    )
                ]
            )
        ]
    }
}
