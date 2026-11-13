# FEN Input Menu: Commit on Blur

The FEN input menu currently parses the FEN text on every keystroke and only hides the result while typing. That lets the state run ahead of the position controls, and it leaves several input paths (invalid number inputs, missing move counts, and non-canonical FENs) without a defined outcome.

This plan moves the menu to a commit model: the FEN text is parsed when it is committed, and the state, the controls, and the text only change together.

## Agreed behavior

### FEN text input

The input validates in one of two modes.

- **Lazy mode (default).** Typing parses nothing. The state, the position controls, the Load button, and the border do not change.
- **Live mode.** Every edit that changes the text re-checks validity, and the red border and the Load button follow it. An event that leaves the text unchanged, such as a cursor move, does nothing. The position, the position controls, and the text still do not change until the next commit.

A blur commits the text, unless the text is the same as it was at the last commit, in which case nothing happens. The comparison is against the last commit, not the text at focus, so a blur that follows a commit made by Load or by a control edit does nothing.

- **Valid commit.** The state takes the parsed position, the text is rewritten in canonical form, the position controls are synced, the border is cleared, and the input returns to lazy mode.
- **Invalid commit.** The text is left as typed, the border turns red, the Load button is disabled, and the input enters live mode. The state and the position controls are untouched, so the state keeps the position it had.

Canonical form is what the state prints: trimmed, castle rights ordered `KQkq`, and both move counts present.

| Text at blur | Result |
| --- | --- |
| Unchanged since the last commit (including an untouched empty input) | Nothing happens |
| `  <valid FEN>  ` | Rewritten without the padding |
| `… w qkQK - 5 10` | Rewritten as `… w KQkq - 5 10` |
| `… b - -` | Rewritten as `… b - - 0 1` |
| `… w KQkq - 300 70000` | Rewritten as `… w KQkq - 255 65535` |
| Whitespace only, or emptied by an edit | Rewritten as the default FEN |
| `abc` | Left as typed, marked invalid |
| `8/8/8/8/8/8/8/8 w - - 0 1` (no kings) | Left as typed, marked invalid |
| `… w KQkq - abc -5` | Left as typed, marked invalid |
| `… w KQkq -  5 10` (double space) | Left as typed, marked invalid |

### Move counts in a typed FEN

- A count that is missing becomes its default: `0` for the halfmove clock and `1` for the fullmove count.
- A count that is present but does not parse as a `u64` makes the whole FEN invalid. That covers non-numeric text, a negative number, a number above `u64::MAX` (18446744073709551615), and the empty count that a double space leaves.
- A count that parses and is over its max is capped at `255` or `65535`.

### En passant in a typed FEN

This work does not change how the en passant field is read or printed. The `chess` crate reads only the file from it, and prints the capturable pawn's square (rank 4 or 5) where a FEN holds the square behind it (rank 3 or 6). That is a known bug in the crate and in the game, and it is in the `TODO.md` backlog.

Rewriting the text on a valid commit makes the crate's output visible in two ways.

- A typed en passant square is rewritten on the crate's rank for the side to move, so `… b KQkq e3 0 1` becomes `… b KQkq e4 0 1`. The side to move never follows the typed rank.
- A token the crate cannot read as a square, such as `x9` or `E3`, is rewritten as `-`.

No test in this plan pins either result.

### Positions that cannot become a board

A typed FEN is only valid if its position converts to a `chess::Board`. One that does not is left as typed and marked invalid, like any other invalid FEN.

- The conversion fails when a color does not have exactly one king, when the side not to move is in check, or when a castle right is given without the king and the rook on their home squares.
- An en passant square with no pawn to capture does not fail. The crate drops it from the board, so the FEN stays valid.
- The crate's conversion reads out of bounds when the side to move has no king, which aborts a debug build. The state therefore checks for exactly one king of each color before converting.

A control edit can also produce such a position, for example giving White kingside castling with no rook on h1, or changing the side to move while a king is in check. The edit still applies and the FEN is rewritten, but the input is marked invalid, with Load disabled, until a later edit or commit gives a position that can become a board. The state can therefore hold a position that cannot be loaded, but only one that a control edit produced.

### Position controls

- A control edit first commits a pending FEN edit, the same as a blur would. Typing a valid FEN and then changing a control therefore gives the typed FEN plus the change.
- It then applies the change to the state's position and rewrites the FEN text from it.
- If the result can become a board, the border is cleared and the input returns to lazy mode. If it cannot, the input is marked invalid the same way as after an invalid commit: red border, Load disabled, and live mode.
- The position controls only change on a valid commit, on a control edit, and on the reset when the menu is entered.

### Number inputs

- A blur without an edit does nothing, the same as the FEN input.
- A blur after an edit is a control edit. A value that parses as a `u64` is capped at the max, and anything else (empty, non-numeric, negative, or above `u64::MAX`) becomes the default. The state, that number input, and the FEN input are all rewritten.
- A number input has no invalid state, which is why it falls back to the default where the same count in a typed FEN marks the FEN invalid.
- Enter commits a valid value the same way. With an invalid value, including a negative one, it does nothing, and the blur replaces the value with the default later.
- Enter also does nothing for a value above `i64::MAX`, which the widget cannot parse. The blur caps it if it fits a `u64`, and replaces it with the default if it does not.

### Load button

- It is disabled while the FEN input is known to be invalid: after an invalid commit, after a control edit that gives a position that cannot become a board, and following each edit in live mode. In lazy mode it stays enabled while typing.
- Load commits a pending FEN edit first, then loads only if the FEN is valid. Typing an invalid FEN and pressing Load therefore marks the input invalid and disables the button instead of loading.

### Reset on enter

Unchanged: entering the menu resets the state, the text, the validation, and every control.

## Design

### `state.rs`

`FenInputMenuState` keeps the position from the last valid commit or control edit, and gains two remembered texts. Each lets one kind of handler skip a text it has already dealt with. The state never holds a position from text that has not been committed.

- **`committed_fen: String`**: the FEN input's text as of the last commit, or as last written by the menu. Empty by default.
- **`checked_fen: String`**: the text that `is_fen_valid` describes. A commit sets it along with `committed_fen`; a live check sets it alone.
- **`commit_fen(&mut self, fen: &str)`** replaces `set_fen` and returns one of three outcomes.
  - *Unchanged*: `fen` equals `committed_fen`. Nothing is parsed.
  - *Valid*: the position and counts are taken from the FEN, `is_fen_valid` is set, and both remembered texts become `fen_with_counts()`.
  - *Invalid*: `is_fen_valid` is cleared and both remembered texts become `fen` as typed. The position is untouched.
- **A FEN is valid** when the `chess` crate parses it, every move count that is present parses as a `u64`, and the position converts to a `chess::Board`. `commit_fen` and `check_fen` share that test.
- **`try_board`** checks for exactly one king of each color before it converts, because the crate's conversion is not safe to call without them. The validity test and the Load handler both go through it.
- **`check_fen(&mut self, fen: &str)`** is the live check. It does nothing when `fen` equals `checked_fen`. Otherwise it parses the text only to learn whether it is valid, sets `is_fen_valid`, and records the text as `checked_fen`. The parsed position is discarded, so a valid edit made in live mode is parsed twice: when it is typed, and again when the blur commits it.
- **A move count written as text** is parsed with `str::parse::<u64>()`, for FEN tokens and number inputs alike. A failure makes a typed FEN invalid, and makes a number input fall back to the default. `parse_move_counts` changes to tell a missing count, which means the default, from one that fails to parse.
- **The setters** (`set_side_to_move`, `set_en_passant`, the castle rights pair, and the two count setters) also set both remembered texts to `fen_with_counts()`, since the menu writes that text. They set `is_fen_valid` from whether the new position converts to a board, where today they always set it to true. The count setters take the parsed count or nothing, and fall back to the default.
- **Removed**: `set_fen` and `fen_has_capped_move_counts`.

### `mod.rs`

`FenInputMenuControls` gains read access to the text inputs and two shared operations.

- **`show_state`**: writes `fen_with_counts()` to the FEN input, sets the invalid and live markers from `is_fen_valid`, and sets every position control. The markers are set only when a control edit gave a position that cannot become a board. Used by a valid commit and by every control handler.
- **`commit_fen_text`**: reads the FEN input, calls `commit_fen`, and applies the outcome. Unchanged does nothing, valid calls `show_state`, and invalid sets the invalid and live markers. Used by the blur, by Load, and by the control handlers.

| Event | Handler does |
| --- | --- |
| FEN input loses focus | `commit_fen_text` |
| FEN input text edit | In live mode only: `check_fen`, then update the border if validity flipped. Text that was already checked is skipped without parsing |
| Radio, checkbox, or file menu change | `commit_fen_text`, apply the change to the state, `show_state` |
| Number input loses focus | Skip if the text is the state's count; otherwise `commit_fen_text`, set the count from the text, `show_state` |
| Number input final `ValueChange` (Enter or blur) | Skip if the value is negative or is the state's count; otherwise `commit_fen_text`, set the count from the value, `show_state` |
| Load activated | `commit_fen_text`, then load if valid |
| Menu entered | Reset, as today |

The control handlers commit first because the blur may not have run yet. A press moves focus, but the blur is only processed later in that frame, and the control's change fires on release. If both land in one frame, the handler would otherwise build on a state that does not include the typed FEN and overwrite it.

The number input widget sends the same `ValueChange` while typing, on Enter, and on blur. Only `is_final` differs: it is `false` while typing and `true` for both Enter and blur. The widget sends nothing when it cannot parse the text.

That is why each number input also needs a focus-lost observer. A blur with unparseable text produces no `ValueChange` to react to, and a final `ValueChange` alone cannot tell a blur from Enter.

Both number inputs use `NumberFormat::I64`. A non-negative value is cast to `u64` and capped, which is safe because `i64::MAX` is below `u64::MAX`, so Enter caps anything up to `i64::MAX`. A value above `i64::MAX` fails the widget's parse: Enter sends nothing, and the focus-lost observer parses the text as a `u64`, capping the value if that succeeds and using the default if it does not.

On a blur, the focus-lost observer and the final `ValueChange` handler both run. Whichever runs second finds the count already in the state and skips, so the `ValueChange` handler only does work on Enter. The result is the same in either order.

- **Removed**: the three separate FEN focus-lost observers (merged into one), `update_fen_text_input_invalid_live`, and the per-handler `set_fen_text_input_value` and `set_en_passant_menu` calls that `show_state` replaces.
- **Test-only counter**: `FenTextEditChangeHandler` becomes a count of FEN parses, so tests can pin when a parse happens and when it is skipped.

## Steps

Each step is one commit on `main` and leaves `cargo test fen_input_menu` passing. They are ordered smallest first, except where one depends on another.

### 1. `test: cover the position controls on an invalid FEN blur`

- Add `fen_input::keeps_controls_on_invalid_blur`: commit a non-default FEN, type `abc`, blur, and assert every position control still shows the non-default values.
- This passes today and stays true under the new model.

### 2. `test: assert control edits clear the invalid FEN marker`

- In the five `updates_invalid_fen` tests (`side_to_move`, `en_passant`, `castle_rights`, `halfmove_clock`, and `fullmove_count`), type the invalid FEN and blur it, assert the invalid marker is set, then assert it is gone after the control edit.
- Do not assert the border color or the live marker in these tests. The border stays covered by `fen_input_invalid`, and the end of live mode by `stops_live_validation_after_valid_blur`.

### 3. `test: add unit tests for the FEN input menu state`

- Add a `tests` module at the bottom of `state.rs` that tests `FenInputMenuState` directly, without the menu.
- Cover the behavior that the later steps keep: the default state, `fen_with_counts`, a valid FEN setting the position and the counts, an invalid FEN leaving them alone, surrounding whitespace, counts over the max being capped, and each setter returning the rewritten FEN.
- These pass today. Every later step that changes `state.rs` adds or updates its state tests in the same commit.

### 4. `fix: default missing FEN move counts and reject unparseable ones`

- Change `state.rs` so a missing count becomes `0` or `1` instead of keeping the previous count, and a count that is present but does not parse makes the FEN invalid instead of being skipped.
- Add `fen_input` tests, each typing then blurring:
  - A FEN without counts after the counts were set to 50 and 30, and a FEN with a halfmove clock only: the missing counts are the defaults in the state and the number inputs.
  - Non-numeric counts, a negative count, counts above `u64::MAX`, and the double space case: the invalid marker is set, and the state and the number inputs keep the earlier counts.
- The text assertions arrive with step 6.
- State tests: a missing count is the default, and an unparseable count leaves the state invalid with its counts unchanged.

### 5. `feat: default invalid number input values on blur`

- Switch both number inputs to `NumberFormat::I64` and their handlers to `ValueChange<i64>`. A non-negative value is cast to `u64` and capped.
- Add a focus-lost observer on each number input. The event bubbles up from the inner text input, so the observer goes on the number input entity next to the existing `ValueChange` one.
- Make an unchanged blur and an unchanged final `ValueChange` do nothing.
- Ignore a negative final `ValueChange` entirely, leaving the text for the blur to replace with the default. Today it reverts the input to the stored count.
- Tests in `halfmove_clock` and `fullmove_count`:
  - Rename `ignores_negative_value` to `defaults_negative_value` and expect the default after the blur.
  - Add `ignores_negative_final_value`: a negative final `ValueChange` without a blur leaves the state, the number input, and the FEN input untouched.
  - Add `defaults_invalid_value` for an empty input, `-`, `1e3`, and 20 nines, which does not fit a `u64`.
  - Extend `caps_at_max` with `99999999999`, `i64::MAX`, and `u64::MAX`, which the widget cannot parse but the blur caps from the text.
  - Add `normalizes_value` for `007`.
  - Add `ignores_unchanged_blur`: with `abc` committed in the FEN input, focus and blur the number input, and assert the FEN text, the marker, and the state are untouched.
- Change the test bed's number input helpers from `i32` to `i64`.
- Add a test bed helper that focuses and blurs a number input without typing. These tests must enter the menu first, because the number inputs are empty until the reset fills them.

### 6. `feat: commit the FEN input on blur`

This is the large step: the state and handler changes from the Design section, apart from the two calls step 7 adds and the board conversion step 8 adds.

State tests: `commit_fen` returns each of its three outcomes and only changes the position for a valid FEN, `check_fen` changes only the validity and skips text it has already checked, and after any setter a `commit_fen` of the rewritten FEN is unchanged.

Tests to change in `fen_input`:

- `updates_fen` becomes `commits_fen_on_blur`: the state is still the default after typing and takes the FEN after the blur.
- `keeps_controls_while_typing` becomes `keeps_state_and_controls_while_typing`, and also covers an invalid FEN leaving the Load button enabled.
- `indicates_invalid_fen` asserts the state is still valid before the blur.
- `caps_move_counts` asserts the capped state after the blur, not before.
- `keeps_uncapped_fen_as_typed` becomes `rewrites_valid_fen_on_blur`, with the cases from the table above: padding, castle order, and missing counts.
- `keeps_invalid_fen_with_capped_counts_as_typed` stays, as `keeps_invalid_fen_as_typed`, and gains the unparseable count cases from the table.
- `ignores_unchanged_fen`, `ignores_unchanged_invalid_fen`, `ignores_fen_set_by_controls`, and `rewrites_capped_fen_on_blur_when_state_is_unchanged` are rewritten around the parse count: a second blur without an edit, and a blur after a control edit, do not parse again.

Tests to add in `fen_input`:

- `keeps_empty_input_on_blur_without_edit`: focus and blur the fresh input, and it stays empty.
- `rewrites_whitespace_only_fen_as_default` and `rewrites_emptied_fen_as_default`.
- `discards_valid_edit_made_invalid_before_blur`: type a valid FEN, replace it with `abc`, blur, and assert the state and the controls still hold the earlier position.
- `live_validation_only_updates_validity`: after an invalid blur, a valid edit clears the border and enables Load while the state and the controls stay put; an invalid edit and a blur then leave the controls unchanged.
- `live_validation_ignores_unchanged_text`: in live mode, further frames without an edit do not parse again. The test bed reports a text edit every frame, which stands in for cursor moves.
- `commits_live_edit_on_blur`: after an invalid blur, a valid edit followed by a blur commits it: the state takes the position, the text is rewritten, and the controls are synced.
- `returns_to_lazy_validation_after_control_edit`: after an invalid blur and a control edit, typing `abc` is not indicated until the next blur.

Tests to change elsewhere:

- `on_enter`: `resets_state`, `resets_invalid_fen`, and `resets_on_every_enter` blur before asserting on the state.
- `load_button`: all three tests type and blur instead of setting the text without focus. Add `stays_enabled_while_typing_invalid_fen`.

### 7. `fix: commit a pending FEN edit before Load and control edits`

- Call `commit_fen_text` at the top of the Load handler and of every position control handler.
- Record `LoadGame` triggers in the test bed and add a helper that activates the Load button.
- Tests:
  - `load_button::loads_typed_fen_without_blur`: the loaded board and counts are the typed ones.
  - `load_button::rejects_invalid_typed_fen`: nothing loads, the input is marked invalid, and the button is disabled.
  - `load_button::commits_typed_fen_once`: Load followed by the blur parses once.
  - `fen_input::control_edit_commits_typed_fen_first`: for each control, type a non-default FEN without blurring, edit the control, and assert the FEN is the typed one plus the change.

### 8. `fix: mark a FEN that cannot become a board as invalid`

- Add the board conversion to the validity test in `state.rs`, behind the check for exactly one king of each color.
- Use the same guarded conversion in the Load handler.
- Make every setter set `is_fen_valid` from the board conversion, and make `show_state` set the invalid and live markers from it.
- Tests for control edits, starting from a committed FEN that makes the edit unloadable:
  - `castle_rights`: with no rook on h1, checking White kingside applies the right and rewrites the FEN, marks the input invalid, and disables Load. Unchecking it clears the marker and enables Load.
  - `side_to_move`: with the side to move in check, changing the side to move marks the input invalid, and changing it back clears the marker.
  - `halfmove_clock`: a count edit while the position cannot become a board keeps the marker.
- Tests in `fen_input`, each typing then blurring: no kings, no king for the side to move, two kings of one color, the side not to move in check, and a castle right without its rook. Each is left as typed and marked invalid, the state keeps the earlier position, and Load is disabled.
- Add a `fen_input` test that a FEN with an en passant square and no pawn to capture stays valid.
- State tests for the same cases, and for each setter setting `is_fen_valid` from the board conversion. The two cases without a king must return an error, where an unguarded conversion would abort the test run.

### 9. `test: start position control tests from a non-default FEN`

- Add `FenBuilder::non_default()` and build `non_default_fen()` from it.
- In `side_to_move`, `en_passant`, `castle_rights`, `halfmove_clock`, and `fullmove_count`, commit the non-default FEN first and build every expected FEN from that base.
- Only `castle_rights` needs new cases: the base has no rights, so it adds all four and then removes them.
- This makes the `updates_invalid_fen` tests assert "the state's position plus the change" directly.

### 10. `docs: update the FEN input menu TODO items`

- Remove the items this work settles: the negative number input, the `NumberFormat::I64` confirmation, the non-numeric count, the FEN that cannot become a board, and the unit tests for `state.rs`.
- Keep the three en passant test items. They belong to the backlog item on syncing the FEN input with the en passant control.

## Out of scope

- The `chess` crate's wrong en passant square, in the menu and in the FEN that `BoardState::fen()` sends to Stockfish. Stockfish only accepts rank 3 or 6 and ignores anything else, so it treats the position as having no en passant capture.
- Normalizing a typed en passant square: setting a wrong rank back, and changing the side to move when the rank belongs to the other color.
- Switching from the `chess` crate to `shakmaty`.
- A test for focus moving straight from one input to another.

## Verify in the running app

The test bed sets text and focus directly, so these need a real pointer and keyboard.

- Typing a FEN and clicking Load loads the typed position.
- Typing an invalid FEN and clicking Load turns the border red, disables the button, and loads nothing.
- Tabbing through the whole form from an invalid FEN leaves the invalid text in place.
- Clicking in and out of the empty FEN input on a fresh menu leaves it empty.
- Typing a FEN and clicking a control applies the change on top of the typed FEN.
- Typing a FEN that changes a castle right or the en passant square, then clicking that checkbox, flips the checkbox once. Today, deleting `K` and clicking the White kingside checkbox leaves `K` removed.
- Loading a FEN with no kings marks the input invalid and does not crash the app.
- Checking a castle right that the position cannot have turns the border red and disables Load, and unchecking it undoes both.
