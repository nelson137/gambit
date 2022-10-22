#![allow(clippy::type_complexity)]

use bevy::{audio::AudioPlugin, prelude::*};

mod assets;
mod data;
mod systems;
mod util;
mod window;

use data::{BoardState, MouseSquare, MouseWorldPosition, ShowingMovesFor, COLOR_BG};
use systems::{
    click_handler, hints_hide, hints_show, mouse_hover, mouse_screen_position_to_world,
    mouse_world_position_to_square, piece_drag_and_drop, piece_move, resize_window, selections,
    setup_board, setup_camera, update_translation_for_square,
};
use window::{WIN_HEIGHT, WIN_WIDTH};

fn main() {
    App::new()
        // Plugins
        .add_plugins_with(DefaultPlugins, |plugins| plugins.disable::<AudioPlugin>())
        // Resources
        .insert_resource(WindowDescriptor {
            title: "Gambit".into(),
            width: WIN_WIDTH,
            height: WIN_HEIGHT,
            resizable: false,
            ..default()
        })
        .insert_resource(ClearColor(COLOR_BG))
        .init_resource::<BoardState>()
        .init_resource::<MouseWorldPosition>()
        .init_resource::<MouseSquare>()
        .init_resource::<ShowingMovesFor>()
        // Startup Systems
        .add_startup_system(resize_window)
        .add_startup_system(setup_camera)
        .add_startup_system(setup_board)
        // Systems
        .add_system_set_to_stage(
            CoreStage::PreUpdate,
            SystemSet::new()
                .with_system(mouse_screen_position_to_world)
                .with_system(mouse_world_position_to_square.after(mouse_screen_position_to_world))
                .with_system(mouse_hover.after(mouse_world_position_to_square)),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new().with_system(update_translation_for_square),
        )
        .add_system(click_handler)
        .add_system(selections.after(click_handler))
        .add_system(piece_drag_and_drop.after(click_handler))
        .add_system(piece_move.after(piece_drag_and_drop))
        .add_system(hints_hide.after(click_handler))
        .add_system(hints_show.after(hints_hide))
        // Run
        .run();
}
