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
            .add_event::<EvaluationUpdate>()
            .add_systems(Startup, spawn_eval_bar.after(spawn_ui))
            .add_systems(Update, update_eval_bar)
            .noop();
    }
}

#[derive(Component)]
pub struct EvaluationBar;

#[derive(Event)]
pub struct EvaluationUpdate(pub f32);

pub fn spawn_eval_bar(mut commands: Commands) {
    const SPACER_H: Val = Val::Px(CAPTURES_PANEL_HEIGHT);
    let spacer_bundle = || NodeBundle {
        style: Style { height: SPACER_H, flex_shrink: 0.0, ..default() },
        ..default()
    };

    let spacer_top =
        commands.spawn((debug_name!("Evaluation Bar Spacer (Top)"), spacer_bundle())).id();

    let bar = commands
        .spawn((
            debug_name!("Evaluation Bar Background (black)"),
            NodeBundle {
                background_color: BackgroundColor(Color::srgb_u8(0x40, 0x3d, 0x39)),
                style: Style {
                    position_type: PositionType::Relative,
                    width: Val::Px(20.0),
                    flex_grow: 1.0,
                    min_height: MIN_BOARD_SIZE,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|cmds| {
            cmds.spawn((
                EvaluationBar,
                debug_name!("Evaluation Bar (white)"),
                NodeBundle {
                    background_color: BackgroundColor(Color::WHITE),
                    style: Style {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(50.0),
                        ..default()
                    },
                    ..default()
                },
            ));
        })
        .id();

    let spacer_bot =
        commands.spawn((debug_name!("Evaluation Bar Spacer (Bottom)"), spacer_bundle())).id();

    commands.reparent_in_tag::<EvaluationBarContainer>([spacer_top, bar, spacer_bot]);
}

pub fn update_eval_bar(
    mut reader: EventReader<EvaluationUpdate>,
    mut q_bar: Query<&mut Style, With<EvaluationBar>>,
) {
    let Ok(mut bar_style) = q_bar.get_single_mut() else { return };

    for update in reader.read() {
        // Map eval stat in [-10.0,10.0] to [0.0,100.0]
        let percent = (update.0 + 10.0).clamp(0.0, 20.0) * 5.0;
        bar_style.height = Val::Percent(percent);
    }
}
