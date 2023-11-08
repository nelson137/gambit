use bevy::{ecs::system::Command, prelude::*};

use crate::{debug_name_f, game::consts::Z_HIGHLIGHT_TILE};

use super::{BoardState, Square};

#[derive(Component)]
pub struct HighlightTile;

/// The color used to highlight tiles.
pub const COLOR_HIGHLIGHT: Color = Color::rgba(1.0, 1.0, 0.0, 0.5);

#[derive(Deref, DerefMut)]
pub struct ShowHighlight(pub Entity);

impl Command for ShowHighlight {
    fn apply(self, world: &mut World) {
        if let Some(mut vis) = world.entity_mut(*self).get_mut::<Visibility>() {
            *vis = Visibility::Visible;
        }
    }
}

#[derive(Deref, DerefMut)]
pub struct HideHighlight(pub Option<Entity>);

impl Command for HideHighlight {
    fn apply(self, world: &mut World) {
        let Some(entity) = *self else { return };
        if let Some(mut vis) = world.entity_mut(entity).get_mut::<Visibility>() {
            *vis = Visibility::Hidden;
        }
    }
}

pub fn spawn_highlight_tiles(mut commands: Commands, mut board_state: ResMut<BoardState>) {
    let top = Val::Px(0.0);
    let left = Val::Px(0.0);

    for square in chess::ALL_SQUARES {
        let square = Square::new(square);

        let hl_tile_entity = commands
            .spawn((
                HighlightTile,
                debug_name_f!("Highlight Tile ({square})"),
                square,
                NodeBundle {
                    background_color: COLOR_HIGHLIGHT.into(),
                    style: Style {
                        position_type: PositionType::Absolute,
                        top,
                        left,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    visibility: Visibility::Hidden,
                    z_index: ZIndex::Global(Z_HIGHLIGHT_TILE),
                    ..default()
                },
            ))
            .id();

        board_state.set_highlight(square, hl_tile_entity);
        commands.entity(board_state.tile(square)).add_child(hl_tile_entity);
    }
}
