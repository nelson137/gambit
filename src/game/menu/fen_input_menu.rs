use std::{fmt::Write, str::FromStr};

use bevy::{
    color::palettes,
    ecs::system::SystemParam,
    feathers::{
        self,
        constants::{fonts, size},
    },
    picking,
    prelude::*,
    text::{self, TextSection},
    ui, ui_widgets,
};

use crate::{
    game::{LoadGame, consts},
    utils::{GameFontWeight, text_font},
};

#[derive(Resource)]
pub struct FenInputMenuState {
    board_builder: chess::BoardBuilder,
    is_fen_valid: bool,
    halfmove_clock: u8,
    fullmove_count: u16,
}

impl Default for FenInputMenuState {
    fn default() -> Self {
        Self {
            board_builder: chess::BoardBuilder::default(),
            is_fen_valid: true,
            halfmove_clock: 0,
            fullmove_count: 1,
        }
    }
}

impl FenInputMenuState {
    fn fen_with_counts(&self) -> String {
        let mut fen = self.board_builder.to_string();

        truncate_trailing_whitespace(&mut fen);

        // Remove full-move count.
        let i = fen.rfind(' ').unwrap_or(fen.len());
        fen.truncate(i);

        truncate_trailing_whitespace(&mut fen);

        // Remove half-move clock.
        let i = fen.rfind(' ').unwrap_or(fen.len());
        fen.truncate(i);

        truncate_trailing_whitespace(&mut fen);

        write!(&mut fen, " {} {}", self.halfmove_clock, self.fullmove_count).unwrap();

        fen
    }

    fn side_to_move(&self) -> chess::Color {
        self.board_builder.get_side_to_move()
    }

    fn en_passant_file(&self) -> Option<chess::File> {
        self.board_builder.get_en_passant().map(|sq| sq.get_file())
    }

    fn castle_rights_white(&self) -> chess::CastleRights {
        self.board_builder.get_castle_rights(chess::Color::White)
    }

    fn castle_rights_black(&self) -> chess::CastleRights {
        self.board_builder.get_castle_rights(chess::Color::Black)
    }

    fn try_board(&self) -> Result<chess::Board, chess::Error> {
        self.board_builder.try_into()
    }
}

impl FenInputMenuState {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    fn set_fen(&mut self, fen: String) {
        let fen = if fen.trim().is_empty() { consts::DEFAULT_FEN } else { &fen };

        self.is_fen_valid = match chess::BoardBuilder::from_str(fen) {
            Ok(next) => {
                self.board_builder = next;

                let mut fen_iter = fen.split(' ').skip(4);
                let halfmove = fen_iter.next().and_then(|s| s.parse().ok());
                let fullmove = fen_iter.next().and_then(|s| s.parse().ok());
                if let Some(halfmove) = halfmove {
                    self.halfmove_clock = halfmove;
                }
                if let Some(fullmove) = fullmove {
                    self.fullmove_count = fullmove;
                }

                true
            }
            Err(_) => false,
        };
    }

    fn set_side_to_move(&mut self, color: chess::Color) -> String {
        self.board_builder.side_to_move(color);
        self.is_fen_valid = true;
        self.fen_with_counts()
    }

    fn set_en_passant(&mut self, file: Option<chess::File>) -> String {
        self.board_builder.en_passant(file);
        self.is_fen_valid = true;
        self.fen_with_counts()
    }

    fn add_castle_rights(&mut self, color: chess::Color, rights: chess::CastleRights) -> String {
        let castle_rights = self.board_builder.get_castle_rights(color);
        self.board_builder.castle_rights(color, castle_rights.add(rights));
        self.is_fen_valid = true;
        self.fen_with_counts()
    }

    fn remove_castle_rights(&mut self, color: chess::Color, rights: chess::CastleRights) -> String {
        let castle_rights = self.board_builder.get_castle_rights(color);
        self.board_builder.castle_rights(color, castle_rights.remove(rights));
        self.is_fen_valid = true;
        self.fen_with_counts()
    }

    fn set_halfmove_clock(&mut self, halfmove: u8) -> String {
        self.halfmove_clock = halfmove;
        self.is_fen_valid = true;
        self.fen_with_counts()
    }

    fn set_fullmove_count(&mut self, fullmoves: u16) -> String {
        self.fullmove_count = fullmoves;
        self.is_fen_valid = true;
        self.fen_with_counts()
    }
}

fn truncate_trailing_whitespace(s: &mut String) {
    let trailing_len = s.len() - s.trim_end().len();
    s.truncate(s.len() - trailing_len);
}

#[derive(Component, Clone, Default)]
pub(super) struct FenInputMenu;

#[derive(Component, Clone, Copy, Default)]
pub(super) struct FenTextInputContainer;

#[derive(Component, Clone, Copy, Default)]
struct FenTextInput;

#[derive(Component, Clone, Copy, Default)]
struct SideToMoveRadioButtonWhite;

#[derive(Component, Clone, Copy, Default)]
struct SideToMoveRadioButtonBlack;

#[derive(Component, Clone, Copy, Default)]
struct EnPassantCheckbox;

#[derive(Component, Clone, Copy, Default)]
struct EnPassantMenu;

#[derive(Component, Clone, Copy, Default)]
struct EnPassantFileLabel;

static EN_PASSANT_FILE_LABELS: [&str; 8] = ["A", "B", "C", "D", "E", "F", "G", "H"];

#[derive(Component, Clone, Copy, Default)]
struct CastleRightsWhiteKingsideCheckbox;

#[derive(Component, Clone, Copy, Default)]
struct CastleRightsWhiteQueensideCheckbox;

#[derive(Component, Clone, Copy, Default)]
struct CastleRightsBlackKingsideCheckbox;

#[derive(Component, Clone, Copy, Default)]
struct CastleRightsBlackQueensideCheckbox;

#[derive(Component, Clone, Copy, Default)]
struct HalfmoveClockNumberInput;

#[derive(Component, Clone, Copy, Default)]
struct FullmoveCountNumberInput;

#[derive(Component, Clone, Copy, Default)]
struct CancelButton;

#[derive(Component, Clone, Copy, Default)]
struct LoadButton;

type ControlQuery<'w, 's, Marker> = Query<'w, 's, Entity, With<Marker>>;

#[derive(SystemParam)]
struct FenInputMenuControls<'w, 's> {
    commands: Commands<'w, 's>,
    q_fen_text_input: ControlQuery<'w, 's, FenTextInput>,
    q_side_to_move_button_white: ControlQuery<'w, 's, SideToMoveRadioButtonWhite>,
    q_side_to_move_button_black: ControlQuery<'w, 's, SideToMoveRadioButtonBlack>,
    q_en_passant_checkbox: ControlQuery<'w, 's, EnPassantCheckbox>,
    q_en_passant_menu: ControlQuery<'w, 's, EnPassantMenu>,
    q_en_passant_file_label: ControlQuery<'w, 's, EnPassantFileLabel>,
    q_castle_checkbox_white_kingside: ControlQuery<'w, 's, CastleRightsWhiteKingsideCheckbox>,
    q_castle_checkbox_white_queenside: ControlQuery<'w, 's, CastleRightsWhiteQueensideCheckbox>,
    q_castle_checkbox_black_kingside: ControlQuery<'w, 's, CastleRightsBlackKingsideCheckbox>,
    q_castle_checkbox_black_queenside: ControlQuery<'w, 's, CastleRightsBlackQueensideCheckbox>,
    q_halfmove_number_input: ControlQuery<'w, 's, HalfmoveClockNumberInput>,
    q_fullmove_number_input: ControlQuery<'w, 's, FullmoveCountNumberInput>,
}

// FEN Text Input
impl FenInputMenuControls<'_, '_> {
    fn set_fen_text_input_value(&mut self, fen: String) {
        let entity = self.q_fen_text_input.single().unwrap();
        self.commands.queue(move |world: &mut World| {
            let mut entity = world.entity_mut(entity);
            let mut editable = entity.get_mut::<text::EditableText>().unwrap();
            editable.editor.set_text(&fen);
        });
    }
}

// Side To Move Radio Group
impl FenInputMenuControls<'_, '_> {
    fn side_to_move_button_color(&self, entity: Entity) -> Option<chess::Color> {
        let entity_white = self.q_side_to_move_button_white.single().unwrap();
        let entity_black = self.q_side_to_move_button_black.single().unwrap();
        match entity {
            e if e == entity_white => Some(chess::Color::White),
            e if e == entity_black => Some(chess::Color::Black),
            _ => None,
        }
    }

    fn set_side_to_move(&mut self, color: chess::Color) {
        let e_white = self.q_side_to_move_button_white.single().unwrap();
        let e_black = self.q_side_to_move_button_black.single().unwrap();
        match color {
            chess::Color::White => {
                self.commands.entity(e_white).insert(ui::Checked);
                self.commands.entity(e_black).remove::<ui::Checked>();
            }
            chess::Color::Black => {
                self.commands.entity(e_white).remove::<ui::Checked>();
                self.commands.entity(e_black).insert(ui::Checked);
            }
        }
    }
}

// En Passant Checkbox & Menu
impl FenInputMenuControls<'_, '_> {
    fn set_en_passant(&mut self, file: Option<chess::File>) {
        self.set_en_passant_checkbox(file.is_some());
        self.set_en_passant_menu(file);
    }

    fn set_en_passant_checkbox(&mut self, enabled: bool) {
        let entity = self.q_en_passant_checkbox.single().unwrap();
        let mut entity = self.commands.entity(entity);
        if enabled {
            entity.insert(ui::Checked);
        } else {
            entity.remove::<ui::Checked>();
        }
    }

    fn set_en_passant_menu(&mut self, file: Option<chess::File>) {
        let visibility = if file.is_some() { Visibility::Visible } else { Visibility::Hidden };
        let entity = self.q_en_passant_menu.single().unwrap();
        self.commands.entity(entity).insert(visibility);
        let entity = self.q_en_passant_file_label.single().unwrap();
        self.commands.entity(entity).insert(visibility);
        if let Some(file) = file {
            self.commands.queue(move |world: &mut World| {
                let mut entity = world.entity_mut(entity);
                let mut text = entity.get_mut::<Text>().unwrap();
                let text = text.get_text_mut();
                text.replace_range(.., EN_PASSANT_FILE_LABELS[file.to_index()]);
            });
        }
    }
}

// Castle Rights
impl FenInputMenuControls<'_, '_> {
    fn set_castle_rights(
        &mut self,
        white_rights: chess::CastleRights,
        black_rights: chess::CastleRights,
    ) {
        let entity = self.q_castle_checkbox_white_kingside.single().unwrap();
        self.set_castle_right_checkbox(entity, white_rights.has_kingside());

        let entity = self.q_castle_checkbox_white_queenside.single().unwrap();
        self.set_castle_right_checkbox(entity, white_rights.has_queenside());

        let entity = self.q_castle_checkbox_black_kingside.single().unwrap();
        self.set_castle_right_checkbox(entity, black_rights.has_kingside());

        let entity = self.q_castle_checkbox_black_queenside.single().unwrap();
        self.set_castle_right_checkbox(entity, black_rights.has_queenside());
    }

    fn set_castle_right_checkbox(&mut self, entity: Entity, has_right: bool) {
        let mut entity = self.commands.entity(entity);
        if has_right {
            entity.insert(ui::Checked);
        } else {
            entity.remove::<ui::Checked>();
        }
    }
}

// Move Counts
impl FenInputMenuControls<'_, '_> {
    fn set_halfmove_clock(&mut self, halfmoves: u8) {
        let e_number_input = self.q_halfmove_number_input.single().unwrap();
        self.commands.queue(move |world: &mut World| {
            let mut entity = world.entity_mut({
                let children = world.entity(e_number_input).get::<Children>().unwrap();
                let has_editable = |c: &Entity| world.entity(*c).contains::<text::EditableText>();
                children.iter().find(has_editable).unwrap()
            });
            let mut editable = entity.get_mut::<text::EditableText>().unwrap();
            let text = format!("{halfmoves}");
            editable.editor.set_text(&text);
        });
    }

    fn set_fullmove_count(&mut self, fullmoves: u16) {
        let e_number_input = self.q_fullmove_number_input.single().unwrap();
        self.commands.queue(move |world: &mut World| {
            let mut entity = world.entity_mut({
                let children = world.entity(e_number_input).get::<Children>().unwrap();
                let has_editable = |c: &Entity| world.entity(*c).contains::<text::EditableText>();
                children.iter().find(has_editable).unwrap()
            });
            let mut editable = entity.get_mut::<text::EditableText>().unwrap();
            let text = format!("{fullmoves}");
            editable.editor.set_text(&text);
        });
    }
}

fn item_gap() -> Val {
    vh(1.5)
}

fn column_gap() -> Val {
    vh(8)
}

fn item_border_radius() -> BorderRadius {
    BorderRadius::all(px(4))
}

pub(super) fn ui() -> impl Scene {
    bsn! {
        #FenInputMenu
        FenInputMenu
        feathers::containers::pane()
        Children [
            (
                feathers::containers::pane_body()
                feathers::theme::ThemeBackgroundColor({ super::tokens::GAME_MENU_BG })
                Node {
                    padding: { UiRect::all(vh(3)) },
                    border_radius: {
                        feathers::rounded_corners::RoundedCorners::All.to_border_radius(12.0)
                    },
                }
                Children [
                    (
                        Text("Load Game")
                        text_font(FontSize::Vh(4.), GameFontWeight::Bold)
                        template_value(text::LineHeight::RelativeToFont(1.))
                    ),
                    (
                        @feathers::controls::FeathersMenuDivider
                    ),
                    (
                        Node {
                            width: percent(100),
                            margin: { vh(2).top() },
                            display: Display::Flex,
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Stretch,
                            row_gap: vh(2),
                        }
                        Children [
                            ui_row_fen(),
                            ui_row_side_to_move(),
                            ui_row_en_passant(),
                            ui_row_castle_rights(),
                            ui_row_counts(),
                            ui_row_action_buttons(),
                        ]
                    ),
                ]
            ),
        ]
    }
}

fn ui_row_fen() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
        }
        Children [
            (
                feathers::display::label("FEN:")
                Node { margin: { item_gap().right() } }
            ),
            (
                FenTextInputContainer
                @feathers::controls::FeathersTextInputContainer
                Node { border: { px(1).all() } }
                Children [
                    (
                        FenTextInput
                        @feathers::controls::FeathersTextInput { @visible_width: 70_f32 }
                        on(on_fen_text_edit_change)
                    ),
                ]
            ),
        ]
    }
}

pub(super) fn update_fen_text_input_invalid_styles(
    state: Res<FenInputMenuState>,
    q_container_border_color: Single<Entity, With<FenTextInputContainer>>,
    mut commands: Commands,
) {
    if state.is_changed() {
        let color = if state.is_fen_valid { Srgba::NONE } else { palettes::css::RED };
        let entity = *q_container_border_color;
        commands.entity(entity).insert(BorderColor::all(color));
    }
}

fn on_fen_text_edit_change(
    edit: On<text::TextEditChange>,
    q_editable_text: Query<&text::EditableText>,
    mut state: ResMut<FenInputMenuState>,
    mut controls: FenInputMenuControls,
) {
    let editable_text = q_editable_text.get(edit.event_target()).unwrap();
    let fen = editable_text.value().to_string();
    state.set_fen(fen);

    controls.set_side_to_move(state.side_to_move());

    controls.set_en_passant(state.en_passant_file());

    controls.set_castle_rights(state.castle_rights_white(), state.castle_rights_black());

    controls.set_halfmove_clock(state.halfmove_clock);
    controls.set_fullmove_count(state.fullmove_count);
}

fn ui_row_side_to_move() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
        }
        Children [
            (
                feathers::display::label("Side to move:")
                Node { margin: { item_gap().right() } }
            ),
            (
                ui_widgets::RadioGroup
                on(ui_widgets::radio_self_update)
                on(on_side_to_move_change)
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: item_gap(),
                }
                Children [
                    (
                        SideToMoveRadioButtonWhite
                        @feathers::controls::FeathersRadio {
                            @caption: bsn! { Text("White") feathers::theme::ThemedText }
                        }
                        ui::Checked
                    ),
                    (
                        SideToMoveRadioButtonBlack
                        @feathers::controls::FeathersRadio {
                            @caption: bsn! { Text("Black") feathers::theme::ThemedText }
                        }
                    ),
                ]
            ),
        ]
    }
}

fn on_side_to_move_change(
    change: On<ui_widgets::ValueChange<Entity>>,
    mut state: ResMut<FenInputMenuState>,
    mut controls: FenInputMenuControls,
) {
    let selected_color =
        controls.side_to_move_button_color(change.value).unwrap_or(chess::Color::White);
    let new_fen = state.set_side_to_move(selected_color);
    controls.set_fen_text_input_value(new_fen);
}

fn ui_row_en_passant() -> impl Scene {
    fn file_button(file: chess::File) -> impl Scene {
        let caption = EN_PASSANT_FILE_LABELS[file.to_index()];

        let on_activate = move |_: On<ui_widgets::Activate>,
                                mut state: ResMut<FenInputMenuState>,
                                mut controls: FenInputMenuControls| {
            let new_fen = state.set_en_passant(Some(file));
            controls.set_fen_text_input_value(new_fen);
            controls.set_en_passant_menu(Some(file));
        };

        bsn! {
            @feathers::controls::FeathersMenuItem {
                @caption: bsn! { Text(caption) feathers::theme::ThemedText }
            }
            on(on_activate)
        }
    }

    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
        }
        Children [
            (
                feathers::display::label("En Passant:")
                Node { margin: { item_gap().right() } }
            ),

            (
                EnPassantCheckbox
                @feathers::controls::FeathersCheckbox
                on(ui_widgets::checkbox_self_update)
                on(on_en_passant_checkbox_change)
            ),

            (
                EnPassantMenu
                @feathers::controls::FeathersMenu
                Visibility::Hidden
                Children [
                    (
                        ui_widgets::ActivateOnPress
                        ui_widgets::MenuButton
                        Node {
                            margin: { item_gap().left() }
                            height: size::ROW_HEIGHT,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(px(8)),
                            border_radius: item_border_radius(),
                        }
                        ui_widgets::Button
                        template_value(feathers::controls::ButtonVariant::Normal)
                        picking::hover::Hovered
                        feathers::cursor::EntityCursor::System(bevy::window::SystemCursorIcon::Pointer)
                        bevy::input_focus::tab_navigation::TabIndex(0)
                        feathers::focus::FocusIndicator
                        super::CustomButtonTheme(feathers::palette::GRAY_3)
                        feathers::font_styles::InheritableFont {
                            font: fonts::REGULAR,
                            font_size: size::MEDIUM_FONT,
                            weight: FontWeight::NORMAL,
                        }
                        Children [
                            (Text("File") feathers::theme::ThemedText),
                            Node { flex_grow: 1.0 },
                            feathers::display::icon(feathers::constants::icons::CHEVRON_DOWN),
                        ]
                    ),

                    (
                        @feathers::controls::FeathersMenuPopup
                        Children [
                            file_button(chess::File::A),
                            file_button(chess::File::B),
                            file_button(chess::File::C),
                            file_button(chess::File::D),
                            file_button(chess::File::E),
                            file_button(chess::File::F),
                            file_button(chess::File::G),
                            file_button(chess::File::H),
                        ]
                    ),
                ]
            ),

            (
                EnPassantFileLabel
                feathers::display::label_dim("A")
                Node { margin: { item_gap().left() } }
                Visibility::Hidden
            ),
        ]
    }
}

fn on_en_passant_checkbox_change(
    change: On<ui_widgets::ValueChange<bool>>,
    mut state: ResMut<FenInputMenuState>,
    mut controls: FenInputMenuControls,
) {
    let new_fen = if change.value {
        controls.set_en_passant_menu(Some(chess::File::A));
        state.set_en_passant(Some(chess::File::A))
    } else {
        controls.set_en_passant_menu(None);
        state.set_en_passant(None)
    };
    controls.set_fen_text_input_value(new_fen);
}

fn ui_row_castle_rights() -> impl Scene {
    fn ui_castle_rights_section<
        K: Component + Clone + Copy + Default + Unpin,
        Q: Component + Clone + Copy + Default + Unpin,
    >(
        color: chess::Color,
        label: &str,
        kingside_marker: K,
        queenside_marker: Q,
    ) -> impl Scene {
        bsn! {
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::Column,
                row_gap: vh(0.5),
            }
            Children [
                (
                    feathers::display::label(label)
                ),

                (
                    template_value(kingside_marker)
                    @feathers::controls::FeathersCheckbox {
                        @caption: bsn! { Text("Kingside (O-O)") feathers::theme::ThemedText },
                    }
                    on(ui_widgets::checkbox_self_update)
                    on(move |change: On<ui_widgets::ValueChange<bool>>,
                             mut state: ResMut<FenInputMenuState>,
                             mut controls: FenInputMenuControls| {
                        let new_fen = if change.value {
                            state.add_castle_rights(color, chess::CastleRights::KingSide)
                        } else {
                            state.remove_castle_rights(color, chess::CastleRights::KingSide)
                        };
                        controls.set_fen_text_input_value(new_fen);
                    })
                ),

                (
                    template_value(queenside_marker)
                    @feathers::controls::FeathersCheckbox {
                        @caption: bsn! { Text("Queenside (O-O-O)") feathers::theme::ThemedText },
                    }
                    on(ui_widgets::checkbox_self_update)
                    on(move |change: On<ui_widgets::ValueChange<bool>>,
                             mut state: ResMut<FenInputMenuState>,
                             mut controls: FenInputMenuControls| {
                        let new_fen = if change.value {
                            state.add_castle_rights(color, chess::CastleRights::QueenSide)
                        } else {
                            state.remove_castle_rights(color, chess::CastleRights::QueenSide)
                        };
                        controls.set_fen_text_input_value(new_fen);
                    })
                ),
            ]
        }
    }

    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: column_gap(),
        }
        Children [
            ui_castle_rights_section(
                chess::Color::White,
                "White Castle Rights:",
                CastleRightsWhiteKingsideCheckbox,
                CastleRightsWhiteQueensideCheckbox,
            ),
            ui_castle_rights_section(
                chess::Color::Black,
                "Black Castle Rights:",
                CastleRightsBlackKingsideCheckbox,
                CastleRightsBlackQueensideCheckbox,
            ),
        ]
    }
}

fn ui_row_counts() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: column_gap(),
        }
        Children [
            (
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                }
                Children [
                    (
                        feathers::display::label("Halfmove Clock:")
                        Node { margin: { item_gap().right() } }
                    ),
                    (
                        HalfmoveClockNumberInput
                        @feathers::controls::FeathersNumberInput {
                            @number_format: feathers::controls::NumberFormat::I32,
                        }
                        Node { width: vh(4) }
                        on(on_halfmove_input_change)
                    ),
                ]
            ),

            (
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                }
                Children [
                    (
                        feathers::display::label("Fullmove Count:")
                        Node { margin: { item_gap().right() } }
                    ),
                    (
                        FullmoveCountNumberInput
                        @feathers::controls::FeathersNumberInput {
                            @number_format: feathers::controls::NumberFormat::I32,
                        }
                        Node { width: vh(4) }
                        on(on_fullmove_input_change)
                    ),
                ]
            ),
        ]
    }
}

fn on_halfmove_input_change(
    change: On<ui_widgets::ValueChange<i32>>,
    mut state: ResMut<FenInputMenuState>,
    mut controls: FenInputMenuControls,
) {
    if change.is_final {
        let new_fen = state.set_halfmove_clock(change.value as u8);
        controls.set_fen_text_input_value(new_fen);
    }
}

fn on_fullmove_input_change(
    change: On<ui_widgets::ValueChange<i32>>,
    mut state: ResMut<FenInputMenuState>,
    mut controls: FenInputMenuControls,
) {
    if change.is_final {
        let new_fen = state.set_fullmove_count(change.value as u16);
        controls.set_fen_text_input_value(new_fen);
    }
}

fn ui_row_action_buttons() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::Center,
            column_gap: vh(2),
        }
        Children [
            (
                CancelButton
                super::CustomButtonTheme(palettes::css::RED)
                Node {
                    height: size::ROW_HEIGHT,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::horizontal(px(8)),
                    border_radius: item_border_radius(),
                }
                ui_widgets::Button
                template_value(feathers::controls::ButtonVariant::Normal)
                picking::hover::Hovered
                feathers::cursor::EntityCursor::System(bevy::window::SystemCursorIcon::Pointer)
                bevy::input_focus::tab_navigation::TabIndex(0)
                feathers::focus::FocusIndicator
                feathers::font_styles::InheritableFont {
                    font: fonts::REGULAR,
                    font_size: size::MEDIUM_FONT,
                    weight: FontWeight::NORMAL,
                }
                on(on_cancel_button_activate)
                Children [
                    (Text("Cancel") feathers::theme::ThemedText),
                ]
            ),
            (
                LoadButton
                @feathers::controls::FeathersButton {
                    @caption: bsn! { Text("Load") feathers::theme::ThemedText }
                }
                on(on_load_button_activate)
            ),
        ]
    }
}

fn on_cancel_button_activate(
    _: On<ui_widgets::Activate>,
    mut next_state: ResMut<NextState<super::MenuState>>,
) {
    next_state.set(super::MenuState::Main);
}

fn on_load_button_activate(
    _: On<ui_widgets::Activate>,
    state: Res<FenInputMenuState>,
    mut commands: Commands,
) {
    if !state.is_fen_valid {
        return;
    }
    let Ok(board) = state.try_board() else {
        return;
    };
    commands.trigger(LoadGame::in_game(board, state.halfmove_clock, state.fullmove_count));
}
