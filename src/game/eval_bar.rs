use bevy::prelude::*;

use crate::{
    debug_name,
    game::{
        consts::{CAPTURES_PANEL_HEIGHT, MIN_BOARD_SIZE},
        ui::EvaluationBarContainer,
    },
    utils::{NoopExts, ReparentInTag},
};

use super::ui::spawn_ui;

pub struct EvaluationBarPlugin;

impl Plugin for EvaluationBarPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            .add_message::<EvaluationUpdate>()
            .add_systems(Startup, spawn_eval_bar.after(spawn_ui))
            .add_systems(Update, update_eval_bar)
            .noop();
    }
}

#[derive(Component)]
#[require(BackgroundColor(Color::WHITE))]
pub struct EvaluationBar;

#[derive(Message)]
pub struct EvaluationUpdate(pub f32);

pub fn spawn_eval_bar(mut commands: Commands) {
    const SPACER_H: Val = Val::Px(CAPTURES_PANEL_HEIGHT);
    let spacer_bundle = || Node { height: SPACER_H, flex_shrink: 0.0, ..default() };

    let spacer_top =
        commands.spawn((debug_name!("Evaluation Bar Spacer (Top)"), spacer_bundle())).id();

    let bar = commands
        .spawn((
            debug_name!("Evaluation Bar Background (black)"),
            Node {
                position_type: PositionType::Relative,
                width: px(20.0),
                flex_grow: 1.0,
                min_height: MIN_BOARD_SIZE,
                ..default()
            },
            BackgroundColor(Color::srgb_u8(0x40, 0x3d, 0x39)),
            children![(
                debug_name!("Evaluation Bar (white)"),
                EvaluationBar,
                Node {
                    position_type: PositionType::Absolute,
                    bottom: px(0.0),
                    width: percent(100.0),
                    height: percent(50.0),
                    ..default()
                },
            )],
        ))
        .id();

    let spacer_bot =
        commands.spawn((debug_name!("Evaluation Bar Spacer (Bottom)"), spacer_bundle())).id();

    commands.reparent_in_tag::<EvaluationBarContainer>([spacer_top, bar, spacer_bot]);
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
