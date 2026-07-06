use bevy::prelude::*;

use crate::{
    game::consts::{CAPTURES_PANEL_HEIGHT, MIN_BOARD_SIZE},
    utils::NoopExts,
};

pub struct EvaluationBarPlugin;

impl Plugin for EvaluationBarPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            // Messages
            .add_message::<EvaluationUpdate>()
            // Systems
            .add_systems(Update, update_eval_bar)
            .noop();
    }
}

#[derive(Component, Clone, Default)]
pub struct EvaluationBar;

#[derive(Message)]
pub struct EvaluationUpdate(pub f32);

pub fn eval_bar_spacer_top() -> impl Scene {
    bsn! {
        #EvaluationBarSpacerTop
        eval_bar_spacer()
    }
}

pub fn eval_bar_spacer_bottom() -> impl Scene {
    bsn! {
        #EvaluationBarSpacerBottom
        eval_bar_spacer()
    }
}

fn eval_bar_spacer() -> impl Scene {
    bsn! {
        Node { height: px(CAPTURES_PANEL_HEIGHT), flex_shrink: 0.0 }
    }
}

pub fn eval_bar() -> impl Scene {
    bsn! {
        #EvaluationBarBackground
        Node {
            position_type: PositionType::Relative,
            width: px(20.0),
            flex_grow: 1.0,
            min_height: MIN_BOARD_SIZE,
        }
        BackgroundColor(Color::srgb_u8(0x40, 0x3d, 0x39))
        Children [
            (
                #EvaluationBar
                EvaluationBar
                BackgroundColor(Color::WHITE)
                Node {
                    position_type: PositionType::Absolute,
                    bottom: px(0.0),
                    width: percent(100.0),
                    height: percent(50.0),
                }
            )
        ]
    }
}

pub fn update_eval_bar(
    mut reader: MessageReader<EvaluationUpdate>,
    mut q_bar: Query<&mut Node, With<EvaluationBar>>,
) {
    let Ok(mut bar_node) = q_bar.single_mut() else { return };

    for update in reader.read() {
        // Map eval stat in [-10.0,10.0] to [0.0,100.0]
        let p = (update.0 + 10.0).clamp(0.0, 20.0) * 5.0;
        bar_node.height = percent(p);
    }
}
