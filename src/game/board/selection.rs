use bevy::prelude::*;

use crate::game::{menu::MenuState, mouse::DragContainer, moves::StartMove};

use super::{BoardState, Square};

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<SelectionState>()
            // Events
            .add_event::<SelectionEvent>()
            // Systems
            // TODO: handle_selection_events should run at the end of the set
            .add_systems(Update, handle_selection_events.run_if(in_state(MenuState::Game)));
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
pub enum SelectionEvent {
    MouseDown(Square),
    MouseUp(Square),
}

fn handle_selection_events(
    mut commands: Commands,
    mut selection_state: ResMut<SelectionState>,
    mut board_state: ResMut<BoardState>,
    mut event_reader: EventReader<SelectionEvent>,
    q_drag_container: Query<Entity, With<DragContainer>>,
) {
    for &event in event_reader.read() {
        let action = match *selection_state {
            SelectionState::Unselected => match event {
                SelectionEvent::MouseDown(square) => {
                    if board_state.has_piece_at(square) {
                        SelectionStateAction::StartSelectingDragging(square)
                    } else {
                        SelectionStateAction::None
                    }
                }
                SelectionEvent::MouseUp(_) => SelectionStateAction::None,
            },
            SelectionState::SelectingDragging(selecting_sq) => match event {
                SelectionEvent::MouseDown(_) => todo!("reset previous drag target"), // TODO
                SelectionEvent::MouseUp(square) => {
                    if board_state.move_is_valid(selecting_sq, square) {
                        SelectionStateAction::Move(selecting_sq, square)
                    } else {
                        SelectionStateAction::DropSelect(selecting_sq)
                    }
                }
            },
            SelectionState::Selected(selected_sq) => match event {
                SelectionEvent::MouseDown(square) => {
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
                SelectionEvent::MouseUp(_) => SelectionStateAction::None,
            },
            SelectionState::SelectedDragging(selected_sq) => match event {
                SelectionEvent::MouseDown(_) => todo!("reset previous drag target"), // TODO
                SelectionEvent::MouseUp(square) => {
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
                // Unselect square
                commands.add(board_state.unselect_square());
                // Re-parent piece to drag container
                let piece = board_state.piece(to_sq);
                commands.entity(piece).set_parent(q_drag_container.single());
                // Select square
                commands.add(board_state.select_square(to_sq));
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
                // Select square
                commands.add(board_state.select_square(square));
                // Set state to SelectingDragging
                *selection_state = SelectionState::SelectingDragging(square);
            }
            SelectionStateAction::Unselect(square) => {
                // Re-parent piece back to its tile
                let piece = board_state.piece(square);
                let tile = board_state.tile(square);
                commands.entity(piece).set_parent(tile);
                // Unselect square
                commands.add(board_state.unselect_square());
                // Set state to Unselected
                *selection_state = SelectionState::Unselected;
            }
        };
    }
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

        app.world.send_event(SelectionEvent::MouseDown(Square::D2));
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

        app.world.send_event(SelectionEvent::MouseDown(Square::D4));
        app.update();

        // The selection state is Selecting Dragging
        assert_eq!(get_state(&app), SelectionState::Unselected);
    }

    #[test]
    fn unselected_does_nothing_on_mouse_up() {
        let mut app = build_app();

        app.world.send_event(SelectionEvent::MouseUp(Square::A1));
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

        app.world.send_event(SelectionEvent::MouseUp(Square::D4));
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

        app.world.send_event(SelectionEvent::MouseUp(Square::A8));
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
        app.world.resource_mut::<BoardState>().select_square(Square::D2).apply(&mut app.world);

        app.world.send_event(SelectionEvent::MouseDown(Square::D2));
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

        app.world.send_event(SelectionEvent::MouseDown(Square::D4));
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

        app.world.send_event(SelectionEvent::MouseDown(Square::D7));
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

        app.world.send_event(SelectionEvent::MouseDown(Square::E4));
        app.update();

        assert_eq!(get_state(&app), SelectionState::Unselected);
    }

    #[test]
    fn selected_does_nothing_on_mouse_up() {
        let mut app = build_app();
        set_state(&mut app, SelectionState::Selected(Square::D2));

        app.world.send_event(SelectionEvent::MouseUp(Square::A1));
        app.update();

        assert_eq!(get_state(&app), SelectionState::Selected(Square::D2));
    }

    #[test]
    fn selected_dragging_unselects_on_mouse_up_on_selected_tile() {
        let mut app = build_app();
        app.update();
        set_state(&mut app, SelectionState::SelectedDragging(Square::D2));
        app.world.resource_mut::<BoardState>().select_square(Square::D2).apply(&mut app.world);
        let board_state = app.world.resource::<BoardState>();
        let dragging_piece = board_state.piece(Square::D2);
        let expected_piece_parent = board_state.tile(Square::D2);
        let unexpected_hl = board_state.highlight(Square::D2);
        let drag_container = get_tagged_entity::<DragContainer>(&mut app);
        // Re-parent the piece to the Drag Container
        AddChild { parent: drag_container, child: dragging_piece }.apply(&mut app.world);

        app.world.send_event(SelectionEvent::MouseUp(Square::D2));
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
        app.world.resource_mut::<BoardState>().select_square(Square::D2).apply(&mut app.world);
        let board_state = app.world.resource::<BoardState>();
        let dragging_piece = board_state.piece(Square::D2);
        let expected_piece_parent = board_state.tile(Square::D4);
        let expected_hl_1 = board_state.highlight(Square::D2);
        let expected_hl_2 = board_state.highlight(Square::D4);
        let drag_container = get_tagged_entity::<DragContainer>(&mut app);
        // Re-parent the piece to the Drag Container
        AddChild { parent: drag_container, child: dragging_piece }.apply(&mut app.world);

        app.world.send_event(SelectionEvent::MouseUp(Square::D4));
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
        app.world.resource_mut::<BoardState>().select_square(Square::D2).apply(&mut app.world);
        let board_state = app.world.resource::<BoardState>();
        let dragging_piece = board_state.piece(Square::D2);
        let expected_piece_parent = board_state.tile(Square::D2);
        let expected_hl = board_state.highlight(Square::D2);
        let drag_container = get_tagged_entity::<DragContainer>(&mut app);
        // Re-parent the piece to the Drag Container
        AddChild { parent: drag_container, child: dragging_piece }.apply(&mut app.world);

        app.world.send_event(SelectionEvent::MouseUp(Square::A8));
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
