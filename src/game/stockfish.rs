use std::{
    borrow::Cow,
    collections::VecDeque,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{self, ChildStdin, Stdio},
    str::FromStr,
};

use bevy::{prelude::*, tasks::IoTaskPool};
use crossbeam_channel::{unbounded, Receiver, TryRecvError};

use crate::{
    game::board::{PieceColor, Square},
    utils::AppNoop,
};

use super::board::{BoardState, MovePiece, MovePlugin};

const STOCKFISH_EXECUTABLE: &[u8] =
    include_bytes!("../../target/stockfish/Stockfish-sf_15/src/stockfish");

pub struct StockfishPlugin;

impl Plugin for StockfishPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<MovePlugin>() {
            panic!("Attempted to add plugin without required dependency: {:?}", MovePlugin);
        }

        app.noop()
            .add_event::<SfCommand>()
            .add_systems(PostStartup, initialize_stockfish)
            .add_systems(PostUpdate, stockfish_update)
            .add_systems(PreUpdate, stockfish_move_as_black)
            .noop();
    }
}

#[derive(Clone, Debug, Event)]
enum SfCommand {
    Uci,
    IsReady,
    UciNewGame,
    Position(String), // FEN
    Go,
    Sleep(u32), // milliseconds
    Stop,
}

impl SfCommand {
    fn into_bytes(self) -> Cow<'static, [u8]> {
        match self {
            Self::Uci => Cow::Borrowed(b"uci\n"),
            Self::IsReady => Cow::Borrowed(b"isready\n"),
            Self::UciNewGame => Cow::Borrowed(b"ucinewgame\n"),
            Self::Position(fen) => Cow::Owned(format!("position fen {fen}\n").into_bytes()),
            Self::Go => Cow::Borrowed(b"go infinite\n"),
            Self::Sleep(_) => Cow::Borrowed(b""),
            Self::Stop => Cow::Borrowed(b"stop\n"),
        }
    }
}

#[derive(Resource)]
struct Stockfish {
    stdin: ChildStdin,
    response: Receiver<String>,
    command_queue: VecDeque<SfCommand>,
}

impl Stockfish {
    fn new(stdin: ChildStdin, response: Receiver<String>) -> Self {
        Self { stdin, response, command_queue: VecDeque::new() }
    }

    // fn push_cmd(&mut self, command: SfCommand) {
    //     self.command_queue.push_back(command);
    // }

    fn extend_cmds(&mut self, commands: impl IntoIterator<Item = SfCommand>) {
        self.command_queue.extend(commands);
    }

    fn write_cmd(&mut self, command: SfCommand) {
        let cmd_bytes = command.into_bytes();
        self.stdin.write_all(&cmd_bytes).expect("write command to stockfish stdin");
    }

    fn iter_responses(&self) -> impl Iterator<Item = String> + '_ {
        SfResponseIter(&self.response)
    }
}

struct SfResponseIter<'r>(&'r Receiver<String>);

impl Iterator for SfResponseIter<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.try_recv() {
            Ok(line) => Some(line),
            Err(TryRecvError::Empty) => None,
            Err(err) => {
                error!("{err}");
                panic!();
            }
        }
    }
}

fn initialize_stockfish(mut commands: Commands) {
    let stockfish_p = ensure_stockfish_executable();

    let mut proc = process::Command::new(stockfish_p)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("start stockfish");

    let stdin = proc.stdin.take().expect("get handle to stockfish stdin");
    let stdout = proc.stdout.take().expect("get handle to stockfish stdout");

    let task_pool = IoTaskPool::get();
    let (response_tx, response_rx) = unbounded::<String>();

    // Read responses from proc stdout
    let task = async move {
        let mut reader = BufReader::new(stdout);
        let mut line = String::with_capacity(1000);
        while let Ok(count) = reader.read_line(&mut line) {
            if count == 0 {
                break;
            }
            response_tx
                .send(line.trim().to_string())
                .expect("send stockfish response over channel");
            line.clear();
        }
    };
    task_pool.spawn(task).detach();

    let mut sf = Stockfish::new(stdin, response_rx);

    sf.extend_cmds([SfCommand::Uci, SfCommand::UciNewGame, SfCommand::IsReady]);

    commands.insert_resource(sf);
}

fn ensure_stockfish_executable() -> PathBuf {
    let mut config_d = dirs::config_local_dir().expect("get local config dir");
    config_d.push("gambit");

    if !config_d.exists() {
        trace!(path = %config_d.display(), "Create config dir");
        fs::create_dir_all(&config_d).expect("create config dir");
    } else if !config_d.is_dir() {
        error!(path = %config_d.display(), "Path for config directory must be a directory");
        panic!();
    }

    let mut stockfish_p = config_d;
    stockfish_p.push("stockfish");

    if stockfish_p.exists() {
        if !stockfish_p.is_file() {
            error!(path = %stockfish_p.display(), "Path for stockfish executable must be a file");
        }
    } else {
        trace!(path = %stockfish_p.display(), "Create stockfish executable");

        let mut f = File::options()
            .write(true)
            .create(true)
            .open(&stockfish_p)
            .expect("create file for stockfish executable");

        f.write_all(STOCKFISH_EXECUTABLE).expect("write to temp file");
        let mut perms = f.metadata().expect("get temp file metadata").permissions();
        perms.set_mode(0o700);
        f.set_permissions(perms).expect("set temp file permissions");
    }

    stockfish_p
}

#[derive(Clone, Copy, Debug, Default)]
enum SfState {
    #[default]
    Idle,
    WaitingUci,
    WaitingReady,
    WaitingFinishSearch,
}

fn stockfish_update(
    mut commands: Commands,
    time: Res<Time>,
    board_state: Res<BoardState>,
    mut sleep_timer: Local<Option<Timer>>,
    mut stockfish: ResMut<Stockfish>,
    mut sf_state: Local<SfState>,
) {
    if let Some(st) = &mut *sleep_timer {
        st.tick(time.delta());
        if st.just_finished() {
            *sleep_timer = None;
        } else {
            return;
        }
    }

    'is_waiting: {
        if let SfState::WaitingUci = *sf_state {
            for line in stockfish.iter_responses() {
                if line == "uciok" {
                    trace!(response = line, "Stockfish");
                    *sf_state = SfState::Idle;
                    break 'is_waiting;
                }
            }
            return;
        } else if let SfState::WaitingReady = *sf_state {
            for line in stockfish.iter_responses() {
                if line == "readyok" {
                    trace!(response = line, "Stockfish");
                    *sf_state = SfState::Idle;
                    break 'is_waiting;
                }
            }
            return;
        } else if let SfState::WaitingFinishSearch = *sf_state {
            for line in stockfish.iter_responses() {
                if line.starts_with("bestmove") {
                    trace!(response = line, "Stockfish");
                    *sf_state = SfState::Idle;

                    let mut chunks = line.splitn(3, ' ');
                    chunks.next();
                    let bestmove = chunks.next().expect("invalid bestmove response from stockfish");
                    if !bestmove.len() == 4 {
                        panic!("Unexpected move from stockfish: {bestmove}");
                    }

                    let from_sq = Square::from_str(&bestmove[0..2]).unwrap_or_else(|_| {
                        panic!("Invalid source square in move from stockfish: {bestmove}")
                    });
                    let to_sq = Square::from_str(&bestmove[2..4]).unwrap_or_else(|_| {
                        panic!("Invalid destination square in move from stockfish: {bestmove}")
                    });

                    let piece = board_state.piece(from_sq);
                    commands.entity(piece).insert(MovePiece::new(from_sq, to_sq, None, true));

                    break 'is_waiting;
                }
            }
            return;
        }
    }

    // Stockfish state is `Idle`

    while let Some(command) = stockfish.command_queue.pop_front() {
        trace!(?command, "Stockfish");
        match command {
            SfCommand::Sleep(ms) => {
                *sleep_timer = Some(Timer::from_seconds(ms as f32 / 1000.0, TimerMode::Once));
                return;
            }
            SfCommand::Uci => {
                *sf_state = SfState::WaitingUci;
                stockfish.write_cmd(command);
                return;
            }
            SfCommand::IsReady => {
                *sf_state = SfState::WaitingReady;
                stockfish.write_cmd(command);
                return;
            }
            SfCommand::Stop => {
                *sf_state = SfState::WaitingFinishSearch;
                stockfish.write_cmd(command);
                return;
            }
            _ => stockfish.write_cmd(command),
        }
    }
}

fn stockfish_move_as_black(
    board_state: Res<BoardState>,
    mut stockfish: ResMut<Stockfish>,
    mut removed: RemovedComponents<MovePiece>,
) {
    for _ in removed.read() {
        if board_state.side_to_move() == PieceColor::BLACK {
            stockfish.extend_cmds([
                SfCommand::Position(board_state.fen()),
                SfCommand::Go,
                SfCommand::Sleep(2500),
                SfCommand::Stop,
            ]);
        }
    }
}
