use std::{fmt::Write, str::FromStr};

use bevy::prelude::*;

use crate::game::consts;

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
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn set_fen(&mut self, fen: String) {
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

    pub(super) fn set_halfmove_clock(&mut self, halfmove: u8) -> String {
        self.halfmove_clock = halfmove;
        self.is_fen_valid = true;
        self.fen_with_counts()
    }

    pub(super) fn set_fullmove_count(&mut self, fullmoves: u16) -> String {
        self.fullmove_count = fullmoves;
        self.is_fen_valid = true;
        self.fen_with_counts()
    }
}

fn truncate_trailing_whitespace(s: &mut String) {
    let trailing_len = s.len() - s.trim_end().len();
    s.truncate(s.len() - trailing_len);
}
