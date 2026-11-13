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
    game::LoadGame,
    utils::{GameFontWeight, text_font},
};

pub use state::FenInputMenuState;

mod state;

pub(super) struct FenInputMenuPlugin;

impl Plugin for FenInputMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FenInputMenuState>();
        #[cfg(test)]
        app.init_resource::<FenTextEditChangeHandler>();

        app.add_observer(FenTextInputInvalid::on_add);
        app.add_observer(FenTextInputInvalid::on_remove);
        app.add_systems(OnEnter(super::MenuState::FenInput), reset_fen_input_menu);
        app.add_systems(Update, update_action_button_disabled_styles);
        app.add_systems(
            PostUpdate,
            update_fen_text_input_invalid_live
                .after(text::EditableTextSystems)
                .run_if(resource_changed::<FenInputMenuState>)
                .run_if(any_with_component::<FenTextInputLiveValidation>),
        );
    }
}

#[derive(Component, Clone, Default)]
pub(super) struct FenInputMenu;

#[derive(Component, Clone, Copy, Default)]
pub(super) struct FenTextInputContainer;

#[derive(Component, Clone, Copy, Default)]
struct FenTextInput;

/// Marks the FEN text input container while its FEN is indicated as invalid.
#[derive(Component, Clone, Copy, Default)]
struct FenTextInputInvalid;

impl FenTextInputInvalid {
    fn on_add(add: On<Add, Self>, mut q_border_color: Query<&mut BorderColor>) {
        if let Ok(mut border_color) = q_border_color.get_mut(add.entity) {
            *border_color = BorderColor::all(palettes::css::RED);
        }
    }

    fn on_remove(remove: On<Remove, Self>, mut q_border_color: Query<&mut BorderColor>) {
        if let Ok(mut border_color) = q_border_color.get_mut(remove.entity) {
            *border_color = BorderColor::all(Srgba::NONE);
        }
    }
}

/// Marks the FEN text input container while validation errors are indicated live: from a blur that
/// found the FEN invalid until a blur that finds it valid.
#[derive(Component, Clone, Copy, Default)]
struct FenTextInputLiveValidation;

#[derive(Component, Clone, Copy, Default)]
struct SideToMoveRadioGroup;

#[derive(Component, Clone, Copy, Default)]
struct SideToMoveRadioButtonWhite;

#[derive(Component, Clone, Copy, Default)]
struct SideToMoveRadioButtonBlack;

#[derive(Component, Clone, Copy, Default)]
struct EnPassantCheckbox;

#[derive(Component, Clone, Copy, Default)]
struct EnPassantMenu;

#[derive(Component, Clone, Copy, Default)]
struct EnPassantMenuPopup;

#[derive(Component, Clone, Copy, Debug)]
struct EnPassantMenuOption(chess::File);

impl Default for EnPassantMenuOption {
    fn default() -> Self {
        Self(chess::File::A)
    }
}

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
    q_fen_text_input_container: ControlQuery<'w, 's, FenTextInputContainer>,
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
    q_load_button: ControlQuery<'w, 's, LoadButton>,
}

// FEN Text Input
impl FenInputMenuControls<'_, '_> {
    fn set_fen_text_input_value(&mut self, fen: String) {
        let entity = self.q_fen_text_input.single().unwrap();
        self.commands.queue(move |world: &mut World| {
            set_editable_text_value(world, entity, &fen);
        });
    }

    fn set_fen_text_input_invalid(&mut self, invalid: bool) {
        let entity = self.q_fen_text_input_container.single().unwrap();
        let mut entity = self.commands.entity(entity);
        if invalid {
            entity.insert(FenTextInputInvalid);
        } else {
            entity.remove::<FenTextInputInvalid>();
        }
    }

    fn set_fen_text_input_live_validation(&mut self, live: bool) {
        let entity = self.q_fen_text_input_container.single().unwrap();
        let mut entity = self.commands.entity(entity);
        if live {
            entity.insert(FenTextInputLiveValidation);
        } else {
            entity.remove::<FenTextInputLiveValidation>();
        }
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
            let entity = {
                let children = world.entity(e_number_input).get::<Children>().unwrap();
                let has_editable = |c: &Entity| world.entity(*c).contains::<text::EditableText>();
                children.iter().find(has_editable).unwrap()
            };
            let text = format!("{halfmoves}");
            set_editable_text_value(world, entity, &text);
        });
    }

    fn set_fullmove_count(&mut self, fullmoves: u16) {
        let e_number_input = self.q_fullmove_number_input.single().unwrap();
        self.commands.queue(move |world: &mut World| {
            let entity = {
                let children = world.entity(e_number_input).get::<Children>().unwrap();
                let has_editable = |c: &Entity| world.entity(*c).contains::<text::EditableText>();
                children.iter().find(has_editable).unwrap()
            };
            let text = format!("{fullmoves}");
            set_editable_text_value(world, entity, &text);
        });
    }
}

// Position Controls
impl FenInputMenuControls<'_, '_> {
    /// Sets every control other than the FEN text input from the state.
    fn set_position_controls(&mut self, state: &FenInputMenuState) {
        self.set_side_to_move(state.side_to_move());

        self.set_en_passant(state.en_passant_file());

        self.set_castle_rights(state.castle_rights_white(), state.castle_rights_black());

        self.set_halfmove_clock(state.halfmove_clock());
        self.set_fullmove_count(state.fullmove_count());
    }
}

// Action Buttons
impl FenInputMenuControls<'_, '_> {
    fn set_load_button_disabled(&mut self, disabled: bool) {
        let entity = self.q_load_button.single().unwrap();
        let mut entity = self.commands.entity(entity);
        if disabled {
            entity.insert(ui::InteractionDisabled);
        } else {
            entity.remove::<ui::InteractionDisabled>();
        }
    }
}

/// Replace the contents of an `EditableText` and leave the cursor at the end of the new text.
///
/// Uses `PlainEditorDriver` directly instead of `PlainEditor` because the editor does not have
/// public methods for changing the selection (`PlainEditor::set_selection`) and
/// `PlainEditor::set_text` does not update the selection so it may be out of bounds.
///
/// The byte-based deletions are used rather than `PlainEditorDriver::select_all` because they work
/// on the text buffer directly and not the text layout. This allows updates to work even in tests
/// which have no font loaded which would cause layout-based selection logic to behave incorrectly.
///
/// TODO: Use `PlainEditor` if parley exposes a way to correctly setting text and update selection.
fn set_editable_text_value(world: &mut World, entity: Entity, value: &str) {
    world.resource_scope(|world, mut font_ctx: Mut<text::FontCx>| {
        world.resource_scope(|world, mut layout_ctx: Mut<text::LayoutCx>| {
            let mut entity = world.entity_mut(entity);
            let mut editable = entity.get_mut::<text::EditableText>().unwrap();
            if editable.value() == value {
                return;
            }
            let raw_len = editable.editor.raw_text().len();
            let mut driver = editable.editor.driver(&mut font_ctx.context, &mut layout_ctx.0);
            // Clamps a selection left stale by any earlier direct `set_text` (e.g. `EditableText::new`).
            driver.refresh_layout();
            if let Some(len) = std::num::NonZeroUsize::new(raw_len) {
                driver.delete_bytes_before_selection(len);
                driver.delete_bytes_after_selection(len);
            }
            driver.insert_or_replace_selection(value);
        });
    });
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

/// Reset the menu to its default state when it is entered by `MenuState`.
///
/// Resets the `FenInputMenuState`, the FEN text input and its validation, and all position
/// controls to the default starting board position.
fn reset_fen_input_menu(mut state: ResMut<FenInputMenuState>, mut controls: FenInputMenuControls) {
    state.reset();

    controls.set_fen_text_input_value(String::new());
    controls.set_fen_text_input_invalid(false);
    controls.set_fen_text_input_live_validation(false);

    controls.set_position_controls(&state);
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
                BorderColor
                Children [
                    (
                        FenTextInput
                        @feathers::controls::FeathersTextInput { @visible_width: 70_f32 }
                        on(on_fen_text_edit_change)
                        on(on_fen_text_input_focus_lost)
                        on(on_fen_text_input_focus_lost_rewrite_capped)
                        on(on_fen_text_input_focus_lost_set_controls)
                    ),
                ]
            ),
        ]
    }
}

// FEN Text Input Validation
//
// Validation is lazy by default so that error indicators are less intrusive while typing:
// they are only made visible on blur. Once focus is regained and the input is invalid the
// indicators begin updating live while typing. This means the input may become valid then invalid
// multiple times and the indicators are updated in both directions, live, to indicate the current
// state of validity. Validation indicators become lazy again when the input is blurred while valid.

fn on_fen_text_input_focus_lost(
    _: On<bevy::input_focus::FocusLost>,
    state: Res<FenInputMenuState>,
    mut controls: FenInputMenuControls,
) {
    let invalid = !state.is_fen_valid();
    controls.set_fen_text_input_invalid(invalid);
    controls.set_fen_text_input_live_validation(invalid);
}

/// When the FEN text input loses focus and the last halfmove clock or fullmove count was capped by
/// the state, rewrite the text input value with the capped values.
fn on_fen_text_input_focus_lost_rewrite_capped(
    _: On<bevy::input_focus::FocusLost>,
    editable_text: Single<&text::EditableText, With<FenTextInput>>,
    state: Res<FenInputMenuState>,
    mut controls: FenInputMenuControls,
) {
    let fen = editable_text.value().to_string();

    if state.is_fen_valid() && FenInputMenuState::fen_has_capped_move_counts(&fen) {
        controls.set_fen_text_input_value(state.fen_with_counts());
    }
}

/// When the FEN text input loses focus sync the position controls with the current state.
fn on_fen_text_input_focus_lost_set_controls(
    _: On<bevy::input_focus::FocusLost>,
    state: Res<FenInputMenuState>,
    mut controls: FenInputMenuControls,
) {
    controls.set_position_controls(&state);
}

/// Runs on state changes while validating live.
fn update_fen_text_input_invalid_live(
    state: Res<FenInputMenuState>,
    mut controls: FenInputMenuControls,
) {
    controls.set_fen_text_input_invalid(!state.is_fen_valid());
}

/// Counts how many times `on_fen_text_edit_change` changed the state in tests.
#[cfg(test)]
#[derive(Resource, Default)]
struct FenTextEditChangeHandler {
    update_count: usize,
}

fn on_fen_text_edit_change(
    edit: On<text::TextEditChange>,
    #[cfg(test)] mut handler: ResMut<FenTextEditChangeHandler>,
    q_editable_text: Query<&text::EditableText>,
    mut state: ResMut<FenInputMenuState>,
) {
    let editable_text = q_editable_text.get(edit.event_target()).unwrap();
    let fen = editable_text.value().to_string();

    // Only the state follows the FEN as it is typed, which keeps the load button and the live
    // validation current. The controls are set when the FEN text input loses focus.
    //
    // `TextEditChange` will fire for cursor moves, selection changes, and programmatic updates, so
    // only mark the state as changed when the FEN actually changes it.
    let mut next_state = state.clone();
    next_state.set_fen(fen);
    if state.set_if_neq(next_state) {
        #[cfg(test)]
        {
            handler.update_count += 1;
        }
    }
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
                SideToMoveRadioGroup
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
            EnPassantMenuOption(file)
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
                        EnPassantMenuPopup
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
                    ui::Checked
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
                    ui::Checked
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
                        Node { width: vh(5.5) }
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
                        Node { width: vh(5.5) }
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
        // A negative value is not a move count, so it is ignored.
        if let Ok(halfmove) = u64::try_from(change.value) {
            let new_fen = state.set_halfmove_clock(halfmove);
            controls.set_fen_text_input_value(new_fen);
        }

        // The value may have been capped or ignored, so the input must show what the state holds.
        controls.set_halfmove_clock(state.halfmove_clock());
    }
}

fn on_fullmove_input_change(
    change: On<ui_widgets::ValueChange<i32>>,
    mut state: ResMut<FenInputMenuState>,
    mut controls: FenInputMenuControls,
) {
    if change.is_final {
        // A negative value is not a move count, so it is ignored.
        if let Ok(fullmoves) = u64::try_from(change.value) {
            let new_fen = state.set_fullmove_count(fullmoves);
            controls.set_fen_text_input_value(new_fen);
        }

        // The value may have been capped or ignored, so the input must show what the state holds.
        controls.set_fullmove_count(state.fullmove_count());
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

fn update_action_button_disabled_styles(
    state: Res<FenInputMenuState>,
    mut controls: FenInputMenuControls,
) {
    if state.is_changed() {
        let disabled = !state.is_fen_valid();
        controls.set_load_button_disabled(disabled);
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
    if !state.is_fen_valid() {
        return;
    }
    let Ok(board) = state.try_board() else {
        return;
    };
    commands.trigger(LoadGame::in_game(board, state.halfmove_clock(), state.fullmove_count()));
}

#[cfg(test)]
mod tests;
