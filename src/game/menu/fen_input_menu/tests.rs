use std::{fmt, sync::LazyLock};

use bevy::{
    ecs::system::SystemState,
    input_focus::{FocusCause, InputFocus},
    ui_widgets::{Activate, ValueChange},
};

use crate::utils::random_values;

use super::{super::MenuState, *};

static NONE: LazyLock<String> = LazyLock::new(|| Color::NONE.to_srgba().to_hex());
static RED: LazyLock<String> = LazyLock::new(|| palettes::css::RED.to_hex());

#[derive(Clone)]
struct FenBuilder {
    bb: chess::BoardBuilder,
    halfmove_clock: u8,
    fullmove_count: u16,
}

impl Default for FenBuilder {
    fn default() -> Self {
        Self { bb: default(), halfmove_clock: 0, fullmove_count: 1 }
    }
}

impl FenBuilder {
    fn random() -> Self {
        let side_to_move = random_values::color();
        let en_passant = rand::random::<bool>().then_some(random_values::file());
        let castle_rights_white = random_values::castle_rights();
        let castle_rights_black = random_values::castle_rights();
        let halfmove_clock = rand::random();
        let fullmove_count = rand::random();

        let mut bb = chess::BoardBuilder::default();
        bb.side_to_move(side_to_move);
        bb.en_passant(en_passant);
        bb.castle_rights(chess::Color::White, castle_rights_white);
        bb.castle_rights(chess::Color::Black, castle_rights_black);

        Self { bb, halfmove_clock, fullmove_count }
    }

    fn side_to_move(mut self, color: chess::Color) -> Self {
        self.bb.side_to_move(color);
        self
    }

    fn en_passant(mut self, file: Option<chess::File>) -> Self {
        self.bb.en_passant(file);
        self
    }

    fn castle_rights(mut self, white: chess::CastleRights, black: chess::CastleRights) -> Self {
        self.bb.castle_rights(chess::Color::White, white);
        self.bb.castle_rights(chess::Color::Black, black);
        self
    }

    fn halfmove_clock(mut self, moves: u8) -> Self {
        self.halfmove_clock = moves;
        self
    }

    fn fullmove_count(mut self, moves: u16) -> Self {
        self.fullmove_count = moves;
        self
    }
}

impl fmt::Display for FenBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bb_fen = self.bb.to_string();
        let bb_fen = &bb_fen[..bb_fen.len() - " 0 1".len()];
        write!(f, "{bb_fen} {} {}", self.halfmove_clock, self.fullmove_count)
    }
}

impl fmt::Debug for FenBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl PartialEq<String> for FenBuilder {
    fn eq(&self, other: &String) -> bool {
        self.to_string().eq(other)
    }
}

impl PartialEq<&str> for FenBuilder {
    fn eq(&self, other: &&str) -> bool {
        self.to_string().eq(other)
    }
}

/// A FEN that moves the state and every control away from their defaults.
fn non_default_fen() -> String {
    FenBuilder::default()
        .side_to_move(chess::Color::Black)
        .en_passant(Some(chess::File::E))
        .castle_rights(chess::CastleRights::NoRights, chess::CastleRights::NoRights)
        .halfmove_clock(5)
        .fullmove_count(10)
        .to_string()
}

struct TestBed {
    app: App,
}

// Setup
impl TestBed {
    fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(bevy::app::TaskPoolPlugin::default());
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.add_plugins(bevy::scene::ScenePlugin);
        app.add_plugins(bevy::text::TextPlugin);
        app.add_plugins(bevy::image::ImagePlugin::default());
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.add_plugins(bevy::input_focus::InputFocusPlugin);
        app.add_plugins(FenInputMenuPlugin);
        app.init_state::<MenuState>();
        app.add_systems(Startup, ui.spawn());

        Self { app }
    }
}

// App Control
impl TestBed {
    fn update(&mut self, n: usize) {
        for _ in 0..n {
            self.app.update();
        }
    }

    fn element_id<M: Component>(&mut self) -> Entity {
        let world = self.app.world_mut();
        let mut q = world.query_filtered::<Entity, With<M>>();
        q.single(world).unwrap()
    }

    fn element_mut<M: Component>(&mut self) -> EntityWorldMut<'_> {
        let id = self.element_id::<M>();
        self.app.world_mut().entity_mut(id)
    }

    fn element_set_checkbox<M: Component>(&mut self, checked: bool) {
        let mut checkbox = self.element_mut::<M>();
        checkbox.trigger(move |source| ValueChange { source, value: checked, is_final: true });
    }

    fn element_set_number_input<M: Component>(&mut self, value: i32, is_final: bool) {
        let mut number_input = self.element_mut::<M>();
        number_input.trigger(move |source| ValueChange { source, value, is_final });
    }

    /// The text input inside a number input.
    fn element_number_input_editable<M: Component>(&mut self) -> Entity {
        let world = self.app.world_mut();
        let mut state = SystemState::<(
            Single<&Children, With<M>>,
            Query<Entity, With<text::EditableText>>,
        )>::new(world);
        let (input_children, q_editable) = state.get(world).unwrap();
        q_editable.iter_many(*input_children).next().unwrap()
    }

    /// Focus a number input, replace its text, and blur it so that the widget reports the text
    /// as its final value.
    fn element_type_number_input<M: Component>(&mut self, text: &str) {
        let editable = self.element_number_input_editable::<M>();
        let world = self.app.world_mut();
        world.resource_mut::<InputFocus>().set(editable, FocusCause::Pressed);
        self.update(1);
        set_editable_text_value(self.app.world_mut(), editable, text);
        self.update(1);
        self.clear_focus();
        self.update(1);
    }

    fn element_set_focus<M: Component>(&mut self) {
        let element = self.element_id::<M>();
        let world = self.app.world_mut();
        world.resource_mut::<InputFocus>().set(element, FocusCause::Pressed);
    }

    fn clear_focus(&mut self) {
        let world = self.app.world_mut();
        world.resource_mut::<InputFocus>().clear();
    }

    fn enter_fen_input_menu(&mut self) {
        let world = self.app.world_mut();
        world.resource_mut::<NextState<MenuState>>().set(MenuState::FenInput);
    }

    fn leave_fen_input_menu(&mut self) {
        let world = self.app.world_mut();
        world.resource_mut::<NextState<MenuState>>().set(MenuState::Main);
    }
}

// State
impl TestBed {
    fn get_state(&self) -> &FenInputMenuState {
        self.app.world().resource::<FenInputMenuState>()
    }
}

// Controls
impl TestBed {
    #[track_caller]
    fn assert_default_controls(&mut self) {
        assert_eq!("", self.get_fen_input_value(), "FEN input");
        assert!(!self.get_fen_input_invalid(), "FEN input invalid");

        self.assert_default_position_controls();

        assert!(!self.get_load_button_disabled(), "load button disabled");
    }

    /// Every control other than the FEN text input and the action buttons.
    #[track_caller]
    fn assert_default_position_controls(&mut self) {
        assert!(self.get_side_to_move_button_white(), "side to move white");
        assert!(!self.get_side_to_move_button_black(), "side to move black");

        assert!(!self.get_en_passant_checkbox(), "en passant checkbox");
        assert_eq!(Visibility::Hidden, self.get_en_passant_file_menu_vis(), "en passant menu");
        assert_eq!(Visibility::Hidden, self.get_en_passant_file_label_vis(), "en passant label");

        assert!(self.get_castle_rights_w_k(), "white kingside");
        assert!(self.get_castle_rights_w_q(), "white queenside");
        assert!(self.get_castle_rights_b_k(), "black kingside");
        assert!(self.get_castle_rights_b_q(), "black queenside");

        assert_eq!("0", self.get_halfmove_clock(), "halfmove clock");
        assert_eq!("1", self.get_fullmove_count(), "fullmove count");
    }

    fn get_fen_input_invalid(&mut self) -> bool {
        self.element_mut::<FenTextInputContainer>().contains::<FenTextInputInvalid>()
    }

    fn set_fen_input_invalid(&mut self, invalid: bool) {
        let mut container = self.element_mut::<FenTextInputContainer>();
        if invalid {
            container.insert(FenTextInputInvalid);
        } else {
            container.remove::<FenTextInputInvalid>();
        }
    }

    fn get_fen_input_border_color(&mut self) -> BorderColor {
        *self.element_mut::<FenTextInputContainer>().get::<BorderColor>().unwrap()
    }

    #[track_caller]
    fn assert_fen_input_border(&mut self, expected: &str) {
        let border_color = self.get_fen_input_border_color();
        let sides = [
            ("top", border_color.top),
            ("right", border_color.right),
            ("bottom", border_color.bottom),
            ("left", border_color.left),
        ];
        for (side, color) in sides {
            assert_eq!(expected, color.to_srgba().to_hex(), "{side} border");
        }
    }

    fn get_fen_input_value(&mut self) -> String {
        let input = self.element_mut::<FenTextInput>();
        input.get::<text::EditableText>().unwrap().value().to_string()
    }

    fn set_fen_input_value(&mut self, fen: &str) {
        let editable = self.element_id::<FenTextInput>();
        let world = self.app.world_mut();
        set_editable_text_value(world, editable, fen);
    }

    /// Focus the FEN text input, replace its value, and run the frames the menu needs to react.
    fn type_fen_input_value(&mut self, fen: &str) {
        self.element_set_focus::<FenTextInput>();
        self.update(1);
        self.set_fen_input_value(fen);
        self.update(1);
    }

    fn blur_fen_input(&mut self) {
        self.clear_focus();
        self.update(1);
    }

    fn get_fen_update_count(&self) -> usize {
        self.app.world().resource::<FenTextEditChangeHandler>().update_count
    }

    fn get_side_to_move_button_white(&mut self) -> bool {
        self.element_mut::<SideToMoveRadioButtonWhite>().contains::<ui::Checked>()
    }

    fn get_side_to_move_button_black(&mut self) -> bool {
        self.element_mut::<SideToMoveRadioButtonBlack>().contains::<ui::Checked>()
    }

    fn select_side_to_move(&mut self, color: chess::Color) {
        let button_id = match color {
            chess::Color::White => self.element_id::<SideToMoveRadioButtonWhite>(),
            chess::Color::Black => self.element_id::<SideToMoveRadioButtonBlack>(),
        };
        let mut group = self.element_mut::<SideToMoveRadioGroup>();
        group.trigger(move |source| ValueChange { source, value: button_id, is_final: true });
    }

    fn get_en_passant_checkbox(&mut self) -> bool {
        self.element_mut::<EnPassantCheckbox>().contains::<ui::Checked>()
    }

    fn set_en_passant_checkbox(&mut self, value: bool) {
        let mut checkbox = self.element_mut::<EnPassantCheckbox>();
        checkbox.trigger(move |source| ValueChange { source, value, is_final: true });
    }

    fn get_en_passant_file_menu_vis(&mut self) -> Visibility {
        *self.element_mut::<EnPassantMenu>().get::<Visibility>().unwrap()
    }

    fn set_en_passant_file(&mut self, file: chess::File) {
        let world = self.app.world_mut();
        let mut state = SystemState::<(
            Single<&Children, With<EnPassantMenuPopup>>,
            Query<(Entity, &EnPassantMenuOption)>,
        )>::new(world);
        let (popup_children, q_options) = state.get(world).unwrap();
        let target = q_options.iter_many(*popup_children).find(|o| o.1.0 == file).unwrap();
        world.entity_mut(target.0).trigger(|entity| Activate { entity });
    }

    fn get_en_passant_file_label_vis(&mut self) -> Visibility {
        *self.element_mut::<EnPassantFileLabel>().get::<Visibility>().unwrap()
    }

    fn get_en_passant_file_label(&mut self) -> String {
        self.element_mut::<EnPassantFileLabel>().get::<Text>().unwrap().0.clone()
    }

    fn get_castle_rights_w_k(&mut self) -> bool {
        self.element_mut::<CastleRightsWhiteKingsideCheckbox>().contains::<ui::Checked>()
    }

    fn set_castle_rights_w_k(&mut self, checked: bool) {
        self.element_set_checkbox::<CastleRightsWhiteKingsideCheckbox>(checked);
    }

    fn get_castle_rights_w_q(&mut self) -> bool {
        self.element_mut::<CastleRightsWhiteQueensideCheckbox>().contains::<ui::Checked>()
    }

    fn set_castle_rights_w_q(&mut self, checked: bool) {
        self.element_set_checkbox::<CastleRightsWhiteQueensideCheckbox>(checked);
    }

    fn get_castle_rights_b_k(&mut self) -> bool {
        self.element_mut::<CastleRightsBlackKingsideCheckbox>().contains::<ui::Checked>()
    }

    fn set_castle_rights_b_k(&mut self, checked: bool) {
        self.element_set_checkbox::<CastleRightsBlackKingsideCheckbox>(checked);
    }

    fn get_castle_rights_b_q(&mut self) -> bool {
        self.element_mut::<CastleRightsBlackQueensideCheckbox>().contains::<ui::Checked>()
    }

    fn set_castle_rights_b_q(&mut self, checked: bool) {
        self.element_set_checkbox::<CastleRightsBlackQueensideCheckbox>(checked);
    }

    fn get_halfmove_clock(&mut self) -> String {
        let world = self.app.world_mut();
        let mut state = SystemState::<(
            Single<&Children, With<HalfmoveClockNumberInput>>,
            Query<&text::EditableText>,
        )>::new(world);
        let (input_children, q_editable) = state.get(world).unwrap();
        q_editable.iter_many(*input_children).next().unwrap().value().to_string()
    }

    fn set_halfmove_clock(&mut self, halfmoves: impl Into<i32>, is_final: bool) {
        self.element_set_number_input::<HalfmoveClockNumberInput>(halfmoves.into(), is_final);
    }

    fn type_halfmove_clock(&mut self, text: &str) {
        self.element_type_number_input::<HalfmoveClockNumberInput>(text);
    }

    fn get_fullmove_count(&mut self) -> String {
        let world = self.app.world_mut();
        let mut state = SystemState::<(
            Single<&Children, With<FullmoveCountNumberInput>>,
            Query<&text::EditableText>,
        )>::new(world);
        let (input_children, q_editable) = state.get(world).unwrap();
        q_editable.iter_many(*input_children).next().unwrap().value().to_string()
    }

    fn set_fullmove_count(&mut self, fullmoves: impl Into<i32>, is_final: bool) {
        self.element_set_number_input::<FullmoveCountNumberInput>(fullmoves.into(), is_final);
    }

    fn type_fullmove_count(&mut self, text: &str) {
        self.element_type_number_input::<FullmoveCountNumberInput>(text);
    }

    fn get_load_button_disabled(&mut self) -> bool {
        self.element_mut::<LoadButton>().contains::<ui::InteractionDisabled>()
    }
}

#[test]
fn default_state() {
    let test_bed = TestBed::new();

    assert_eq!(&FenInputMenuState::default(), test_bed.get_state());
}

#[test]
fn default_controls() {
    let mut test_bed = TestBed::new();
    test_bed.update(1);

    test_bed.enter_fen_input_menu();
    test_bed.update(1);

    test_bed.assert_default_controls();
}

mod on_enter {
    use super::*;

    #[test]
    fn resets_state() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let expected_fen = non_default_fen();

        test_bed.type_fen_input_value(&expected_fen);
        assert_eq!(expected_fen, test_bed.get_state().fen_with_counts());

        test_bed.enter_fen_input_menu();
        test_bed.update(1);
        assert_eq!(&FenInputMenuState::default(), test_bed.get_state());
    }

    #[test]
    fn resets_fen_input() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let expected_fen = non_default_fen();

        test_bed.type_fen_input_value(&expected_fen);
        assert_eq!(expected_fen, test_bed.get_fen_input_value());

        test_bed.enter_fen_input_menu();
        test_bed.update(1);
        assert_eq!("", test_bed.get_fen_input_value());
    }

    #[test]
    fn resets_controls() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        test_bed.type_fen_input_value(&non_default_fen());
        test_bed.blur_fen_input();
        assert!(test_bed.get_side_to_move_button_black());
        assert!(test_bed.get_en_passant_checkbox());
        assert!(!test_bed.get_castle_rights_w_k());
        assert_eq!("5", test_bed.get_halfmove_clock());
        assert_eq!("10", test_bed.get_fullmove_count());

        test_bed.enter_fen_input_menu();
        test_bed.update(1);
        test_bed.assert_default_controls();
    }

    #[test]
    fn resets_invalid_fen() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        test_bed.type_fen_input_value("abc");
        test_bed.update(1);
        assert!(!test_bed.get_state().is_fen_valid());
        assert!(test_bed.get_load_button_disabled());

        test_bed.enter_fen_input_menu();
        test_bed.update(1);
        assert!(test_bed.get_state().is_fen_valid());
        test_bed.assert_default_controls();
    }

    #[test]
    fn resets_on_every_enter() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        test_bed.enter_fen_input_menu();
        test_bed.update(1);

        test_bed.type_fen_input_value(&non_default_fen());

        test_bed.leave_fen_input_menu();
        test_bed.update(1);
        test_bed.enter_fen_input_menu();
        test_bed.update(1);

        assert_eq!(&FenInputMenuState::default(), test_bed.get_state());
        test_bed.assert_default_controls();
    }
}

mod fen_input {
    use bevy::prelude::Visibility::*;
    use chess::{Color::*, File::*};

    use super::*;

    #[test]
    fn updates_fen() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let expected_fen = FenBuilder::random().to_string();

        test_bed.set_fen_input_value(&expected_fen);
        test_bed.update(1);

        assert_eq!(expected_fen, test_bed.get_fen_input_value());
        assert_eq!(expected_fen, test_bed.get_state().fen_with_counts());
    }

    #[test]
    fn keeps_controls_while_typing() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        test_bed.enter_fen_input_menu();
        test_bed.update(1);

        let fen = non_default_fen();
        test_bed.type_fen_input_value(&fen);
        test_bed.update(2);

        // Only the state follows the FEN.
        assert_eq!(fen, test_bed.get_state().fen_with_counts());
        test_bed.assert_default_position_controls();
    }

    #[test]
    fn updates_controls_on_blur() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let builder = FenBuilder::random();

        let expected_fen = builder.to_string();
        let expected_white_to_move = builder.bb.get_side_to_move() == White;
        let expected_black_to_move = !expected_white_to_move;
        let (expected_checked, expected_vis, expected_label) = match builder.bb.get_en_passant() {
            Some(sq) => (true, Visible, format!("{:?}", sq.get_file())),
            None => (false, Hidden, String::new()),
        };
        let expected_white = builder.bb.get_castle_rights(White);
        let expected_black = builder.bb.get_castle_rights(Black);
        let expected_halfmoves = builder.halfmove_clock.to_string();
        let expected_fullmoves = builder.fullmove_count.to_string();

        test_bed.type_fen_input_value(&expected_fen);
        test_bed.blur_fen_input();

        assert_eq!(expected_white_to_move, test_bed.get_side_to_move_button_white());
        assert_eq!(expected_black_to_move, test_bed.get_side_to_move_button_black());

        assert_eq!(expected_checked, test_bed.get_en_passant_checkbox());
        assert_eq!(expected_vis, test_bed.get_en_passant_file_menu_vis());
        assert_eq!(expected_vis, test_bed.get_en_passant_file_label_vis());
        if expected_checked {
            assert_eq!(expected_label, test_bed.get_en_passant_file_label());
        }

        assert_eq!(expected_white.has_kingside(), test_bed.get_castle_rights_w_k());
        assert_eq!(expected_white.has_queenside(), test_bed.get_castle_rights_w_q());
        assert_eq!(expected_black.has_kingside(), test_bed.get_castle_rights_b_k());
        assert_eq!(expected_black.has_queenside(), test_bed.get_castle_rights_b_q());

        assert_eq!(expected_halfmoves, test_bed.get_halfmove_clock());
        assert_eq!(expected_fullmoves, test_bed.get_fullmove_count());
    }

    #[test]
    fn indicates_invalid_fen() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        // Not indicated while typing.
        test_bed.type_fen_input_value("abc");
        assert!(!test_bed.get_state().is_fen_valid());
        assert!(!test_bed.get_fen_input_invalid());

        test_bed.blur_fen_input();
        assert!(test_bed.get_fen_input_invalid());
    }

    #[test]
    fn indicates_valid_fen_after_invalid() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        test_bed.type_fen_input_value("abc");
        test_bed.blur_fen_input();
        assert!(test_bed.get_fen_input_invalid());

        // Indicated immediately, without a blur.
        test_bed.type_fen_input_value(&FenBuilder::random().to_string());
        assert!(test_bed.get_state().is_fen_valid());
        assert!(!test_bed.get_fen_input_invalid());
    }

    #[test]
    fn indicates_invalid_fen_live_after_blur() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        test_bed.type_fen_input_value("abc");
        test_bed.blur_fen_input();
        assert!(test_bed.get_fen_input_invalid());

        test_bed.type_fen_input_value(&FenBuilder::random().to_string());
        assert!(!test_bed.get_fen_input_invalid());

        // Indicated immediately, without a blur, until the next blur.
        test_bed.type_fen_input_value("abc");
        assert!(test_bed.get_fen_input_invalid());

        test_bed.type_fen_input_value(&FenBuilder::random().to_string());
        assert!(!test_bed.get_fen_input_invalid());
    }

    #[test]
    fn stops_live_validation_after_valid_blur() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        test_bed.type_fen_input_value("abc");
        test_bed.blur_fen_input();
        assert!(test_bed.get_fen_input_invalid());

        test_bed.type_fen_input_value(&FenBuilder::random().to_string());
        test_bed.blur_fen_input();
        assert!(!test_bed.get_fen_input_invalid());

        // Back to only indicating on blur.
        test_bed.type_fen_input_value("abc");
        assert!(!test_bed.get_fen_input_invalid());

        test_bed.blur_fen_input();
        assert!(test_bed.get_fen_input_invalid());
    }

    #[test]
    fn resets_validation_on_enter() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        test_bed.type_fen_input_value("abc");
        test_bed.blur_fen_input();
        assert!(test_bed.get_fen_input_invalid());

        test_bed.enter_fen_input_menu();
        test_bed.update(1);
        assert!(!test_bed.get_fen_input_invalid());

        // Back to only indicating on blur.
        test_bed.type_fen_input_value("def");
        assert!(!test_bed.get_fen_input_invalid());

        test_bed.blur_fen_input();
        assert!(test_bed.get_fen_input_invalid());
    }

    #[test]
    fn ignores_unchanged_fen() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let update_count = test_bed.get_fen_update_count();

        test_bed.set_fen_input_value(&FenBuilder::random().to_string());
        test_bed.update(1);

        assert_eq!(update_count + 1, test_bed.get_fen_update_count());

        // Nothing syncs `EditableTextGeneration` in the test bed, so the FEN input reports a
        // text edit every frame even though its text has not changed.
        test_bed.update(1);

        assert_eq!(update_count + 1, test_bed.get_fen_update_count());
    }

    #[test]
    fn ignores_unchanged_invalid_fen() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let update_count = test_bed.get_fen_update_count();

        test_bed.set_fen_input_value("abc");
        test_bed.update(1);

        assert!(!test_bed.get_state().is_fen_valid());
        assert_eq!(update_count + 1, test_bed.get_fen_update_count());

        // The state is already invalid, so re-reading the same text does not change it.
        test_bed.update(3);

        assert_eq!(update_count + 1, test_bed.get_fen_update_count());

        // Neither does a different invalid FEN.
        test_bed.set_fen_input_value("def");
        test_bed.update(1);

        assert_eq!(update_count + 1, test_bed.get_fen_update_count());
    }

    #[test]
    fn ignores_fen_set_by_controls() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let cases: [(&str, fn(&mut TestBed)); 7] = [
            ("side to move black", |t| t.select_side_to_move(Black)),
            ("side to move white", |t| t.select_side_to_move(White)),
            ("en passant enabled", |t| t.set_en_passant_checkbox(true)),
            ("en passant B", |t| t.set_en_passant_file(B)),
            ("castle rights", |t| t.set_castle_rights_w_k(false)),
            ("halfmove clock", |t| t.set_halfmove_clock(50, true)),
            ("fullmove count", |t| t.set_fullmove_count(100, true)),
        ];

        let update_count = test_bed.get_fen_update_count();

        for (control, act) in cases {
            act(&mut test_bed);
            test_bed.update(1);

            assert_eq!(update_count, test_bed.get_fen_update_count(), "{control}");
        }
    }

    /// The default FEN with move counts that do not have to fit their types.
    fn fen_with_counts(halfmove: u64, fullmove: u64) -> String {
        let fen = FenBuilder::default().to_string();
        let position = fen.strip_suffix(" 0 1").unwrap();
        format!("{position} {halfmove} {fullmove}")
    }

    /// Typed halfmove and fullmove counts, then the counts they are capped at.
    const CAPPED_COUNTS: [(u64, u64, u8, u16); 4] = [
        (256, 1, u8::MAX, 1),
        (0, 65536, 0, u16::MAX),
        (300, 70000, u8::MAX, u16::MAX),
        (u64::MAX, u64::MAX, u8::MAX, u16::MAX),
    ];

    #[test]
    fn caps_move_counts() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        for (i, (halfmove, fullmove, expected_halfmove, expected_fullmove)) in
            CAPPED_COUNTS.into_iter().enumerate()
        {
            test_bed.type_fen_input_value(&fen_with_counts(halfmove, fullmove));

            assert_eq!(expected_halfmove, test_bed.get_state().halfmove_clock(), "{i}");
            assert_eq!(expected_fullmove, test_bed.get_state().fullmove_count(), "{i}");

            test_bed.blur_fen_input();
            assert_eq!(expected_halfmove.to_string(), test_bed.get_halfmove_clock(), "{i}");
            assert_eq!(expected_fullmove.to_string(), test_bed.get_fullmove_count(), "{i}");
        }
    }

    #[test]
    fn rewrites_capped_fen_on_blur() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        for (i, (halfmove, fullmove, expected_halfmove, expected_fullmove)) in
            CAPPED_COUNTS.into_iter().enumerate()
        {
            let typed = fen_with_counts(halfmove, fullmove);
            let expected_fen = FenBuilder::default()
                .halfmove_clock(expected_halfmove)
                .fullmove_count(expected_fullmove);

            // Left as typed while the input has focus.
            test_bed.type_fen_input_value(&typed);
            test_bed.update(2);
            assert_eq!(typed, test_bed.get_fen_input_value(), "{i}");

            test_bed.blur_fen_input();
            assert_eq!(expected_fen, test_bed.get_fen_input_value(), "{i}");
        }
    }

    #[test]
    fn rewrites_capped_fen_on_blur_when_state_is_unchanged() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let expected_fen = FenBuilder::default().halfmove_clock(u8::MAX).fullmove_count(u16::MAX);

        test_bed.type_fen_input_value(&expected_fen.to_string());
        let update_count = test_bed.get_fen_update_count();

        let typed = fen_with_counts(256, 65536);
        test_bed.type_fen_input_value(&typed);
        assert_eq!(update_count, test_bed.get_fen_update_count());
        assert_eq!(typed, test_bed.get_fen_input_value());

        test_bed.blur_fen_input();
        assert_eq!(expected_fen, test_bed.get_fen_input_value());
    }

    #[test]
    fn keeps_uncapped_fen_as_typed() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        // Valid, but not how the state would write it.
        let typed = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b - -";
        test_bed.type_fen_input_value(typed);
        test_bed.update(1);

        assert!(test_bed.get_state().is_fen_valid());
        assert_eq!(typed, test_bed.get_fen_input_value());

        test_bed.blur_fen_input();
        assert_eq!(typed, test_bed.get_fen_input_value());
    }

    #[test]
    fn keeps_invalid_fen_with_capped_counts_as_typed() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let typed = "abc def ghi jkl 300 70000";
        test_bed.type_fen_input_value(typed);
        test_bed.update(1);

        assert!(!test_bed.get_state().is_fen_valid());
        assert_eq!(typed, test_bed.get_fen_input_value());

        test_bed.blur_fen_input();
        assert_eq!(typed, test_bed.get_fen_input_value());
    }
}

mod fen_input_invalid {
    use super::*;

    #[test]
    fn default_border_color() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        assert!(!test_bed.get_fen_input_invalid());
        test_bed.assert_fen_input_border(&NONE);
    }

    #[test]
    fn sets_border_color_on_add() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        test_bed.set_fen_input_invalid(true);
        test_bed.update(1);

        test_bed.assert_fen_input_border(&RED);
    }

    #[test]
    fn resets_border_color_on_remove() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        test_bed.set_fen_input_invalid(true);
        test_bed.update(1);
        test_bed.set_fen_input_invalid(false);
        test_bed.update(1);

        test_bed.assert_fen_input_border(&NONE);
    }
}

mod side_to_move {
    use chess::ALL_COLORS;

    use super::*;

    #[test]
    fn updates_state_on_interaction() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        for color in ALL_COLORS {
            let expected_fen = FenBuilder::default().side_to_move(color);

            test_bed.select_side_to_move(color);
            test_bed.update(1);

            assert_eq!(color, test_bed.get_state().side_to_move(), "{color:?}");
            assert_eq!(expected_fen, test_bed.get_state().fen_with_counts(), "{color:?}");
        }
    }

    #[test]
    fn updates_fen_input_on_interaction() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        for color in ALL_COLORS {
            let expected_fen = FenBuilder::default().side_to_move(color);

            test_bed.select_side_to_move(color);
            test_bed.update(1);

            assert_eq!(expected_fen, test_bed.get_fen_input_value(), "{color:?}");
        }
    }

    #[test]
    fn updates_invalid_fen() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        for color in ALL_COLORS {
            let expected_fen = FenBuilder::default().side_to_move(color);

            test_bed.set_fen_input_value("abc");
            test_bed.update(1);
            assert!(!test_bed.get_state().is_fen_valid(), "{color:?}");

            test_bed.select_side_to_move(color);
            test_bed.update(1);

            assert_eq!(color, test_bed.get_state().side_to_move(), "{color:?}");
            assert!(test_bed.get_state().is_fen_valid(), "{color:?}");
            assert_eq!(expected_fen, test_bed.get_state().fen_with_counts(), "{color:?}");
            assert_eq!(expected_fen, test_bed.get_fen_input_value(), "{color:?}");
        }
    }
}

mod en_passant {
    use bevy::prelude::Visibility::*;
    use chess::File::{self, *};

    use super::*;

    #[test]
    fn updates_state_on_interaction() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let cases: [(Option<File>, fn(&mut TestBed)); 5] = [
            (Some(A), |t| t.set_en_passant_checkbox(true)),
            (Some(B), |t| t.set_en_passant_file(B)),
            (Some(H), |t| t.set_en_passant_file(H)),
            (None, |t| t.set_en_passant_checkbox(false)),
            (Some(A), |t| t.set_en_passant_checkbox(true)),
        ];

        for (i, (expected_file, act)) in cases.into_iter().enumerate() {
            let expected_fen = FenBuilder::default().en_passant(expected_file);

            act(&mut test_bed);
            test_bed.update(1);

            assert_eq!(expected_file, test_bed.get_state().en_passant_file(), "{i}");
            assert_eq!(expected_fen, test_bed.get_state().fen_with_counts(), "{i}");
        }
    }

    #[test]
    fn updates_file_menu_on_interaction() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let cases: [(Visibility, fn(&mut TestBed)); 5] = [
            (Visible, |t| t.set_en_passant_checkbox(true)),
            (Visible, |t| t.set_en_passant_file(B)),
            (Visible, |t| t.set_en_passant_file(H)),
            (Hidden, |t| t.set_en_passant_checkbox(false)),
            (Visible, |t| t.set_en_passant_checkbox(true)),
        ];

        for (i, (expected_vis, act)) in cases.into_iter().enumerate() {
            act(&mut test_bed);
            test_bed.update(1);

            assert_eq!(expected_vis, test_bed.get_en_passant_file_menu_vis(), "{i}");
        }
    }

    #[test]
    fn updates_file_label_on_interaction() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let cases: [(Visibility, &str, fn(&mut TestBed)); 5] = [
            (Visible, "A", |t| t.set_en_passant_checkbox(true)),
            (Visible, "B", |t| t.set_en_passant_file(B)),
            (Visible, "H", |t| t.set_en_passant_file(H)),
            (Hidden, "", |t| t.set_en_passant_checkbox(false)),
            (Visible, "A", |t| t.set_en_passant_checkbox(true)),
        ];

        for (i, (expected_vis, expected_label, act)) in cases.into_iter().enumerate() {
            act(&mut test_bed);
            test_bed.update(1);

            assert_eq!(expected_vis, test_bed.get_en_passant_file_label_vis(), "{i}");
            if expected_vis == Visible {
                assert_eq!(expected_label, test_bed.get_en_passant_file_label(), "{i}");
            }
        }
    }

    #[test]
    fn updates_fen_input_on_interaction() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let cases: [(Option<File>, fn(&mut TestBed)); 5] = [
            (Some(A), |t| t.set_en_passant_checkbox(true)),
            (Some(B), |t| t.set_en_passant_file(B)),
            (Some(H), |t| t.set_en_passant_file(H)),
            (None, |t| t.set_en_passant_checkbox(false)),
            (Some(A), |t| t.set_en_passant_checkbox(true)),
        ];

        for (i, (expected_file, act)) in cases.into_iter().enumerate() {
            let expected_fen = FenBuilder::default().en_passant(expected_file);

            act(&mut test_bed);
            test_bed.update(1);

            assert_eq!(expected_fen, test_bed.get_fen_input_value(), "{i}");
        }
    }

    #[test]
    fn updates_invalid_fen() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let cases: [(Option<File>, fn(&mut TestBed)); 5] = [
            (Some(A), |t| t.set_en_passant_checkbox(true)),
            (Some(B), |t| t.set_en_passant_file(B)),
            (Some(H), |t| t.set_en_passant_file(H)),
            (None, |t| t.set_en_passant_checkbox(false)),
            (Some(A), |t| t.set_en_passant_checkbox(true)),
        ];

        for (i, (expected_file, act)) in cases.into_iter().enumerate() {
            let expected_fen = FenBuilder::default().en_passant(expected_file);

            test_bed.set_fen_input_value("abc");
            test_bed.update(1);
            assert!(!test_bed.get_state().is_fen_valid(), "{i}");

            act(&mut test_bed);
            test_bed.update(1);

            assert_eq!(expected_file, test_bed.get_state().en_passant_file(), "{i}");
            assert!(test_bed.get_state().is_fen_valid(), "{i}");
            assert_eq!(expected_fen, test_bed.get_state().fen_with_counts(), "{i}");
            assert_eq!(expected_fen, test_bed.get_fen_input_value(), "{i}");
        }
    }
}

mod castle_rights {
    use chess::CastleRights::{self, *};

    use super::*;

    #[test]
    fn updates_state_on_interaction() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let cases: [(CastleRights, CastleRights, fn(&mut TestBed)); 4] = [
            (QueenSide, Both, |t| t.set_castle_rights_w_k(false)),
            (NoRights, Both, |t| t.set_castle_rights_w_q(false)),
            (NoRights, QueenSide, |t| t.set_castle_rights_b_k(false)),
            (NoRights, NoRights, |t| t.set_castle_rights_b_q(false)),
        ];

        for (i, (expected_white, expected_black, act)) in cases.into_iter().enumerate() {
            let expected_fen = FenBuilder::default().castle_rights(expected_white, expected_black);

            act(&mut test_bed);
            test_bed.update(1);

            assert_eq!(expected_white, test_bed.get_state().castle_rights_white(), "{i}");
            assert_eq!(expected_black, test_bed.get_state().castle_rights_black(), "{i}");
            assert_eq!(expected_fen, test_bed.get_state().fen_with_counts(), "{i}");
        }
    }

    #[test]
    fn updates_fen_input_on_interaction() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let cases: [(CastleRights, CastleRights, fn(&mut TestBed)); 4] = [
            (QueenSide, Both, |t| t.set_castle_rights_w_k(false)),
            (NoRights, Both, |t| t.set_castle_rights_w_q(false)),
            (NoRights, QueenSide, |t| t.set_castle_rights_b_k(false)),
            (NoRights, NoRights, |t| t.set_castle_rights_b_q(false)),
        ];

        for (i, (expected_white, expected_black, act)) in cases.into_iter().enumerate() {
            let expected_fen = FenBuilder::default().castle_rights(expected_white, expected_black);

            act(&mut test_bed);
            test_bed.update(1);

            assert_eq!(expected_fen, test_bed.get_fen_input_value(), "{i}");
        }
    }

    #[test]
    fn updates_invalid_fen() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let cases: [(CastleRights, CastleRights, fn(&mut TestBed)); 4] = [
            (QueenSide, Both, |t| t.set_castle_rights_w_k(false)),
            (NoRights, Both, |t| t.set_castle_rights_w_q(false)),
            (NoRights, QueenSide, |t| t.set_castle_rights_b_k(false)),
            (NoRights, NoRights, |t| t.set_castle_rights_b_q(false)),
        ];

        for (i, (expected_white, expected_black, act)) in cases.into_iter().enumerate() {
            let expected_fen = FenBuilder::default().castle_rights(expected_white, expected_black);

            test_bed.set_fen_input_value("abc");
            test_bed.update(1);
            assert!(!test_bed.get_state().is_fen_valid(), "{i}");

            act(&mut test_bed);
            test_bed.update(1);

            assert_eq!(expected_white, test_bed.get_state().castle_rights_white(), "{i}");
            assert_eq!(expected_black, test_bed.get_state().castle_rights_black(), "{i}");
            assert!(test_bed.get_state().is_fen_valid(), "{i}");
            assert_eq!(expected_fen, test_bed.get_state().fen_with_counts(), "{i}");
            assert_eq!(expected_fen, test_bed.get_fen_input_value(), "{i}");
        }
    }
}

mod halfmove_clock {
    use super::*;

    const HALFMOVES: [u8; 4] = [1, 50, u8::MAX, 0];

    #[test]
    fn updates_state_on_interaction() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        for expected in HALFMOVES {
            let expected_fen = FenBuilder::default().halfmove_clock(expected);

            test_bed.set_halfmove_clock(expected, true);
            test_bed.update(1);

            assert_eq!(expected, test_bed.get_state().halfmove_clock(), "{expected}");
            assert_eq!(expected_fen, test_bed.get_state().fen_with_counts(), "{expected}");
        }
    }

    #[test]
    fn updates_fen_input_on_interaction() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        for expected in HALFMOVES {
            let expected_fen = FenBuilder::default().halfmove_clock(expected);

            test_bed.set_halfmove_clock(expected, true);
            test_bed.update(1);

            assert_eq!(expected_fen, test_bed.get_fen_input_value(), "{expected}");
        }
    }

    #[test]
    fn updates_invalid_fen() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        for expected in HALFMOVES {
            let expected_fen = FenBuilder::default().halfmove_clock(expected);

            test_bed.set_fen_input_value("abc");
            test_bed.update(1);
            assert!(!test_bed.get_state().is_fen_valid(), "{expected}");

            test_bed.set_halfmove_clock(expected, true);
            test_bed.update(1);

            assert_eq!(expected, test_bed.get_state().halfmove_clock(), "{expected}");
            assert!(test_bed.get_state().is_fen_valid(), "{expected}");
            assert_eq!(expected_fen, test_bed.get_state().fen_with_counts(), "{expected}");
            assert_eq!(expected_fen, test_bed.get_fen_input_value(), "{expected}");
        }
    }

    #[test]
    fn ignores_non_final_interaction() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let fen_input_before = test_bed.get_fen_input_value();

        test_bed.set_halfmove_clock(50, false);
        test_bed.update(1);

        assert_eq!(0, test_bed.get_state().halfmove_clock());
        assert_eq!(FenBuilder::default(), test_bed.get_state().fen_with_counts());
        assert_eq!(fen_input_before, test_bed.get_fen_input_value());
    }

    #[test]
    fn updates_state_on_typing() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        for expected in HALFMOVES {
            let expected_fen = FenBuilder::default().halfmove_clock(expected);

            test_bed.type_halfmove_clock(&expected.to_string());

            assert_eq!(expected, test_bed.get_state().halfmove_clock(), "{expected}");
            assert_eq!(expected_fen, test_bed.get_fen_input_value(), "{expected}");
        }
    }

    #[test]
    fn caps_at_max() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let expected_fen = FenBuilder::default().halfmove_clock(u8::MAX);

        for typed in [u8::MAX.to_string(), i32::MAX.to_string()] {
            test_bed.type_halfmove_clock(&typed);

            assert_eq!(u8::MAX, test_bed.get_state().halfmove_clock(), "{typed}");
            assert_eq!(u8::MAX.to_string(), test_bed.get_halfmove_clock(), "{typed}");
            assert_eq!(expected_fen, test_bed.get_fen_input_value(), "{typed}");
        }
    }

    #[test]
    fn ignores_negative_value() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        test_bed.type_halfmove_clock("7");
        assert_eq!(7, test_bed.get_state().halfmove_clock());
        let fen_input_before = test_bed.get_fen_input_value();

        test_bed.type_halfmove_clock("-1");

        assert_eq!(7, test_bed.get_state().halfmove_clock());
        assert_eq!("7", test_bed.get_halfmove_clock());
        assert_eq!(fen_input_before, test_bed.get_fen_input_value());
    }
}

mod fullmove_count {
    use super::*;

    const FULLMOVES: [u16; 4] = [2, 100, u16::MAX, 1];

    #[test]
    fn updates_state_on_interaction() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        for expected in FULLMOVES {
            let expected_fen = FenBuilder::default().fullmove_count(expected);

            test_bed.set_fullmove_count(expected, true);
            test_bed.update(1);

            assert_eq!(expected, test_bed.get_state().fullmove_count(), "{expected}");
            assert_eq!(expected_fen, test_bed.get_state().fen_with_counts(), "{expected}");
        }
    }

    #[test]
    fn updates_fen_input_on_interaction() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        for expected in FULLMOVES {
            let expected_fen = FenBuilder::default().fullmove_count(expected);

            test_bed.set_fullmove_count(expected, true);
            test_bed.update(1);

            assert_eq!(expected_fen, test_bed.get_fen_input_value(), "{expected}");
        }
    }

    #[test]
    fn updates_invalid_fen() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        for expected in FULLMOVES {
            let expected_fen = FenBuilder::default().fullmove_count(expected);

            test_bed.set_fen_input_value("abc");
            test_bed.update(1);
            assert!(!test_bed.get_state().is_fen_valid(), "{expected}");

            test_bed.set_fullmove_count(expected, true);
            test_bed.update(1);

            assert_eq!(expected, test_bed.get_state().fullmove_count(), "{expected}");
            assert!(test_bed.get_state().is_fen_valid(), "{expected}");
            assert_eq!(expected_fen, test_bed.get_state().fen_with_counts(), "{expected}");
            assert_eq!(expected_fen, test_bed.get_fen_input_value(), "{expected}");
        }
    }

    #[test]
    fn ignores_non_final_interaction() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let fen_input_before = test_bed.get_fen_input_value();

        test_bed.set_fullmove_count(100, false);
        test_bed.update(1);

        assert_eq!(1, test_bed.get_state().fullmove_count());
        assert_eq!(FenBuilder::default(), test_bed.get_state().fen_with_counts());
        assert_eq!(fen_input_before, test_bed.get_fen_input_value());
    }

    #[test]
    fn updates_state_on_typing() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        for expected in FULLMOVES {
            let expected_fen = FenBuilder::default().fullmove_count(expected);

            test_bed.type_fullmove_count(&expected.to_string());

            assert_eq!(expected, test_bed.get_state().fullmove_count(), "{expected}");
            assert_eq!(expected_fen, test_bed.get_fen_input_value(), "{expected}");
        }
    }

    #[test]
    fn caps_at_max() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let expected_fen = FenBuilder::default().fullmove_count(u16::MAX);

        for typed in [u16::MAX.to_string(), i32::MAX.to_string()] {
            test_bed.type_fullmove_count(&typed);

            assert_eq!(u16::MAX, test_bed.get_state().fullmove_count(), "{typed}");
            assert_eq!(u16::MAX.to_string(), test_bed.get_fullmove_count(), "{typed}");
            assert_eq!(expected_fen, test_bed.get_fen_input_value(), "{typed}");
        }
    }

    #[test]
    fn ignores_negative_value() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        test_bed.type_fullmove_count("7");
        assert_eq!(7, test_bed.get_state().fullmove_count());
        let fen_input_before = test_bed.get_fen_input_value();

        test_bed.type_fullmove_count("-1");

        assert_eq!(7, test_bed.get_state().fullmove_count());
        assert_eq!("7", test_bed.get_fullmove_count());
        assert_eq!(fen_input_before, test_bed.get_fen_input_value());
    }
}

mod load_button {
    use chess::{Color::*, File::*};

    use super::*;

    #[test]
    fn disabled_on_invalid_fen() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        test_bed.set_fen_input_value("abc");
        test_bed.update(2);

        assert!(!test_bed.get_state().is_fen_valid());
        assert!(test_bed.get_load_button_disabled());
    }

    #[test]
    fn enabled_on_valid_fen() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        test_bed.set_fen_input_value("abc");
        test_bed.update(2);

        assert!(!test_bed.get_state().is_fen_valid());
        assert!(test_bed.get_load_button_disabled());

        test_bed.set_fen_input_value(&FenBuilder::random().to_string());
        test_bed.update(2);

        assert!(test_bed.get_state().is_fen_valid());
        assert!(!test_bed.get_load_button_disabled());
    }

    #[test]
    fn enabled_on_interaction() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let cases: [(&str, fn(&mut TestBed)); 7] = [
            ("side to move black", |t| t.select_side_to_move(Black)),
            ("side to move white", |t| t.select_side_to_move(White)),
            ("en passant enabled", |t| t.set_en_passant_checkbox(true)),
            ("en passant B", |t| t.set_en_passant_file(B)),
            ("castle rights", |t| t.set_castle_rights_w_k(false)),
            ("halfmove clock", |t| t.set_halfmove_clock(50, true)),
            ("fullmove count", |t| t.set_fullmove_count(100, true)),
        ];

        for (control, act) in cases {
            test_bed.set_fen_input_value("abc");
            test_bed.update(2);

            assert!(!test_bed.get_state().is_fen_valid());
            assert!(test_bed.get_load_button_disabled(), "{control}");

            act(&mut test_bed);
            test_bed.update(1);

            assert!(test_bed.get_state().is_fen_valid(), "{control}");
            assert!(!test_bed.get_load_button_disabled(), "{control}");
        }
    }
}
