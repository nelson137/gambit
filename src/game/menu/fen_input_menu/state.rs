use std::{
    fmt::{self, Write},
    str::FromStr,
};

use bevy::prelude::*;

use crate::game::consts;

#[derive(Resource, Clone)]
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

impl fmt::Debug for FenInputMenuState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FenInputMenuState")
            .field("board_builder", &self.board_builder.to_string())
            .field("is_fen_valid", &self.is_fen_valid)
            .field("halfmove_clock", &self.halfmove_clock)
            .field("fullmove_count", &self.fullmove_count)
            .finish()
    }
}

/// `chess::BoardBuilder` has no `PartialEq`, so the position is compared through its FEN.
impl PartialEq for FenInputMenuState {
    fn eq(&self, other: &Self) -> bool {
        self.is_fen_valid == other.is_fen_valid && self.fen_with_counts() == other.fen_with_counts()
    }
}

impl Eq for FenInputMenuState {}

impl FenInputMenuState {
    pub(super) fn is_fen_valid(&self) -> bool {
        self.is_fen_valid
    }

    pub(super) fn halfmove_clock(&self) -> u8 {
        self.halfmove_clock
    }

    pub(super) fn fullmove_count(&self) -> u16 {
        self.fullmove_count
    }

    pub(super) fn fen_with_counts(&self) -> String {
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

    pub(super) fn side_to_move(&self) -> chess::Color {
        self.board_builder.get_side_to_move()
    }

    pub(super) fn en_passant_file(&self) -> Option<chess::File> {
        self.board_builder.get_en_passant().map(|sq| sq.get_file())
    }

    pub(super) fn castle_rights_white(&self) -> chess::CastleRights {
        self.board_builder.get_castle_rights(chess::Color::White)
    }

    pub(super) fn castle_rights_black(&self) -> chess::CastleRights {
        self.board_builder.get_castle_rights(chess::Color::Black)
    }

    pub(super) fn try_board(&self) -> Result<chess::Board, chess::Error> {
        self.board_builder.try_into()
    }
}

impl FenInputMenuState {
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    /// Return whether the given FEN has a valid halfmove clock and fullmove count.
    ///
    /// `set_fen` automatically caps each to its max if the value is a valid `u64` so the state may
    /// hold a different value than the given FEN.
    pub(super) fn fen_has_capped_move_counts(fen: &str) -> bool {
        let (halfmove, fullmove) = parse_move_counts(fen.trim());
        halfmove.is_some_and(|halfmove| cap_halfmove_clock(halfmove) as u64 != halfmove)
            || fullmove.is_some_and(|fullmove| cap_fullmove_count(fullmove) as u64 != fullmove)
    }

    /// Move counts over their max are capped.
    pub(super) fn set_fen(&mut self, fen: String) {
        let fen = fen.trim();
        let fen = if fen.is_empty() { consts::DEFAULT_FEN } else { fen };

        self.is_fen_valid = match chess::BoardBuilder::from_str(fen) {
            Ok(next) => {
                self.board_builder = next;

                let (halfmove, fullmove) = parse_move_counts(fen);
                if let Some(halfmove) = halfmove {
                    self.halfmove_clock = cap_halfmove_clock(halfmove);
                }
                if let Some(fullmove) = fullmove {
                    self.fullmove_count = cap_fullmove_count(fullmove);
                }

                true
            }
            Err(_) => false,
        };
    }

    pub(super) fn set_side_to_move(&mut self, color: chess::Color) -> String {
        self.board_builder.side_to_move(color);
        self.is_fen_valid = true;
        self.fen_with_counts()
    }

    pub(super) fn set_en_passant(&mut self, file: Option<chess::File>) -> String {
        self.board_builder.en_passant(file);
        self.is_fen_valid = true;
        self.fen_with_counts()
    }

    pub(super) fn add_castle_rights(
        &mut self,
        color: chess::Color,
        rights: chess::CastleRights,
    ) -> String {
        let castle_rights = self.board_builder.get_castle_rights(color);
        self.board_builder.castle_rights(color, castle_rights.add(rights));
        self.is_fen_valid = true;
        self.fen_with_counts()
    }

    pub(super) fn remove_castle_rights(
        &mut self,
        color: chess::Color,
        rights: chess::CastleRights,
    ) -> String {
        let castle_rights = self.board_builder.get_castle_rights(color);
        self.board_builder.castle_rights(color, castle_rights.remove(rights));
        self.is_fen_valid = true;
        self.fen_with_counts()
    }

    /// A halfmove clock over its max is capped.
    pub(super) fn set_halfmove_clock(&mut self, halfmove: u64) -> String {
        self.halfmove_clock = cap_halfmove_clock(halfmove);
        self.is_fen_valid = true;
        self.fen_with_counts()
    }

    /// A fullmove count over its max is capped.
    pub(super) fn set_fullmove_count(&mut self, fullmoves: u64) -> String {
        self.fullmove_count = cap_fullmove_count(fullmoves);
        self.is_fen_valid = true;
        self.fen_with_counts()
    }
}

/// The move counts of a FEN as they are written, which may not fit their types.
fn parse_move_counts(fen: &str) -> (Option<u64>, Option<u64>) {
    let mut fen_iter = fen.split(' ').skip(4);
    let halfmove = fen_iter.next().and_then(|s| s.parse().ok());
    let fullmove = fen_iter.next().and_then(|s| s.parse().ok());
    (halfmove, fullmove)
}

fn cap_halfmove_clock(halfmove: u64) -> u8 {
    halfmove.min(u64::from(u8::MAX)) as u8
}

fn cap_fullmove_count(fullmoves: u64) -> u16 {
    fullmoves.min(u64::from(u16::MAX)) as u16
}

fn truncate_trailing_whitespace(s: &mut String) {
    let trailing_len = s.len() - s.trim_end().len();
    s.truncate(s.len() - trailing_len);
}
