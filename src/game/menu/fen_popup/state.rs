use std::{
    collections::hash_map::DefaultHasher,
    fmt::Write,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    str::FromStr,
    sync::Arc,
};

use bevy::{ecs::system::Resource, utils::default};
use bevy_egui::egui::{Context, FontId, TextStyle};
use chess::{Board, BoardBuilder, CastleRights};

use crate::game::consts::DEFAULT_FEN;

fn compute_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn castle_rights(kingside: bool, queenside: bool) -> CastleRights {
    match (kingside, queenside) {
        (true, true) => CastleRights::Both,
        (true, false) => CastleRights::KingSide,
        (false, true) => CastleRights::QueenSide,
        (false, false) => CastleRights::NoRights,
    }
}

#[derive(Resource)]
pub struct PopupState {
    pub focus_fen: bool,
    pub fen: String,
    fen_hash: u64,
    pub invalid_fen: bool,
    controls: PopupModel,
    controls_hash: u64,
    pub(super) fonts: PopupFonts,
}

impl Default for PopupState {
    fn default() -> Self {
        let controls = PopupModel::default();
        let fen = String::new();
        Self {
            focus_fen: true,
            fen_hash: compute_hash(&fen),
            fen,
            invalid_fen: false,
            controls_hash: compute_hash(&controls),
            controls,
            fonts: PopupFonts::default(),
        }
    }
}

impl PopupState {
    pub fn reset(&mut self) {
        self.fen.clear();
        self.fen_hash = compute_hash(&self.fen);
        self.focus_fen = true;
        self.controls = default();
        self.controls_hash = compute_hash(&self.controls);
        self.invalid_fen = false;
    }

    pub(super) fn sync(&mut self) {
        let fen_hash = compute_hash(&self.fen);
        let controls_hash = compute_hash(&self.controls);

        if fen_hash != self.fen_hash {
            self.update_controls_from_fen();
            self.fen_hash = fen_hash;
            self.controls_hash = compute_hash(&self.controls);
        } else if controls_hash != self.controls_hash {
            self.update_fen_from_controls();
            self.fen_hash = compute_hash(&self.fen);
            self.controls_hash = controls_hash;
        }
    }

    fn update_controls_from_fen(&mut self) {
        let board = Board::from_str(&self.fen);
        self.invalid_fen = board.is_err();

        if let Ok(board) = board {
            self.black_to_move = board.side_to_move() == chess::Color::Black;

            let white_castle_rights = board.castle_rights(chess::Color::White);
            self.castle_rights_white_kingside = white_castle_rights.has_kingside();
            self.castle_rights_white_queenside = white_castle_rights.has_queenside();

            let black_castle_rights = board.castle_rights(chess::Color::Black);
            self.castle_rights_black_kingside = black_castle_rights.has_kingside();
            self.castle_rights_black_queenside = black_castle_rights.has_queenside();

            match board.en_passant() {
                Some(square) => {
                    self.has_en_passant_target = true;
                    self.en_passant_target_file = square.get_file();
                }
                None => self.has_en_passant_target = false,
            }

            let mut fen_iter = self.fen.split(' ').skip(4);
            let half_move = fen_iter.next().and_then(|s| s.parse().ok());
            let full_move = fen_iter.next().and_then(|s| s.parse().ok());
            if let Some(half_move) = half_move {
                self.halfmove_clock = half_move;
            }
            if let Some(full_move) = full_move {
                self.fullmove_count = full_move;
            }
        }
    }

    fn update_fen_from_controls(&mut self) {
        if self.fen.is_empty() {
            self.fen.replace_range(.., DEFAULT_FEN);
        }

        let board = BoardBuilder::from_str(&self.fen);
        self.invalid_fen = board.is_err();
        if let Ok(mut board) = board {
            let fen_black_to_move = board.get_side_to_move() == chess::Color::Black;
            if fen_black_to_move != self.black_to_move {
                board.side_to_move(!board.get_side_to_move());
            }

            let white_castle_rights = castle_rights(
                self.castle_rights_white_kingside,
                self.castle_rights_white_queenside,
            );
            board.castle_rights(chess::Color::White, white_castle_rights);
            let black_castle_rights = castle_rights(
                self.castle_rights_black_kingside,
                self.castle_rights_black_queenside,
            );
            board.castle_rights(chess::Color::Black, black_castle_rights);

            let en_passant = self.has_en_passant_target.then_some(self.en_passant_target_file);
            board.en_passant(en_passant);

            self.fen = board.to_string();

            self.fen.truncate(self.fen.len() - 3);
            let half_move = self.halfmove_clock;
            let full_move = self.fullmove_count;
            write!(&mut self.fen, "{half_move} {full_move}")
                .expect("Write halfmove clock & fullmove count to FEN");
        }
    }
}

impl Deref for PopupState {
    type Target = PopupModel;

    fn deref(&self) -> &Self::Target {
        &self.controls
    }
}

impl DerefMut for PopupState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.controls
    }
}

#[derive(Hash)]
pub struct PopupModel {
    pub black_to_move: bool,
    pub castle_rights_white_kingside: bool,
    pub castle_rights_white_queenside: bool,
    pub castle_rights_black_kingside: bool,
    pub castle_rights_black_queenside: bool,
    pub has_en_passant_target: bool,
    pub en_passant_target_file: chess::File,
    pub halfmove_clock: u8,
    pub fullmove_count: u16,
}

impl Default for PopupModel {
    fn default() -> Self {
        Self {
            black_to_move: false,
            castle_rights_white_kingside: true,
            castle_rights_white_queenside: true,
            castle_rights_black_kingside: true,
            castle_rights_black_queenside: true,
            has_en_passant_target: false,
            en_passant_target_file: chess::File::A,
            halfmove_clock: 0,
            fullmove_count: 1,
        }
    }
}

pub(super) struct PopupFonts {
    is_initialized: bool,
    title: FontId,
    label: FontId,
    sublabel: FontId,
    button: FontId,
    editable: FontId,
    drag_value: TextStyle,
}

impl Default for PopupFonts {
    fn default() -> Self {
        Self {
            is_initialized: false,
            title: FontId::new(32.0, default()),
            label: FontId::new(18.0, default()),
            sublabel: FontId::new(16.0, default()),
            button: FontId::new(20.0, default()),
            editable: FontId::new(14.5, default()),
            drag_value: TextStyle::Name(Arc::from("gambit-drag-value")),
        }
    }
}

impl PopupFonts {
    pub(super) fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    pub(super) fn initialize(&mut self, ctx: &mut Context) {
        self.is_initialized = true;
        ctx.style_mut(|style| {
            let drag_value_style = FontId::new(16.0, default());
            style.text_styles.entry(self.drag_value()).or_insert(drag_value_style);
            style.drag_value_text_style = self.drag_value();

            style.spacing.interact_size = bevy_egui::egui::vec2(16.0, 16.0);
            style.spacing.icon_width = 16.0;
        });
    }
}

macro_rules! handle_getters {
    ($( $handle:ident : $ty:ty ; )+) => {
        $( pub(super) fn $handle(&self) -> $ty { self.$handle.clone() } )+
    }
}

impl PopupFonts {
    handle_getters! {
        title: FontId;
        label: FontId;
        sublabel: FontId;
        button: FontId;
        editable: FontId;
        drag_value: TextStyle;
    }
}
