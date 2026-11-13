# ToDo

**Bevy 0.19 migration**

- [ ] More FEN input menu tests
  - Invalid en passant file
  - Invalid en passant rank
  - Changing en passant rank to other color (should update side to move)
  - Validation
- [ ] Finish feathers migration
- [ ] Make sure we're using the latest gizmo APIs
  - [ ] See if we can now fix the rendering order bug
- [ ] Use `EnableOnEnter(MenuState::?)` & `DisableOnExit(MenuState::?)` for menu UI ??
- [ ] FEN input menu follow-ups
  - [ ] Check in the running app
    - Reopening the menu after typing a FEN gives an empty input and default controls
    - A move count over the max is left alone while typing, then corrected on blur
    - Clicking a checkbox that the typed FEN changes, while the FEN input has focus, may flip it twice
  - [ ] Confirm: a negative value in a number input is ignored and the input reverts to the stored value
  - [ ] Confirm: number inputs use `NumberFormat::I64` so large values reach the cap
  - [ ] Load does nothing on a FEN that parses but cannot become a board (e.g. no kings)
    - Load stays enabled and clicking it is silent
    - No tests for what Load sends
  - [ ] A FEN whose halfmove clock or fullmove count is not a number (e.g. `abc`, `-5`) is accepted as valid
    - The bad count is skipped and the state keeps the count it had before
    - The FEN text and the number input then disagree, and no error is shown
    - Either treat the FEN as invalid, or rewrite the bad count on blur like a capped one
  - [ ] Unit tests for `state.rs` (partial FENs, whitespace, capping)

**Bevy 0.20 migration**

- [ ] Replace all `bsn_list! []` with `bsn_list! {}`
- [ ] Convert all `bsn_list! { (...), ..., }` with `bsn_list! { ... -- ... }`

## Backlog

### Features

- Press any key during game over reset countdown to stay on the board
- Improve the syncing of the FEN input with the en passant control
  - Covers the en passant cases listed under "More FEN input menu tests"
  - A typed square with a valid file and an invalid rank has its rank corrected
  - A typed square with neither part valid makes the whole FEN invalid
  - Current preference: an invalid file with a valid rank also makes the whole FEN invalid
  - Undecided: whether the side to move or the rank wins when a typed FEN has them disagree
    - Investigate comparing against the last valid state to determine which wins (the one that changed)

### Bugs

- Fix the bugs caused by the `chess` crate's wrong en passant square
  - The crate prints the capturable pawn's square (rank 4 or 5) where a FEN holds the square behind it (rank 3 or 6)
  - The FEN input menu shows the crate's square, and rewrites a typed `e3` as `e4`
  - `BoardState::fen()` sends the crate's square to Stockfish, which ignores an en passant square that is not on rank 3 or 6

### Internal

- Switch from the `chess` crate to `shakmaty`
- Refactor any `On<LoadGame>` logic into one plugin
- Use state-scoped entities for menus

## Notes

| Name                    | Winner | FEN                                                                    | Next Move(s)
| ----------------------- | ------ | ---------------------------------------------------------------------- | -------------
| Default                 |        | `rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1`             |
| Castles                 |        | `r3k2r/ppp1bppp/n2qp2n/3p1b2/3P1B2/N2QP2N/PPP1BPPP/R3K2R w KQkq - 0 1` | `O-O(-O)`
| Bishop Wins             | W      | `r4bkr/ppp3pp/2n1b3/3Bp3/8/8/PPPP1PPP/RNB1K2R w KQ - 0 1`              | `Bxe6#`
| Queen Wins              | W      | `r1b2r1k/1p1n2b1/1np1pN1p/p7/3P4/qP2B3/2Q2PPP/2RR2K1 w - - 0 1`        | `Qh7#`
| Queen Wins              | B      | `2k5/ppp2p2/7q/6p1/2Nb1p2/1B3Kn1/PP2Q1P1/8 b - - 0 1`                  | `qh5#`
| Knight Wins             | B      | `8/8/P7/8/8/7k/7p/4n2K b - - 0 1`                                      | `nd3, a7, nf2#`
| Scholar's Mate          | W      | `r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 0 4`  | `Qxf7#`
| Stalemate               | W      | `8/8/8/8/8/4K3/5Q2/7k w - - 0 1`                                       | `K*`
| Stalemate               | W      | `8/8/8/2K4k/8/4N3/6R1/8 w - - 0 1`                                     | `Nf5`
| Draw: 3-move repetition | †      | `8/5Q1k/6pp/8/8/8/1K6/8 b - - 0 1`                                     | `kh8, Qf8, kh7, Qf7, kh8, Qf8, kh7, Qf7`
| Draw: 50-move rule      | †      | `8/5k2/3p4/1p1Pp2p/pP2Pp1P/P4P1K/8/8 b - - 99 50`                      | `k*`
| Promotion               |        | `8/3P4/8/2K5/5k2/8/4p3/8 w - - 0 1`                                    | `d8=Q`
| Promotion + Capture     |        | `4n3/3P4/8/2K5/5k2/8/4p3/3N4 w - - 0 1`                                | `dxe8=Q`
| Promotion to Knight     |        | `7k/7p/7B/8/8/8/2K1p1Q1/8 w - - 0 1`                                   | `*, e2e1n`
