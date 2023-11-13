use bevy::prelude::*;

use crate::{
    game::{menu::MenuState, mouse::DragContainer, moves::StartMove},
    utils::AppNoop,
};

use super::{BoardState, HighlightTile, MoveHint, Square};

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            // Resources
            .init_resource::<SelectionState>()
            // Events
            .add_event::<MouseSelectionEvent>()
            .add_event::<SelectionEvent>()
            // Systems
            // TODO: handle_selection_events should run at the end of the set
            .add_systems(Update, handle_mouse_selection_events.run_if(in_state(MenuState::Game)))
            .add_systems(PostUpdate, handle_selection_events.run_if(in_state(MenuState::Game)))
            .add_systems(Last, update_indicators.run_if(in_state(MenuState::Game)))
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
    Move(Square, Square),
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
    q_drag_container: Query<Entity, With<DragContainer>>,
    mut selection_events: EventWriter<SelectionEvent>,
) {
    for &event in event_reader.read() {
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
                        SelectionStateAction::Move(selecting_sq, square)
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
                        SelectionStateAction::Move(selected_sq, square)
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
                        SelectionStateAction::Move(selected_sq, square)
                    } else {
                        SelectionStateAction::DropSelect(selected_sq)
                    }
                }
            },
        };

        match action {
            SelectionStateAction::None => {}
            SelectionStateAction::ChangeSelection(to_sq) => {
                // Re-parent piece to drag container
                let piece = board_state.piece(to_sq);
                commands.entity(piece).set_parent(q_drag_container.single());
                // Update selection & hints
                let hl_tile = board_state.highlight(to_sq);
                let hints = board_state.calculate_valid_moves(to_sq);
                selection_events
                    .send(SelectionEvent::UpdateSelection { highlight: hl_tile, hints });
                // Set state to SelectingDragging
                *selection_state = SelectionState::SelectingDragging(to_sq);
            }
            SelectionStateAction::DropSelect(square) => {
                // Re-parent piece back to its tile
                let piece = board_state.piece(square);
                let tile = board_state.tile(square);
                commands.entity(piece).set_parent(tile);
                // Set state to Selected
                *selection_state = SelectionState::Selected(square);
            }
            SelectionStateAction::Move(from_sq, to_sq) => {
                let piece = board_state.piece(from_sq);
                commands.entity(piece).insert(StartMove::new(from_sq, to_sq));
                // Set state to Unselected
                *selection_state = SelectionState::Unselected;
            }
            SelectionStateAction::StartSelectedDragging(square) => {
                // Re-parent piece to drag container
                let piece = board_state.piece(square);
                commands.entity(piece).set_parent(q_drag_container.single());
                // Set state to SelectedDragging
                *selection_state = SelectionState::SelectedDragging(square);
            }
            SelectionStateAction::StartSelectingDragging(square) => {
                // Re-parent piece to drag container
                let piece = board_state.piece(square);
                commands.entity(piece).set_parent(q_drag_container.single());
                // Update selection & hints
                let hl_tile = board_state.highlight(square);
                let hints = board_state.calculate_valid_moves(square);
                selection_events
                    .send(SelectionEvent::UpdateSelection { highlight: hl_tile, hints });
                // Set state to SelectingDragging
                *selection_state = SelectionState::SelectingDragging(square);
            }
            SelectionStateAction::Unselect(square) => {
                // Re-parent piece back to its tile
                let piece = board_state.piece(square);
                let tile = board_state.tile(square);
                commands.entity(piece).set_parent(tile);
                // Unselect square & remove hints
                selection_events.send(SelectionEvent::Unselect);
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

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, SystemSet)]
pub struct SelectionEventHandler;

#[derive(Clone, Debug, Event)]
#[allow(dead_code)]
pub enum SelectionEvent {
    UpdateSelection { highlight: Entity, hints: Vec<Entity> },
    Unselect,
    UpdateLastMove(Square, Square),
    UnsetLastMove,
    UnsetAll,
}

pub fn handle_selection_events(
    mut commands: Commands,
    board_state: Res<BoardState>,
    mut event_reader: EventReader<SelectionEvent>,
    q_selection: Query<Entity, (With<HighlightTile>, With<Selected>)>,
    q_last_move: Query<Entity, (With<HighlightTile>, With<LastMove>)>,
    q_selected_hints: Query<Entity, (With<MoveHint>, With<EnabledHint>)>,
) {
    let unset_selection = |commands: &mut Commands| {
        for entity in &q_selection {
            commands.entity(entity).remove::<Selected>().insert(HideIndicator);
        }
    };

    let unset_last_move = |commands: &mut Commands| {
        for entity in &q_last_move {
            commands.entity(entity).remove::<LastMove>().insert(HideIndicator);
        }
    };

    let unset_hints = |commands: &mut Commands| {
        for entity in &q_selected_hints {
            commands.entity(entity).remove::<EnabledHint>().insert(HideIndicator);
        }
    };

    for event in event_reader.read() {
        match event {
            SelectionEvent::UpdateSelection { highlight, hints } => {
                unset_selection(&mut commands);
                unset_hints(&mut commands);
                commands.entity(*highlight).insert((Selected, ShowIndicator));
                for &entity in hints {
                    commands.entity(entity).insert((EnabledHint, ShowIndicator));
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
                commands.entity(e1).insert((LastMove, ShowIndicator));
                commands.entity(e2).insert((LastMove, ShowIndicator));
            }
            SelectionEvent::UnsetLastMove => unset_last_move(&mut commands),
            SelectionEvent::UnsetAll => {
                unset_selection(&mut commands);
                unset_hints(&mut commands);
                unset_last_move(&mut commands);
            }
        }
    }
}

#[derive(Component)]
struct ShowIndicator;

#[derive(Component)]
struct HideIndicator;

fn update_indicators(
    mut commands: Commands,
    mut q_show: Query<&mut Visibility, (With<ShowIndicator>, Without<HideIndicator>)>,
    mut q_hide: Query<&mut Visibility, (With<HideIndicator>, Without<ShowIndicator>)>,
) {
    q_show.for_each_mut(|mut vis| *vis = Visibility::Visible);
    q_hide.for_each_mut(|mut vis| *vis = Visibility::Hidden);

    commands.add(|world: &mut World| {
        let mut entities = Vec::with_capacity(8);

        entities.extend(world.query_filtered::<Entity, With<ShowIndicator>>().iter(world));
        for entity in entities.drain(..) {
            world.entity_mut(entity).remove::<ShowIndicator>();
        }

        entities.extend(world.query_filtered::<Entity, With<HideIndicator>>().iter(world));
        for entity in entities.drain(..) {
            world.entity_mut(entity).remove::<HideIndicator>();
        }
    });
}

#[cfg(test)]
mod tests {
    use bevy::ecs::system::Command;

    use super::*;

    mod utils {
        use bevy::ecs::system::SystemState;

        use crate::game::{
            core::{GameHeadlessPlugin, GameTestPlugin},
            menu::test::TestMenuStateInGamePlugin,
            ui::GameUiPlugin,
        };

        use super::*;

        pub fn build_app() -> App {
            let mut app = App::new();
            app.add_plugins((GameHeadlessPlugin, GameTestPlugin))
                .add_plugins(TestMenuStateInGamePlugin)
                .add_plugins(GameUiPlugin)
                .add_plugins(SelectionPlugin);
            app
        }

        pub fn get_state(app: &App) -> SelectionState {
            *app.world.resource::<SelectionState>()
        }

        pub fn set_state(app: &mut App, state: SelectionState) {
            *app.world.resource_mut::<SelectionState>() = state;
        }

        pub fn get_tagged_entity<Tag: Component>(app: &mut App) -> Entity {
            app.world.query_filtered::<Entity, With<Tag>>().single(&app.world)
        }

        pub fn view_tag_descendants<Tag: Component>(
            app: &mut App,
            f: impl FnOnce(DescendantIter<'_, '_, &Children, With<Tag>>),
        ) {
            let entity = get_tagged_entity::<Tag>(app);

            let mut q_state = SystemState::<Query<&Children, With<Tag>>>::new(&mut app.world);
            let q = q_state.get(&app.world);
            f(q.iter_descendants(entity));
        }

        pub fn get_entity_parent(app: &App, entity: Entity) -> Option<Entity> {
            app.world.entity(entity).get::<Parent>().map(Parent::get)
        }

        pub fn entity_is_visible(app: &App, entity: Entity) -> bool {
            app.world
                .entity(entity)
                .get::<Visibility>()
                .map(|vis| *vis == Visibility::Visible)
                .unwrap_or(false)
        }
    }
    use utils::*;

    #[test]
    fn unselected_is_initial_state() {
        let app = build_app();

        assert_eq!(get_state(&app), SelectionState::Unselected);
    }

    #[test]
    fn unselected_starts_dragging_on_mouse_down_on_piece() {
        let mut app = build_app();

        app.world.send_event(MouseSelectionEvent::MouseDown(Square::D2));
        app.update();

        let board_state = app.world.resource::<BoardState>();
        let expected_hl = board_state.highlight(Square::D2);
        let expected_piece = board_state.piece(Square::D2);

        // The selection state is Selecting Dragging
        assert_eq!(get_state(&app), SelectionState::SelectingDragging(Square::D2));

        // The piece being dragged is a child of Drag Container
        view_tag_descendants::<DragContainer>(&mut app, |mut descendants| {
            assert!(descendants.any(|d| { d == expected_piece }));
        });

        // The tile under the piece being dragged is highlighted
        assert!(entity_is_visible(&app, expected_hl));
    }

    #[test]
    fn unselected_does_nothing_on_mouse_down_when_not_on_piece() {
        let mut app = build_app();

        app.world.send_event(MouseSelectionEvent::MouseDown(Square::D4));
        app.update();

        // The selection state is Selecting Dragging
        assert_eq!(get_state(&app), SelectionState::Unselected);
    }

    #[test]
    fn unselected_does_nothing_on_mouse_up() {
        let mut app = build_app();

        app.world.send_event(MouseSelectionEvent::MouseUp(Square::A1));
        app.update();

        assert_eq!(get_state(&app), SelectionState::Unselected);
    }

    #[test]
    fn selecting_dragging_makes_move_on_mouse_up_when_on_valid_move() {
        let mut app = build_app();
        app.update();
        set_state(&mut app, SelectionState::SelectingDragging(Square::D2));
        let board_state = app.world.resource::<BoardState>();
        let dragging_piece = board_state.piece(Square::D2);
        let expected_piece_parent = board_state.tile(Square::D4);
        let expected_hl_1 = board_state.highlight(Square::D2);
        let expected_hl_2 = board_state.highlight(Square::D4);
        let drag_container = get_tagged_entity::<DragContainer>(&mut app);
        // Re-parent the piece to the Drag Container
        AddChild { parent: drag_container, child: dragging_piece }.apply(&mut app.world);

        app.world.send_event(MouseSelectionEvent::MouseUp(Square::D4));
        app.update();

        // The selection state is Unselected
        assert_eq!(get_state(&app), SelectionState::Unselected);

        // The piece is re-parented to the mouse up tile
        view_tag_descendants::<DragContainer>(&mut app, |descendants| {
            assert_eq!(descendants.count(), 0, "drag container has unexpected children");
        });
        assert_eq!(get_entity_parent(&app, dragging_piece), Some(expected_piece_parent));

        // Last move highlights are visible
        assert!(entity_is_visible(&app, expected_hl_1));
        assert!(entity_is_visible(&app, expected_hl_2));
    }

    #[test]
    fn selecting_dragging_selects_on_mouse_up_when_not_a_move() {
        let mut app = build_app();
        app.update();
        set_state(&mut app, SelectionState::SelectingDragging(Square::D2));
        let board_state = app.world.resource::<BoardState>();
        let dragging_piece = board_state.piece(Square::D2);
        let drag_container = get_tagged_entity::<DragContainer>(&mut app);
        // Re-parent the piece to the Drag Container
        AddChild { parent: drag_container, child: dragging_piece }.apply(&mut app.world);

        app.world.send_event(MouseSelectionEvent::MouseUp(Square::A8));
        app.update();

        // The selection state is Selected
        assert_eq!(get_state(&app), SelectionState::Selected(Square::D2));

        // The Drag Container has no children
        view_tag_descendants::<DragContainer>(&mut app, |descendants| {
            assert_eq!(descendants.count(), 0, "drag container has unexpected children");
        });
    }

    #[test]
    fn selected_starts_dragging_on_mouse_down_on_selected_tile() {
        let mut app = build_app();
        app.update();
        set_state(&mut app, SelectionState::Selected(Square::D2));
        let hl_tile = app.world.resource::<BoardState>().highlight(Square::D2);
        app.world.entity_mut(hl_tile).insert(Selected);

        app.world.send_event(MouseSelectionEvent::MouseDown(Square::D2));
        app.update();

        let board_state = app.world.resource::<BoardState>();
        let expected_hl = board_state.highlight(Square::D2);
        let expected_piece = board_state.piece(Square::D2);

        // The selection state is Selecting Dragging
        assert_eq!(get_state(&app), SelectionState::SelectedDragging(Square::D2));

        // The piece being dragged is a child of Drag Container
        view_tag_descendants::<DragContainer>(&mut app, |mut descendants| {
            assert!(descendants.any(|d| { d == expected_piece }));
        });

        // The tile under the piece being dragged is *still* highlighted
        assert!(entity_is_visible(&app, expected_hl));
    }

    #[test]
    fn selected_makes_move_on_mouse_down_when_on_valid_move() {
        let mut app = build_app();
        app.update();
        set_state(&mut app, SelectionState::Selected(Square::D2));
        let board_state = app.world.resource::<BoardState>();
        let dragging_piece = board_state.piece(Square::D2);
        let expected_piece_parent = board_state.tile(Square::D4);
        let expected_hl_1 = board_state.highlight(Square::D2);
        let expected_hl_2 = board_state.highlight(Square::D4);

        app.world.send_event(MouseSelectionEvent::MouseDown(Square::D4));
        app.update();

        // The selection state is Unselected
        assert_eq!(get_state(&app), SelectionState::Unselected);

        // The piece is re-parented to the mouse down square
        assert_eq!(get_entity_parent(&app, dragging_piece), Some(expected_piece_parent));

        // Last move highlights are visible
        assert!(entity_is_visible(&app, expected_hl_1));
        assert!(entity_is_visible(&app, expected_hl_2));
    }

    #[test]
    fn selected_starts_dragging_on_mouse_down_on_different_piece() {
        let mut app = build_app();
        set_state(&mut app, SelectionState::Selected(Square::D2));

        app.world.send_event(MouseSelectionEvent::MouseDown(Square::D7));
        app.update();

        let board_state = app.world.resource::<BoardState>();
        let expected_hl = board_state.highlight(Square::D7);
        let expected_piece = board_state.piece(Square::D7);

        // The selection state is Selecting Dragging
        assert_eq!(get_state(&app), SelectionState::SelectingDragging(Square::D7));

        // The piece being dragged is a child of Drag Container
        view_tag_descendants::<DragContainer>(&mut app, |mut descendants| {
            assert!(descendants.any(|d| { d == expected_piece }));
        });

        // The tile under the piece being dragged is highlighted
        assert!(entity_is_visible(&app, expected_hl));
    }

    #[test]
    fn selected_unselects_on_mouse_down_when_not_on_a_piece() {
        let mut app = build_app();
        set_state(&mut app, SelectionState::Selected(Square::D2));

        app.world.send_event(MouseSelectionEvent::MouseDown(Square::E4));
        app.update();

        assert_eq!(get_state(&app), SelectionState::Unselected);
    }

    #[test]
    fn selected_does_nothing_on_mouse_up() {
        let mut app = build_app();
        set_state(&mut app, SelectionState::Selected(Square::D2));

        app.world.send_event(MouseSelectionEvent::MouseUp(Square::A1));
        app.update();

        assert_eq!(get_state(&app), SelectionState::Selected(Square::D2));
    }

    #[test]
    fn selected_dragging_unselects_on_mouse_up_on_selected_tile() {
        let mut app = build_app();
        app.update();
        set_state(&mut app, SelectionState::SelectedDragging(Square::D2));
        let hl_tile = app.world.resource::<BoardState>().highlight(Square::D2);
        app.world.entity_mut(hl_tile).insert(Selected);
        let board_state = app.world.resource::<BoardState>();
        let dragging_piece = board_state.piece(Square::D2);
        let expected_piece_parent = board_state.tile(Square::D2);
        let unexpected_hl = board_state.highlight(Square::D2);
        let drag_container = get_tagged_entity::<DragContainer>(&mut app);
        // Re-parent the piece to the Drag Container
        AddChild { parent: drag_container, child: dragging_piece }.apply(&mut app.world);

        app.world.send_event(MouseSelectionEvent::MouseUp(Square::D2));
        app.update();

        // The selection state is Selected
        assert_eq!(get_state(&app), SelectionState::Unselected);

        // The piece is re-parented to its original tile
        view_tag_descendants::<DragContainer>(&mut app, |descendants| {
            assert_eq!(descendants.count(), 0, "drag container has unexpected children");
        });
        assert_eq!(get_entity_parent(&app, dragging_piece), Some(expected_piece_parent));

        // The original tile of the piece is un-highlighted
        assert!(!entity_is_visible(&app, unexpected_hl));
    }

    #[test]
    fn selected_dragging_makes_move_on_mouse_up_when_on_valid_move() {
        let mut app = build_app();
        app.update();
        set_state(&mut app, SelectionState::SelectedDragging(Square::D2));
        let hl_tile = app.world.resource::<BoardState>().highlight(Square::D2);
        app.world.entity_mut(hl_tile).insert(Selected);
        let board_state = app.world.resource::<BoardState>();
        let dragging_piece = board_state.piece(Square::D2);
        let expected_piece_parent = board_state.tile(Square::D4);
        let expected_hl_1 = board_state.highlight(Square::D2);
        let expected_hl_2 = board_state.highlight(Square::D4);
        let drag_container = get_tagged_entity::<DragContainer>(&mut app);
        // Re-parent the piece to the Drag Container
        AddChild { parent: drag_container, child: dragging_piece }.apply(&mut app.world);

        app.world.send_event(MouseSelectionEvent::MouseUp(Square::D4));
        app.update();

        // The selection state is Unselected
        assert_eq!(get_state(&app), SelectionState::Unselected);

        // The piece is re-parented to the mouse up tile
        view_tag_descendants::<DragContainer>(&mut app, |descendants| {
            assert_eq!(descendants.count(), 0, "drag container has unexpected children");
        });
        assert_eq!(get_entity_parent(&app, dragging_piece), Some(expected_piece_parent));

        // Last move highlights are visible
        assert!(entity_is_visible(&app, expected_hl_1));
        assert!(entity_is_visible(&app, expected_hl_2));
    }

    #[test]
    fn selected_dragging_reselects_on_mouse_up_when_not_a_move() {
        let mut app = build_app();
        app.update();
        set_state(&mut app, SelectionState::SelectedDragging(Square::D2));
        let hl_tile = app.world.resource::<BoardState>().highlight(Square::D2);
        app.world.entity_mut(hl_tile).insert(Selected);
        let board_state = app.world.resource::<BoardState>();
        let dragging_piece = board_state.piece(Square::D2);
        let expected_piece_parent = board_state.tile(Square::D2);
        let expected_hl = board_state.highlight(Square::D2);
        let drag_container = get_tagged_entity::<DragContainer>(&mut app);
        // Re-parent the piece to the Drag Container
        AddChild { parent: drag_container, child: dragging_piece }.apply(&mut app.world);

        app.world.send_event(MouseSelectionEvent::MouseUp(Square::A8));
        app.update();

        // The selection state is Selected
        assert_eq!(get_state(&app), SelectionState::Selected(Square::D2));

        // The Drag Container has no children
        view_tag_descendants::<DragContainer>(&mut app, |descendants| {
            assert_eq!(descendants.count(), 0, "drag container has unexpected children");
        });
        assert_eq!(get_entity_parent(&app, dragging_piece), Some(expected_piece_parent));

        // The tile under the piece being dragged is *still* highlighted
        assert!(entity_is_visible(&app, expected_hl));
    }
}
