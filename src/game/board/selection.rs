use std::fmt::{self, Write};

use bevy::prelude::*;

use crate::{
    game::{
        LoadGame,
        menu::MenuState,
        mouse::{self, Dragging},
    },
    utils::NoopExts,
};

use super::{
    BoardState, HighlightTile, HighlightTilesMap, Hint, MovePiece, PiecesMap, Square,
    StartPromotion, TileHintsMap,
};

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.noop()
            // Resources
            .init_resource::<SelectionState>()
            // Events
            .add_message::<MouseSelectionEvent>()
            // Observers
            .add_observer(mouse::on_add_dragging_piece)
            .add_observer(mouse::on_remove_dragging_piece)
            .add_observer(Select::handle)
            .add_observer(Unselect::handle)
            .add_observer(EnableHints::handle)
            .add_observer(EnableLastMove::handle)
            .add_observer(DisableSelectionHighlight::handle)
            .add_observer(DisableHints::handle)
            .add_observer(DisableLastMove::handle)
            // Observers - LoadGame
            .add_observer(|_load_game: On<LoadGame>, mut commands: Commands| {
                commands.trigger(Unselect);
                commands.trigger(DisableLastMove);
            })
            // Observers - StartPromotion
            .add_observer(|_start_promotion: On<StartPromotion>, mut commands: Commands| {
                commands.trigger(Unselect);
            })
            // Systems
            .add_systems(Update, handle_mouse_selection_events.run_if(in_state(MenuState::Game)))
            .noop();
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Resource)]
enum SelectionState {
    #[default]
    Unselected,
    SelectingDragging(Square),
    Selected(Square),
    SelectedDragging(Square),
}

#[derive(Clone, Debug)]
enum SelectionStateAction {
    ChangeSelection(Square),
    DropSelect(Square),
    Move { from_sq: Square, to_sq: Square, animate: bool },
    None,
    StartSelectedDragging(Square),
    StartSelectingDragging(Square),
    Unselect(Square),
}

impl fmt::Display for SelectionStateAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::ChangeSelection(sq) => f.write_fmt(format_args!("ChangeSelection(=> {sq})")),
            Self::DropSelect(sq) => f.write_fmt(format_args!("DropSelect({sq})")),
            Self::Move { from_sq, to_sq, animate } => f.write_fmt(format_args!(
                "Move({from_sq} => {to_sq}, {})",
                if animate { "anim" } else { "no_anim" }
            )),
            Self::None => f.write_str("None"),
            Self::StartSelectedDragging(sq) => {
                f.write_fmt(format_args!("StartSelectedDragging({sq})"))
            }
            Self::StartSelectingDragging(sq) => {
                f.write_fmt(format_args!("StartSelectingDragging({sq})"))
            }
            Self::Unselect(sq) => f.write_fmt(format_args!("Unselect({sq})")),
        }
    }
}

#[derive(Clone, Copy, Debug, Message)]
pub enum MouseSelectionEvent {
    MouseDown(Square),
    MouseUp(Square),
}

impl fmt::Display for MouseSelectionEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::MouseDown(sq) => f.write_fmt(format_args!("MouseDown({sq})")),
            Self::MouseUp(sq) => f.write_fmt(format_args!("MouseUp({sq})")),
        }
    }
}

fn handle_mouse_selection_events(
    mut commands: Commands,
    mut selection_state: ResMut<SelectionState>,
    highlights_map: Res<HighlightTilesMap>,
    hints_map: Res<TileHintsMap>,
    pieces_map: Res<PiecesMap>,
    board_state: Res<BoardState>,
    mut message_reader: MessageReader<MouseSelectionEvent>,
) {
    for &event in message_reader.read() {
        trace!(state = ?*selection_state, %event, "Processing mouse selection");
        let action = match *selection_state {
            SelectionState::Unselected => match event {
                MouseSelectionEvent::MouseDown(square) => {
                    if pieces_map.has(square) {
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
                    } else if pieces_map.has(square) {
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

        trace!(%action, "Processing mouse selection");
        match action {
            SelectionStateAction::None => {}
            SelectionStateAction::ChangeSelection(to_sq) => {
                // Start dragging piece
                let piece = pieces_map.at(to_sq);
                commands.entity(piece).insert(Dragging { original_square: to_sq });
                // Update selection & hints
                let hl_tile = highlights_map.at(to_sq);
                let hint_descriptors = board_state.calculate_valid_moves(to_sq);
                let hints = hints_map.many_from_descriptors(&hint_descriptors);
                commands.trigger(Select { highlight: hl_tile, hints });
                // Set state to SelectingDragging
                *selection_state = SelectionState::SelectingDragging(to_sq);
            }
            SelectionStateAction::DropSelect(square) => {
                // Drop piece
                let piece = pieces_map.at(square);
                commands.entity(piece).remove::<Dragging>();
                // Set state to Selected
                *selection_state = SelectionState::Selected(square);
            }
            SelectionStateAction::Move { from_sq, to_sq, animate } => {
                let piece = pieces_map.at(from_sq);
                commands.trigger(MovePiece::new(piece, from_sq, to_sq, None, animate));
                // Set state to Unselected
                *selection_state = SelectionState::Unselected;
            }
            SelectionStateAction::StartSelectedDragging(square) => {
                // Start dragging piece
                let piece = pieces_map.at(square);
                commands.entity(piece).insert(Dragging { original_square: square });
                // Set state to SelectedDragging
                *selection_state = SelectionState::SelectedDragging(square);
            }
            SelectionStateAction::StartSelectingDragging(square) => {
                // Start dragging piece
                let piece = pieces_map.at(square);
                commands.entity(piece).insert(Dragging { original_square: square });
                // Update selection & hints
                let hl_tile = highlights_map.at(square);
                let hint_descriptors = board_state.calculate_valid_moves(square);
                let hints = hints_map.many_from_descriptors(&hint_descriptors);
                commands.trigger(Select { highlight: hl_tile, hints });
                // Set state to SelectingDragging
                *selection_state = SelectionState::SelectingDragging(square);
            }
            SelectionStateAction::Unselect(square) => {
                // Drop piece
                let piece = pieces_map.at(square);
                commands.entity(piece).remove::<Dragging>();
                // Unselect square & remove hints
                commands.trigger(Unselect);
                // Set state to Unselected
                *selection_state = SelectionState::Unselected;
            }
        };
    }
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct Selected;

#[derive(Component)]
#[component(storage = "SparseSet")]
struct EnabledHint;

#[derive(Component)]
#[component(storage = "SparseSet")]
struct LastMove;

#[derive(Component)]
#[component(storage = "SparseSet")]
pub(super) struct ShowingIndicator;

impl ShowingIndicator {
    pub(super) fn on_add(add: On<Add, Self>, mut q_vis: Query<&mut Visibility>) {
        if let Ok(mut vis) = q_vis.get_mut(add.entity) {
            *vis = Visibility::Visible;
        }
    }

    pub(super) fn on_remove(remove: On<Remove, Self>, mut q_vis: Query<&mut Visibility>) {
        if let Ok(mut vis) = q_vis.get_mut(remove.entity) {
            *vis = Visibility::Hidden;
        }
    }
}

pub fn unselect_on_move_piece(move_piece: On<MovePiece>, mut commands: Commands) {
    let MovePiece { from_sq, to_sq, .. } = *move_piece;
    commands.trigger(Unselect);
    commands.trigger(DisableLastMove);
    commands.trigger(EnableLastMove(from_sq, to_sq));
}

#[derive(Event, Debug)]
struct Select {
    highlight: Entity,
    hints: Vec<Entity>,
}

impl fmt::Display for Select {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Select(")?;
        f.write_fmt(format_args!("highlight={}, hints=", self.highlight))?;
        if !self.hints.is_empty() {
            f.write_fmt(format_args!("{}", self.hints[0]))?;
            if self.hints.len() > 1 {
                for e in &self.hints[1..] {
                    f.write_fmt(format_args!(",{e}"))?;
                }
            }
        }
        f.write_char(')')?;
        Ok(())
    }
}

impl Select {
    fn handle(select: On<Self>, mut commands: Commands) {
        commands.trigger(Unselect);
        commands.trigger(EnableSelectionHighlight(select.highlight));
        commands.trigger(EnableHints(select.hints.clone()));
    }
}

#[derive(Event, Debug)]
struct Unselect;

impl fmt::Display for Unselect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

impl Unselect {
    fn handle(_unselect: On<Self>, mut commands: Commands) {
        commands.trigger(DisableSelectionHighlight);
        commands.trigger(DisableHints);
    }
}

#[derive(EntityEvent)]
pub(super) struct EnableSelectionHighlight(Entity);

impl EnableSelectionHighlight {
    pub(super) fn handle(enable_selection_highlight: On<Self>, mut commands: Commands) {
        commands.entity(enable_selection_highlight.0).insert((ShowingIndicator, Selected));
    }
}

#[derive(Event)]
struct EnableHints(Vec<Entity>);

impl EnableHints {
    fn handle(enable_hints: On<Self>, mut commands: Commands) {
        for &entity in &enable_hints.0 {
            commands.entity(entity).insert((ShowingIndicator, EnabledHint));
        }
    }
}

#[derive(Event, Debug)]
struct EnableLastMove(Square, Square);

impl fmt::Display for EnableLastMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self(from_sq, to_sq) = *self;
        f.write_fmt(format_args!("UpdateLastMove({from_sq} => {to_sq})"))
    }
}

impl EnableLastMove {
    fn handle(
        enable_last_move: On<Self>,
        mut commands: Commands,
        highlights_map: Res<HighlightTilesMap>,
    ) {
        let Self(from_sq, to_sq) = *enable_last_move;
        let [e1, e2] = highlights_map.many([from_sq, to_sq]);
        commands.entity(e1).insert((ShowingIndicator, LastMove));
        commands.entity(e2).insert((ShowingIndicator, LastMove));
    }
}

#[derive(Event)]
struct DisableSelectionHighlight;

impl DisableSelectionHighlight {
    fn handle(
        _disable_selection_highlight: On<Self>,
        mut commands: Commands,
        q_selection: Query<(Entity, Has<LastMove>), (With<HighlightTile>, With<Selected>)>,
    ) {
        for (entity, is_last_move) in &q_selection {
            let mut entity_cmds = commands.entity(entity);
            entity_cmds.remove::<Selected>();
            if !is_last_move {
                entity_cmds.remove::<ShowingIndicator>();
            }
        }
    }
}

#[derive(Event)]
struct DisableHints;

impl DisableHints {
    fn handle(
        _disable_hints: On<Self>,
        mut commands: Commands,
        q_selected_hints: Query<Entity, (With<Hint>, With<EnabledHint>)>,
    ) {
        for entity in &q_selected_hints {
            commands.entity(entity).remove::<(ShowingIndicator, EnabledHint)>();
        }
    }
}

#[derive(Event)]
struct DisableLastMove;

impl DisableLastMove {
    fn handle(
        _disable_last_move: On<Self>,
        mut commands: Commands,
        q_last_move: Query<Entity, (With<HighlightTile>, With<LastMove>)>,
    ) {
        for entity in &q_last_move {
            commands.entity(entity).remove::<(ShowingIndicator, LastMove)>();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::game::board::{PieceColor, PieceType};

    use super::*;

    mod utils {
        use bevy::platform::collections::HashSet;

        use crate::game::{
            board::{PieceMeta, TilesMap},
            menu::test::TestMenuStateInGamePlugin,
            ui::GameUiPlugin,
        };

        use super::*;

        pub fn build_app() -> App {
            let mut app = App::new();
            app.add_plugins(crate::utils::GameTestPlugin)
                .add_plugins(TestMenuStateInGamePlugin)
                .add_plugins(GameUiPlugin)
                .add_plugins(SelectionPlugin);
            app.update();
            app.world_mut().trigger(LoadGame::in_game_default());
            app.update();
            app
        }

        #[allow(dead_code)]
        pub trait TestAppExts {
            fn tiles_map(&self) -> &TilesMap;
            fn highlights_map(&self) -> &HighlightTilesMap;
            fn hints_map(&self) -> &TileHintsMap;
            fn pieces_map(&self) -> &PiecesMap;

            fn set_state(&mut self, state: SelectionState);
            fn set_selected(&mut self, square: Square);
            fn set_hints(&mut self, squares: impl IntoIterator<Item = Square>);
            fn set_dragging_piece(&mut self, square: Square);

            fn entity_debug_name(&mut self, entity: Entity) -> String;

            fn assert_state(&self, expected: SelectionState);
            fn assert_piece_has_dragging(&mut self, piece: Square);
            fn assert_selected(&mut self, expected: Option<Square>);
            fn assert_move_hints(&mut self, expected: impl IntoIterator<Item = Square>);
            fn assert_piece_on_tile(&mut self, sq: Square, color: PieceColor, typ: PieceType);
        }

        impl TestAppExts for App {
            fn tiles_map(&self) -> &TilesMap {
                self.world().resource::<TilesMap>()
            }

            fn highlights_map(&self) -> &HighlightTilesMap {
                self.world().resource::<HighlightTilesMap>()
            }

            fn hints_map(&self) -> &TileHintsMap {
                self.world().resource::<TileHintsMap>()
            }

            fn pieces_map(&self) -> &PiecesMap {
                self.world().resource::<PiecesMap>()
            }

            fn set_state(&mut self, state: SelectionState) {
                *self.world_mut().resource_mut::<SelectionState>() = state;
            }

            fn set_selected(&mut self, square: Square) {
                let entity = self.highlights_map().at(square);
                self.world_mut().entity_mut(entity).insert(Selected);
            }

            fn set_hints(&mut self, squares: impl IntoIterator<Item = Square>) {
                for sq in squares.into_iter() {
                    // Test assertions don't differentiate between move hints and capture hints
                    let entity = self.hints_map().at(sq).move_entity;
                    self.world_mut().entity_mut(entity).insert(EnabledHint);
                }
            }

            fn set_dragging_piece(&mut self, square: Square) {
                let piece = self.pieces_map().at(square);
                self.world_mut().entity_mut(piece).insert(Dragging { original_square: square });
            }

            fn entity_debug_name(&mut self, entity: Entity) -> String {
                let world = self.world_mut();
                let mut q = world.query::<NameOrEntity>();
                let name_or_entity = q.get(world, entity).unwrap();
                match name_or_entity.name {
                    Some(name) => format!("{entity:?} ({name})"),
                    None => format!("{entity:?}"),
                }
            }

            #[track_caller]
            fn assert_state(&self, expected: SelectionState) {
                let actual = *self.world().resource::<SelectionState>();
                assert_eq!(actual, expected);
            }

            #[track_caller]
            fn assert_piece_has_dragging(&mut self, piece: Square) {
                let expected = Some(Dragging { original_square: piece });
                let entity = self.pieces_map().at(piece);

                let actual = self.world().entity(entity).get::<Dragging>().copied();

                assert_eq!(actual, expected);
            }

            #[track_caller]
            fn assert_selected(&mut self, expected: Option<Square>) {
                if let Some(expected) = expected {
                    let entity = self.highlights_map().at(expected);
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

            #[track_caller]
            fn assert_move_hints(&mut self, expected: impl IntoIterator<Item = Square>) {
                let hints_map = self.hints_map();

                let expected = expected
                    .into_iter()
                    .map(|sq| hints_map.at(sq).move_entity)
                    .collect::<HashSet<_>>();

                let mut q =
                    self.world_mut().query_filtered::<Entity, (With<Hint>, With<EnabledHint>)>();
                let actual = q.iter(self.world()).collect::<HashSet<_>>();

                assert_eq!(actual, expected, "enabled hint entities");
            }

            #[track_caller]
            fn assert_piece_on_tile(&mut self, sq: Square, color: PieceColor, typ: PieceType) {
                let piece = self.pieces_map().at(sq);
                let tile = self.tiles_map().at(sq);
                let piece = self.world().entity(piece);
                let piece_typ = piece.get::<PieceMeta>();
                assert_eq!(
                    piece_typ,
                    Some(&PieceMeta::new(color, typ)),
                    "piece at {sq} in pieces map is not a {color} {typ}"
                );
                let piece_parent =
                    piece.get::<ChildOf>().cloned().map(|c| self.entity_debug_name(c.parent()));
                assert_eq!(
                    piece_parent,
                    Some(self.entity_debug_name(tile)),
                    "piece at {sq} in pieces map is not a child of tile {sq}"
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

        app.world_mut().write_message(MouseSelectionEvent::MouseDown(Square::D2));
        app.update();

        app.assert_state(SelectionState::SelectingDragging(Square::D2));
        app.assert_piece_has_dragging(Square::D2);
        app.assert_selected(Some(Square::D2));
        app.assert_move_hints([Square::D3, Square::D4]);
    }

    #[test]
    fn unselected_does_nothing_on_mouse_down_when_not_on_piece() {
        let mut app = build_app();

        app.world_mut().write_message(MouseSelectionEvent::MouseDown(Square::D4));
        app.update();

        app.assert_state(SelectionState::Unselected);
        app.assert_selected(None);
        app.assert_move_hints([]);
    }

    #[test]
    fn unselected_does_nothing_on_mouse_up() {
        let mut app = build_app();

        app.world_mut().write_message(MouseSelectionEvent::MouseUp(Square::A1));
        app.update();

        app.assert_state(SelectionState::Unselected);
        app.assert_selected(None);
        app.assert_move_hints([]);
    }

    #[test]
    fn selecting_dragging_makes_move_on_mouse_up_when_on_valid_move() {
        let mut app = build_app();
        app.set_state(SelectionState::SelectingDragging(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);
        app.set_dragging_piece(Square::D2);

        app.world_mut().write_message(MouseSelectionEvent::MouseUp(Square::D4));
        app.update();

        app.assert_state(SelectionState::Unselected);
        app.assert_piece_on_tile(Square::D4, PieceColor::WHITE, PieceType::PAWN);
    }

    #[test]
    fn selecting_dragging_selects_on_mouse_up_when_not_a_move() {
        let mut app = build_app();
        app.set_state(SelectionState::SelectingDragging(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);
        app.set_dragging_piece(Square::D2);

        app.world_mut().write_message(MouseSelectionEvent::MouseUp(Square::A8));
        app.update();

        app.assert_state(SelectionState::Selected(Square::D2));
        app.assert_piece_on_tile(Square::D2, PieceColor::WHITE, PieceType::PAWN);
        app.assert_selected(Some(Square::D2));
        app.assert_move_hints([Square::D3, Square::D4]);
    }

    #[test]
    fn selected_starts_dragging_on_mouse_down_on_selected_tile() {
        let mut app = build_app();
        app.set_state(SelectionState::Selected(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);

        app.world_mut().write_message(MouseSelectionEvent::MouseDown(Square::D2));
        app.update();

        app.assert_state(SelectionState::SelectedDragging(Square::D2));
        app.assert_piece_has_dragging(Square::D2);
        app.assert_selected(Some(Square::D2));
        app.assert_move_hints([Square::D3, Square::D4]);
    }

    #[test]
    fn selected_makes_move_on_mouse_down_when_on_valid_move() {
        let mut app = build_app();
        app.set_state(SelectionState::Selected(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);

        app.world_mut().write_message(MouseSelectionEvent::MouseDown(Square::D4));
        app.update();

        app.assert_state(SelectionState::Unselected);
        app.assert_piece_on_tile(Square::D4, PieceColor::WHITE, PieceType::PAWN);
    }

    #[test]
    fn selected_starts_dragging_on_mouse_down_on_different_piece() {
        let mut app = build_app();
        app.set_state(SelectionState::Selected(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);

        app.world_mut().write_message(MouseSelectionEvent::MouseDown(Square::H2));
        app.update();

        app.assert_state(SelectionState::SelectingDragging(Square::H2));
        app.assert_piece_on_tile(Square::D2, PieceColor::WHITE, PieceType::PAWN);
        app.assert_piece_has_dragging(Square::H2);
        app.assert_selected(Some(Square::H2));
        app.assert_move_hints([Square::H3, Square::H4]);
    }

    #[test]
    fn selected_unselects_on_mouse_down_when_not_on_a_piece() {
        let mut app = build_app();
        app.set_state(SelectionState::Selected(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);

        app.world_mut().write_message(MouseSelectionEvent::MouseDown(Square::E4));
        app.update();

        app.assert_state(SelectionState::Unselected);
        app.assert_selected(None);
        app.assert_move_hints([]);
    }

    #[test]
    fn selected_does_nothing_on_mouse_up() {
        let mut app = build_app();
        app.set_state(SelectionState::Selected(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);

        app.world_mut().write_message(MouseSelectionEvent::MouseUp(Square::A1));
        app.update();

        app.assert_state(SelectionState::Selected(Square::D2));
        app.assert_selected(Some(Square::D2));
        app.assert_move_hints([Square::D3, Square::D4]);
    }

    #[test]
    fn selected_dragging_unselects_on_mouse_up_on_selected_tile() {
        let mut app = build_app();
        app.set_state(SelectionState::SelectedDragging(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);
        app.set_dragging_piece(Square::D2);

        app.world_mut().write_message(MouseSelectionEvent::MouseUp(Square::D2));
        app.update();

        app.assert_state(SelectionState::Unselected);
        app.assert_piece_on_tile(Square::D2, PieceColor::WHITE, PieceType::PAWN);
        app.assert_selected(None);
        app.assert_move_hints([]);
    }

    #[test]
    fn selected_dragging_makes_move_on_mouse_up_when_on_valid_move() {
        let mut app = build_app();
        app.set_state(SelectionState::SelectedDragging(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);
        app.set_dragging_piece(Square::D2);

        app.world_mut().write_message(MouseSelectionEvent::MouseUp(Square::D4));
        app.update();

        app.assert_state(SelectionState::Unselected);
        app.assert_piece_on_tile(Square::D4, PieceColor::WHITE, PieceType::PAWN);
    }

    #[test]
    fn selected_dragging_reselects_on_mouse_up_when_not_a_move() {
        let mut app = build_app();
        app.set_state(SelectionState::SelectedDragging(Square::D2));
        app.set_selected(Square::D2);
        app.set_hints([Square::D3, Square::D4]);
        app.set_dragging_piece(Square::D2);

        app.world_mut().write_message(MouseSelectionEvent::MouseUp(Square::A8));
        app.update();

        app.assert_state(SelectionState::Selected(Square::D2));
        app.assert_piece_on_tile(Square::D2, PieceColor::WHITE, PieceType::PAWN);
        app.assert_selected(Some(Square::D2));
        app.assert_move_hints([Square::D3, Square::D4]);
    }

    #[test]
    fn dragging_removed_on_piece_move() {
        // Bug reproduction:
        // - Start with a default board
        // - `a3`
        // - `a6`
        // - Start dragging `a3` to move it to `a4` but it isn't reparented to
        //   the drag container

        let mut app = build_app();

        // `a3`
        app.world_mut().write_message(MouseSelectionEvent::MouseDown(Square::A2));
        app.update();
        app.assert_piece_has_dragging(Square::A2);
        app.world_mut().write_message(MouseSelectionEvent::MouseUp(Square::A3));
        app.update();
        app.assert_piece_on_tile(Square::A3, PieceColor::WHITE, PieceType::PAWN);

        // `a6`
        app.world_mut().write_message(MouseSelectionEvent::MouseDown(Square::A7));
        app.update();
        app.assert_piece_has_dragging(Square::A7);
        app.world_mut().write_message(MouseSelectionEvent::MouseUp(Square::A6));
        app.update();
        app.assert_piece_on_tile(Square::A6, PieceColor::BLACK, PieceType::PAWN);

        // Start dragging `a3`
        app.world_mut().write_message(MouseSelectionEvent::MouseDown(Square::A3));
        app.update();
        app.assert_piece_has_dragging(Square::A3);
    }
}
