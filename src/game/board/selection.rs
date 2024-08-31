use bevy::{
    ecs::{
        component::{ComponentHooks, ComponentId, StorageType},
        world::DeferredWorld,
    },
    prelude::*,
};

use crate::{
    game::{board::MovePiece, menu::MenuState, mouse::Dragging, LoadGame},
    utils::NoopExts,
};

use super::{BoardState, HighlightTile, Hint, Square};

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            // Resources
            .init_resource::<SelectionState>()
            // Events
            .add_event::<MouseSelectionEvent>()
            .add_event::<SelectionEvent>()
            // Observers
            .observe(unset_selections_on_load_game)
            .observe(handle_selection_events)
            // Systems
            .add_systems(Update, handle_mouse_selection_events.run_if(in_state(MenuState::Game)))
            .noop();
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Resource)]
pub enum SelectionState {
    #[default]
    Unselected,
    SelectingDragging(Square),
    Selected(Square),
    SelectedDragging(Square),
}

#[derive(Debug)]
pub enum SelectionStateAction {
    ChangeSelection(Square),
    DropSelect(Square),
    Move { from_sq: Square, to_sq: Square, animate: bool },
    None,
    StartSelectedDragging(Square),
    StartSelectingDragging(Square),
    Unselect(Square),
}

#[derive(Clone, Copy, Debug, Event)]
pub enum MouseSelectionEvent {
    MouseDown(Square),
    MouseUp(Square),
}

fn handle_mouse_selection_events(
    mut commands: Commands,
    mut selection_state: ResMut<SelectionState>,
    board_state: Res<BoardState>,
    mut event_reader: EventReader<MouseSelectionEvent>,
) {
    for &event in event_reader.read() {
        trace!(?event, "Mouse selection event");
        let action = match *selection_state {
            SelectionState::Unselected => match event {
                MouseSelectionEvent::MouseDown(square) => {
                    if board_state.has_piece_at(square) {
                        SelectionStateAction::StartSelectingDragging(square)
                    } else {
                        SelectionStateAction::None
                    }
                }
                MouseSelectionEvent::MouseUp(_) => SelectionStateAction::None,
            },
            SelectionState::SelectingDragging(selecting_sq) => match event {
                MouseSelectionEvent::MouseDown(_) => todo!("reset previous drag target"), // TODO
                MouseSelectionEvent::MouseUp(square) => {
                    if board_state.move_is_valid(selecting_sq, square) {
                        SelectionStateAction::Move {
                            from_sq: selecting_sq,
                            to_sq: square,
                            animate: false,
                        }
                    } else {
                        SelectionStateAction::DropSelect(selecting_sq)
                    }
                }
            },
            SelectionState::Selected(selected_sq) => match event {
                MouseSelectionEvent::MouseDown(square) => {
                    if square == selected_sq {
                        SelectionStateAction::StartSelectedDragging(selected_sq)
                    } else if board_state.move_is_valid(selected_sq, square) {
                        SelectionStateAction::Move {
                            from_sq: selected_sq,
                            to_sq: square,
                            animate: true,
                        }
                    } else if board_state.has_piece_at(square) {
                        SelectionStateAction::ChangeSelection(square)
                    } else {
                        SelectionStateAction::Unselect(selected_sq)
                    }
                }
                MouseSelectionEvent::MouseUp(_) => SelectionStateAction::None,
            },
            SelectionState::SelectedDragging(selected_sq) => match event {
                MouseSelectionEvent::MouseDown(_) => todo!("reset previous drag target"), // TODO
                MouseSelectionEvent::MouseUp(square) => {
                    if square == selected_sq {
                        SelectionStateAction::Unselect(selected_sq)
                    } else if board_state.move_is_valid(selected_sq, square) {
                        SelectionStateAction::Move {
                            from_sq: selected_sq,
                            to_sq: square,
                            animate: false,
                        }
                    } else {
                        SelectionStateAction::DropSelect(selected_sq)
                    }
                }
            },
        };

        trace!(?action, "Selection action");
        match action {
            SelectionStateAction::None => {}
            SelectionStateAction::ChangeSelection(to_sq) => {
                // Start dragging piece
                let piece = board_state.piece(to_sq);
                commands.entity(piece).insert(Dragging { original_square: to_sq });
                // Update selection & hints
                let hl_tile = board_state.highlight(to_sq);
                let hints = board_state.calculate_valid_moves(to_sq);
                commands.trigger(SelectionEvent::UpdateSelection { highlight: hl_tile, hints });
                // Set state to SelectingDragging
                *selection_state = SelectionState::SelectingDragging(to_sq);
            }
            SelectionStateAction::DropSelect(square) => {
                // Drop piece
                let piece = board_state.piece(square);
                commands.entity(piece).remove::<Dragging>();
                // Set state to Selected
                *selection_state = SelectionState::Selected(square);
            }
            SelectionStateAction::Move { from_sq, to_sq, animate } => {
                let piece = board_state.piece(from_sq);
                commands.trigger_targets(MovePiece::new(from_sq, to_sq, None, animate), piece);
                // Set state to Unselected
                *selection_state = SelectionState::Unselected;
            }
            SelectionStateAction::StartSelectedDragging(square) => {
                // Start dragging piece
                let piece = board_state.piece(square);
                commands.entity(piece).insert(Dragging { original_square: square });
                // Set state to SelectedDragging
                *selection_state = SelectionState::SelectedDragging(square);
            }
            SelectionStateAction::StartSelectingDragging(square) => {
                // Start dragging piece
                let piece = board_state.piece(square);
                commands.entity(piece).insert(Dragging { original_square: square });
                // Update selection & hints
                let hl_tile = board_state.highlight(square);
                let hints = board_state.calculate_valid_moves(square);
                commands.trigger(SelectionEvent::UpdateSelection { highlight: hl_tile, hints });
                // Set state to SelectingDragging
                *selection_state = SelectionState::SelectingDragging(square);
            }
            SelectionStateAction::Unselect(square) => {
                // Drop piece
                let piece = board_state.piece(square);
                commands.entity(piece).remove::<Dragging>();
                // Unselect square & remove hints
                commands.trigger(SelectionEvent::Unselect);
                // Set state to Unselected
                *selection_state = SelectionState::Unselected;
            }
        };
    }
}

#[derive(Component)]
pub struct Selected;

#[derive(Component)]
pub struct EnabledHint;

#[derive(Component)]
pub struct LastMove;

#[derive(Clone, Debug, Event)]
#[allow(dead_code)]
pub enum SelectionEvent {
    UpdateSelection { highlight: Entity, hints: Vec<Entity> },
    Unselect,
    UpdateLastMove(Square, Square),
    UnsetLastMove,
    UnsetAll,
}

fn unset_selections_on_load_game(_trigger: Trigger<LoadGame>, mut commands: Commands) {
    commands.trigger(SelectionEvent::UnsetAll);
}

fn handle_selection_events(
    trigger: Trigger<SelectionEvent>,
    mut commands: Commands,
    board_state: Res<BoardState>,
    q_selection: Query<Entity, (With<HighlightTile>, With<Selected>)>,
    q_last_move: Query<Entity, (With<HighlightTile>, With<LastMove>)>,
    q_selected_hints: Query<Entity, (With<Hint>, With<EnabledHint>)>,
) {
    let unset_selection = |commands: &mut Commands| {
        for entity in &q_selection {
            let mut entity_cmds = commands.entity(entity);
            entity_cmds.remove::<Selected>();
            if !q_last_move.contains(entity) {
                entity_cmds.remove::<ShowingIndicator>();
            }
        }
    };

    let unset_last_move = |commands: &mut Commands| {
        for entity in &q_last_move {
            commands.entity(entity).remove::<(ShowingIndicator, LastMove)>();
        }
    };

    let unset_hints = |commands: &mut Commands| {
        for entity in &q_selected_hints {
            commands.entity(entity).remove::<(ShowingIndicator, EnabledHint)>();
        }
    };

    let event = trigger.event();
    trace!(?event, "Selection event");

    match event {
        SelectionEvent::UpdateSelection { highlight, hints } => {
            unset_selection(&mut commands);
            unset_hints(&mut commands);
            commands.entity(*highlight).insert((ShowingIndicator, Selected));
            for &entity in hints {
                commands.entity(entity).insert((ShowingIndicator, EnabledHint));
            }
        }
        SelectionEvent::Unselect => {
            unset_selection(&mut commands);
            unset_hints(&mut commands);
        }
        &SelectionEvent::UpdateLastMove(from_sq, to_sq) => {
            unset_last_move(&mut commands);
            let e1 = board_state.highlight(from_sq);
            let e2 = board_state.highlight(to_sq);
            commands.entity(e1).insert((ShowingIndicator, LastMove));
            commands.entity(e2).insert((ShowingIndicator, LastMove));
        }
        SelectionEvent::UnsetLastMove => unset_last_move(&mut commands),
        SelectionEvent::UnsetAll => {
            unset_selection(&mut commands);
            unset_hints(&mut commands);
            unset_last_move(&mut commands);
        }
    }
}

struct ShowingIndicator;

impl Component for ShowingIndicator {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;

    fn register_component_hooks(hooks: &mut ComponentHooks) {
        hooks.on_add(on_add_showing_indicator).on_remove(on_remove_showing_indicator);
    }
}

fn on_add_showing_indicator(mut world: DeferredWorld, entity: Entity, _cid: ComponentId) {
    world.commands().add(move |world: &mut World| {
        trace!(%entity, "show indicator");
        let mut entity = world.entity_mut(entity);
        let Some(mut vis) = entity.get_mut::<Visibility>() else { return };
        *vis = Visibility::Visible;
    });
}

fn on_remove_showing_indicator(mut world: DeferredWorld, entity: Entity, _cid: ComponentId) {
    world.commands().add(move |world: &mut World| {
        trace!(%entity, "hide indicator");
        let mut entity = world.entity_mut(entity);
        let Some(mut vis) = entity.get_mut::<Visibility>() else { return };
        *vis = Visibility::Hidden;
    });
}

#[cfg(test)]
mod tests {
    use bevy::ecs::world::Command;

    use crate::game::board::{PieceColor, PieceType};

    use super::*;

    mod utils {
        use bevy::utils::HashSet;

        use crate::game::{
            board::PieceMeta,
            core::{GameHeadlessPlugin, GameTestPlugin},
            menu::test::TestMenuStateInGamePlugin,
            mouse::DragContainer,
            ui::GameUiPlugin,
        };

        use super::*;

        pub fn build_app() -> App {
            let mut app = App::new();
            app.add_plugins((GameHeadlessPlugin, GameTestPlugin))
                .add_plugins(TestMenuStateInGamePlugin)
                .add_plugins(GameUiPlugin)
                .add_plugins(SelectionPlugin);
            app.update();
            app
        }

        pub fn get_tagged_entity<Tag: Component>(app: &mut App) -> Entity {
            app.world_mut().query_filtered::<Entity, With<Tag>>().single(app.world())
        }

        pub trait TestAppExts {
            fn board_state(&self) -> &BoardState;

            fn set_state(&mut self, state: SelectionState);
            fn set_selected(&mut self, square: Square);
            fn set_hints(&mut self, squares: impl IntoIterator<Item = Square>);
            fn set_piece_to_drag_container(&mut self, square: Square);

            fn assert_state(&self, expected: SelectionState);
            fn assert_piece_in_drag_container(&mut self, expected: Square);
            fn assert_piece_move_marker(&mut self, expected: MovePiece);
            fn assert_selected(&mut self, expected: Option<Square>);
            fn assert_hints(&mut self, expected: impl IntoIterator<Item = Square>);
            fn assert_piece_on_tile(&self, sq: Square, color: PieceColor, typ: PieceType);
        }

        impl TestAppExts for App {
            fn board_state(&self) -> &BoardState {
                self.world().resource::<BoardState>()
            }

            fn set_state(&mut self, state: SelectionState) {
                *self.world_mut().resource_mut::<SelectionState>() = state;
            }

            fn set_selected(&mut self, square: Square) {
                let entity = self.board_state().highlight(square);
                self.world_mut().entity_mut(entity).insert(Selected);
            }

            fn set_hints(&mut self, squares: impl IntoIterator<Item = Square>) {
                for sq in squares.into_iter() {
                    // Test assertions don't differentiate between move hints and capture hints
                    let entity = self.board_state().tile_hints(sq).move_entity;
                    self.world_mut().entity_mut(entity).insert(EnabledHint);
                }
            }

            fn set_piece_to_drag_container(&mut self, square: Square) {
                let parent = get_tagged_entity::<DragContainer>(self);
                let child = self.board_state().piece(square);
                PushChild { parent, child }.apply(self.world_mut());
            }

            fn assert_state(&self, expected: SelectionState) {
                let actual = *self.world().resource::<SelectionState>();
                assert_eq!(actual, expected);
            }

            fn assert_piece_in_drag_container(&mut self, piece: Square) {
                let piece = self.board_state().piece(piece);
                let expected = Some(HashSet::from_iter([piece]));

                let drag_container = get_tagged_entity::<DragContainer>(self);
                let children = self.world().entity(drag_container).get::<Children>();
                let actual = children.map(|c| HashSet::from_iter(c.iter().copied()));

                assert_eq!(actual, expected);
            }

            fn assert_piece_move_marker(&mut self, expected: MovePiece) {
                let actual = self.world_mut().resource_mut::<Events<MovePiece>>().drain().next();
                assert_eq!(actual, Some(expected), "move piece event on piece");
            }

            fn assert_selected(&mut self, expected: Option<Square>) {
                if let Some(expected) = expected {
                    let entity = self.board_state().highlight(expected);
                    let is_selected = self.world().entity(entity).contains::<Selected>();
                    assert!(is_selected, "selected highlight tile entitiy");
                } else {
                    let mut q = self
                        .world_mut()
                        .query_filtered::<(), (With<HighlightTile>, With<Selected>)>();
                    let actual_count = q.iter(self.world()).count();
                    assert_eq!(actual_count, 0, "count of selected highlight tile entities");
                }
            }

            fn assert_hints(&mut self, expected: impl IntoIterator<Item = Square>) {
                let board_state = self.board_state();

                let expected = expected
                    .into_iter()
                    .map(|sq| board_state.tile_hints(sq).move_entity)
                    .collect::<HashSet<_>>();

                let mut q =
                    self.world_mut().query_filtered::<Entity, (With<Hint>, With<EnabledHint>)>();
                let actual = q.iter(self.world()).collect::<HashSet<_>>();

                assert_eq!(actual, expected, "enabled hint entities");
            }

            fn assert_piece_on_tile(&self, sq: Square, color: PieceColor, typ: PieceType) {
                let board_state = self.board_state();
                let piece = board_state.piece(sq);
                let tile = board_state.tile(sq);
                let piece = self.world().entity(piece);
                let piece_typ = piece.get::<PieceMeta>();
                assert_eq!(
                    piece_typ,
                    Some(&PieceMeta::new(color, typ)),
                    "piece at square {sq} in piece state is not a {color} {typ}"
                );
                let piece_parent = piece.get::<Parent>().map(Parent::get);
                assert_eq!(
                    piece_parent,
                    Some(tile),
                    "piece at square {sq} in piece state is a child of {sq} tile"
                );
            }
        }
    }
    use utils::*;

    #[test]
    fn unselected_is_initial_state() {
        let app = build_app();

        app.assert_state(SelectionState::Unselected);
    }

    #[test]
    fn unselected_starts_dragging_on_mouse_down_on_piece() {
        let mut app = build_app();

        app.world_mut().send_event(MouseSelectionEvent::MouseDown(Square::D2));
        app.update();

        app.assert_state(SelectionState::SelectingDragging(Square::D2));
        app.assert_piece_in_drag_container(Square::D2);
        app.assert_selected(Some(Square::D2));
        app.assert_hints([Square::D3, Square::D4]);
    }

    #[test]
    fn unselected_does_nothing_on_mouse_down_when_not_on_piece() {
        let mut app = build_app();

        app.world_mut().send_event(MouseSelectionEvent::MouseDown(Square::D4));
        app.update();

        app.assert_state(SelectionState::Unselected);
        app.assert_selected(None);
        app.assert_hints([]);
    }

    #[test]
    fn unselected_does_nothing_on_mouse_up() {
        let mut app = build_app();

        app.world_mut().send_event(MouseSelectionEvent::MouseUp(Square::A1));
        app.update();

        app.assert_state(SelectionState::Unselected);
        app.assert_selected(None);
        app.assert_hints([]);
    }

    #[test]
    fn selecting_dragging_makes_move_on_mouse_up_when_on_valid_move() {
        let mut app = build_app();
        app.set_state(SelectionState::SelectingDragging(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);
        app.set_piece_to_drag_container(Square::D2);

        app.world_mut().send_event(MouseSelectionEvent::MouseUp(Square::D4));
        app.update();

        app.assert_state(SelectionState::Unselected);
        app.assert_piece_move_marker(MovePiece::new(Square::D2, Square::D4, None, false));
    }

    #[test]
    fn selecting_dragging_selects_on_mouse_up_when_not_a_move() {
        let mut app = build_app();
        app.set_state(SelectionState::SelectingDragging(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);
        app.set_piece_to_drag_container(Square::D2);

        app.world_mut().send_event(MouseSelectionEvent::MouseUp(Square::A8));
        app.update();

        app.assert_state(SelectionState::Selected(Square::D2));
        app.assert_piece_on_tile(Square::D2, PieceColor::WHITE, PieceType::PAWN);
        app.assert_selected(Some(Square::D2));
        app.assert_hints([Square::D3, Square::D4]);
    }

    #[test]
    fn selected_starts_dragging_on_mouse_down_on_selected_tile() {
        let mut app = build_app();
        app.set_state(SelectionState::Selected(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);

        app.world_mut().send_event(MouseSelectionEvent::MouseDown(Square::D2));
        app.update();

        app.assert_state(SelectionState::SelectedDragging(Square::D2));
        app.assert_piece_in_drag_container(Square::D2);
        app.assert_selected(Some(Square::D2));
        app.assert_hints([Square::D3, Square::D4]);
    }

    #[test]
    fn selected_makes_move_on_mouse_down_when_on_valid_move() {
        let mut app = build_app();
        app.set_state(SelectionState::Selected(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);

        app.world_mut().send_event(MouseSelectionEvent::MouseDown(Square::D4));
        app.update();

        app.assert_state(SelectionState::Unselected);
        app.assert_piece_move_marker(MovePiece::new(Square::D2, Square::D4, None, true));
    }

    #[test]
    fn selected_starts_dragging_on_mouse_down_on_different_piece() {
        let mut app = build_app();
        app.set_state(SelectionState::Selected(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);

        app.world_mut().send_event(MouseSelectionEvent::MouseDown(Square::H2));
        app.update();

        app.assert_state(SelectionState::SelectingDragging(Square::H2));
        app.assert_piece_on_tile(Square::D2, PieceColor::WHITE, PieceType::PAWN);
        app.assert_piece_in_drag_container(Square::H2);
        app.assert_selected(Some(Square::H2));
        app.assert_hints([Square::H3, Square::H4]);
    }

    #[test]
    fn selected_unselects_on_mouse_down_when_not_on_a_piece() {
        let mut app = build_app();
        app.set_state(SelectionState::Selected(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);

        app.world_mut().send_event(MouseSelectionEvent::MouseDown(Square::E4));
        app.update();

        app.assert_state(SelectionState::Unselected);
        app.assert_selected(None);
        app.assert_hints([]);
    }

    #[test]
    fn selected_does_nothing_on_mouse_up() {
        let mut app = build_app();
        app.set_state(SelectionState::Selected(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);

        app.world_mut().send_event(MouseSelectionEvent::MouseUp(Square::A1));
        app.update();

        app.assert_state(SelectionState::Selected(Square::D2));
        app.assert_selected(Some(Square::D2));
        app.assert_hints([Square::D3, Square::D4]);
    }

    #[test]
    fn selected_dragging_unselects_on_mouse_up_on_selected_tile() {
        let mut app = build_app();
        app.set_state(SelectionState::SelectedDragging(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);
        app.set_piece_to_drag_container(Square::D2);

        app.world_mut().send_event(MouseSelectionEvent::MouseUp(Square::D2));
        app.update();

        app.assert_state(SelectionState::Unselected);
        app.assert_piece_on_tile(Square::D2, PieceColor::WHITE, PieceType::PAWN);
        app.assert_selected(None);
        app.assert_hints([]);
    }

    #[test]
    fn selected_dragging_makes_move_on_mouse_up_when_on_valid_move() {
        let mut app = build_app();
        app.set_state(SelectionState::SelectedDragging(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);
        app.set_piece_to_drag_container(Square::D2);

        app.world_mut().send_event(MouseSelectionEvent::MouseUp(Square::D4));
        app.update();

        app.assert_state(SelectionState::Unselected);
        app.assert_piece_move_marker(MovePiece::new(Square::D2, Square::D4, None, false));
    }

    #[test]
    fn selected_dragging_reselects_on_mouse_up_when_not_a_move() {
        let mut app = build_app();
        app.set_state(SelectionState::SelectedDragging(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);
        app.set_piece_to_drag_container(Square::D2);

        app.world_mut().send_event(MouseSelectionEvent::MouseUp(Square::A8));
        app.update();

        app.assert_state(SelectionState::Selected(Square::D2));
        app.assert_piece_on_tile(Square::D2, PieceColor::WHITE, PieceType::PAWN);
        app.assert_selected(Some(Square::D2));
        app.assert_hints([Square::D3, Square::D4]);
    }
}
