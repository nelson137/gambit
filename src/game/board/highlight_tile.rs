use bevy::{ecs::system::Command, prelude::*};

use crate::{debug_name, game::consts::Z_HIGHLIGHT_TILE};

use super::{BoardLocation, BoardState};

#[derive(Component)]
pub struct HighlightTile;

/// The color used to highlight tiles.
pub const COLOR_HIGHLIGHT: Color = Color::rgba(1.0, 1.0, 0.0, 0.5);

#[derive(Deref, DerefMut)]
pub struct ShowHighlight(pub Entity);

impl Command for ShowHighlight {
    fn write(self, world: &mut World) {
        if let Some(mut vis) = world.entity_mut(*self).get_mut::<Visibility>() {
            vis.is_visible = true;
        }
    }
}

#[derive(Deref, DerefMut)]
pub struct HideHighlight(pub Entity);

impl Command for HideHighlight {
    fn write(self, world: &mut World) {
        if let Some(mut vis) = world.entity_mut(*self).get_mut::<Visibility>() {
            vis.is_visible = false;
        }
    }
}

pub fn spawn_highlight_tiles(mut commands: Commands, mut board_state: ResMut<BoardState>) {
    let pos_top_left = UiRect { top: Val::Px(0.0), left: Val::Px(0.0), ..default() };

    for square in chess::ALL_SQUARES {
        let location = BoardLocation::new(square);

        let hl_tile_entity = commands
            .spawn((
                HighlightTile,
                debug_name!("Highlight Tile ({square})"),
                location,
                NodeBundle {
                    background_color: COLOR_HIGHLIGHT.into(),
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: pos_top_left,
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        ..default()
                    },
                    visibility: Visibility::INVISIBLE,
                    z_index: ZIndex::Local(Z_HIGHLIGHT_TILE),
                    ..default()
                },
            ))
            .id();

        board_state.set_highlight(square, hl_tile_entity);
        commands.entity(board_state.tile(square)).add_child(hl_tile_entity);
    }
}
