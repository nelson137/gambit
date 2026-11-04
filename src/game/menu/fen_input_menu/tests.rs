use std::{fmt, sync::LazyLock};

use bevy::{
    ecs::system::SystemState,
    ui_widgets::{Activate, ValueChange},
};

use crate::utils::random_values;

use super::*;

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
        app.add_plugins(FenInputMenuPlugin);
        app.init_resource::<FenInputMenuState>();
        app.init_resource::<FenTextEditChangeHandler>();
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
}

// State
impl TestBed {
    fn get_state(&self) -> &FenInputMenuState {
        self.app.world().resource::<FenInputMenuState>()
    }
}

// Controls
impl TestBed {
    fn get_fen_input_border_color(&mut self) -> BorderColor {
        *self.element_mut::<FenTextInputContainer>().get::<BorderColor>().unwrap()
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

    fn get_fen_parse_count(&self) -> usize {
        self.app.world().resource::<FenTextEditChangeHandler>().parse_count
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

    fn set_halfmove_clock(&mut self, halfmoves: u8, is_final: bool) {
        self.element_set_number_input::<HalfmoveClockNumberInput>(halfmoves as i32, is_final);
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

    fn set_fullmove_count(&mut self, fullmoves: u16, is_final: bool) {
        self.element_set_number_input::<FullmoveCountNumberInput>(fullmoves as i32, is_final);
    }
}

#[test]
fn default_state() {
    let test_bed = TestBed::new();

    assert_eq!(&FenInputMenuState::default(), test_bed.get_state());
}

#[test]
fn default_fen_text_edit_border() {
    let mut test_bed = TestBed::new();
    test_bed.update(1);

    let border_color = test_bed.get_fen_input_border_color();

    assert_eq!(*NONE, border_color.top.to_srgba().to_hex(), "top color");
    assert_eq!(*NONE, border_color.right.to_srgba().to_hex(), "right color");
    assert_eq!(*NONE, border_color.bottom.to_srgba().to_hex(), "bottom color");
    assert_eq!(*NONE, border_color.left.to_srgba().to_hex(), "left color");
}

#[test]
fn default_controls() {
    let mut test_bed = TestBed::new();
    test_bed.update(1);

    assert!(test_bed.get_side_to_move_button_white());
    assert!(!test_bed.get_side_to_move_button_black());

    assert!(!test_bed.get_en_passant_checkbox());
    assert_eq!(Visibility::Hidden, test_bed.get_en_passant_file_menu_vis());
    assert_eq!(Visibility::Hidden, test_bed.get_en_passant_file_label_vis());

    assert!(test_bed.get_castle_rights_w_k());
    assert!(test_bed.get_castle_rights_w_q());
    assert!(test_bed.get_castle_rights_b_k());
    assert!(test_bed.get_castle_rights_b_q());

    assert_eq!("0", test_bed.get_halfmove_clock());
    assert_eq!("1", test_bed.get_fullmove_count());
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
    fn updates_controls() {
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

        test_bed.set_fen_input_value(&expected_fen);
        test_bed.update(1);

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

        test_bed.set_fen_input_value("abc");
        test_bed.update(2);

        assert!(!test_bed.get_state().is_fen_valid());

        let border_color = test_bed.get_fen_input_border_color();

        assert_eq!(*RED, border_color.top.to_srgba().to_hex(), "top border");
        assert_eq!(*RED, border_color.right.to_srgba().to_hex(), "right border");
        assert_eq!(*RED, border_color.bottom.to_srgba().to_hex(), "bottom border");
        assert_eq!(*RED, border_color.left.to_srgba().to_hex(), "left border");
    }

    #[test]
    fn ignores_unchanged_fen() {
        let mut test_bed = TestBed::new();
        test_bed.update(1);

        let parse_count = test_bed.get_fen_parse_count();

        test_bed.set_fen_input_value(&FenBuilder::random().to_string());
        test_bed.update(1);

        assert_eq!(parse_count + 1, test_bed.get_fen_parse_count());

        // Nothing syncs `EditableTextGeneration` in the test bed, so the FEN input reports a
        // text edit every frame even though its text has not changed.
        test_bed.update(1);

        assert_eq!(parse_count + 1, test_bed.get_fen_parse_count());
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

        let parse_count = test_bed.get_fen_parse_count();

        for (control, act) in cases {
            act(&mut test_bed);
            test_bed.update(1);

            assert_eq!(parse_count, test_bed.get_fen_parse_count(), "{control}");
        }
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
}
