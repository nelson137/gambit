use bevy::prelude::*;

use self::{
    handler::{mouse_handler, update_drag_container},
    position::{
        mouse_screen_position_to_world, mouse_world_position_to_square, MouseSquare,
        MouseWorldPosition,
    },
};

pub mod handler;
pub mod position;

pub struct MouseLogicPlugin;

impl Plugin for MouseLogicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MouseWorldPosition>()
            .init_resource::<MouseSquare>()
            .add_system_set_to_stage(
                CoreStage::PreUpdate,
                SystemSet::new().with_system(mouse_screen_position_to_world).with_system(
                    mouse_world_position_to_square.after(mouse_screen_position_to_world),
                ),
            )
            .add_system(mouse_handler)
            .add_system(update_drag_container);
    }
}
